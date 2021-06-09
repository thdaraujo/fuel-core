use crate::{
    Client, Expression, Instruction, Message, MessageClass, Output, OutputVariant, Request, Status,
};
use std::convert::TryFrom;
use std::io::{self, BufRead, Write};
use std::net;
use std::str::FromStr;

use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct Service {
    session: Option<String>,
    dap: Client,
}

impl From<net::SocketAddr> for Service {
    fn from(socket: net::SocketAddr) -> Self {
        let dap = Client::from(socket);
        let session = None;

        Self { dap, session }
    }
}

impl Service {
    pub async fn handle<I, O>(&mut self, input: I, output: O) -> Result<bool, io::Error>
    where
        I: BufRead,
        O: Write,
    {
        let message = Message::from_buffer(input)?;

        let class = message.class();
        let proceed = match class {
            MessageClass::Request => self.handle_request(output, message).await?,

            MessageClass::Unimplemented => {
                warn!("Message class '{}' not implemented", message.schema_type());

                true
            }
        };

        Ok(proceed)
    }

    async fn handle_request<O>(&mut self, mut output: O, message: Message) -> io::Result<bool>
    where
        O: Write,
    {
        let request = Request::try_from(message)?;

        debug!("{} request accepted", request.command());
        let proceed = match self.handle_request_inner(output.by_ref(), &request).await {
            Ok(p) => p,
            Err(e) => {
                Output::new(
                    &request,
                    Status::Error,
                    OutputVariant::Output(format!("{}", e)),
                )
                .send(output.by_ref())?;

                return Err(e);
            }
        };

        Ok(proceed)
    }

    async fn handle_request_inner<O>(&mut self, output: O, request: &Request) -> io::Result<bool>
    where
        O: Write,
    {
        let mut proceed = true;

        match request {
            Request::Initialize(_, _) => Output::new(
                request,
                Status::Success,
                OutputVariant::InitializeResponse(None),
            )
            .send(output)?,

            Request::Launch(_, _) => {
                let session = self.dap.start_session().await?;
                info!("Debug session {} started", session);

                Output::new(request, Status::Success, OutputVariant::LaunchResponse)
                    .batch(Output::new(
                        request,
                        Status::Success,
                        OutputVariant::Output(format!("Session {} started", session)),
                    ))
                    .send(output)?;

                self.session.replace(session);
            }

            Request::Disconnect(_, _, args) => {
                proceed = args.restart.unwrap_or(true);
                let session = self.session.take();

                match session {
                    Some(s) if proceed => {
                        let reset = self.dap.reset(s.as_str()).await?;
                        let status = if reset {
                            debug!("Instance {} reset", s);
                            Status::Success
                        } else {
                            debug!("Failed to reset instance {}", s);
                            Status::ErrorMessage("Failed to reset VM instance!".to_owned())
                        };

                        Output::new(request, status, OutputVariant::DisconnectResponse)
                            .send(output)?
                    }

                    Some(s) if self.dap.end_session(s.as_str()).await? => {
                        debug!("Disconnect session {} executed!", s.as_str());

                        Output::new(request, Status::Success, OutputVariant::DisconnectResponse)
                            .send(output)?
                    }

                    Some(s) => {
                        warn!(
                            "Disconnect session {} attempted but drop failed!",
                            s.as_str()
                        );

                        Output::new(
                            request,
                            Status::ErrorMessage("Backend failed to drop session!".to_owned()),
                            OutputVariant::DisconnectResponse,
                        )
                        .send(output)?
                    }

                    None => {
                        debug!("Disconnect request received without initialized session!");

                        Output::new(
                            request,
                            Status::ErrorMessage("No session initialized!".to_owned()),
                            OutputVariant::DisconnectResponse,
                        )
                        .send(output)?
                    }
                }
            }

            Request::Evaluate(_, _, args) => {
                let instruction = Instruction::from_str(args.expression.as_str())?;
                let s = self.session.as_ref().ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    "Debug session not initialized!",
                ))?;

                let result = match instruction {
                    Instruction::Print(Expression::Word(w)) => format!("0x{:x}", w),
                    Instruction::Print(Expression::Register(r)) => {
                        format!("0x{:x}", self.dap.register(s.as_str(), r).await?)
                    }
                    Instruction::Print(Expression::Memory(start, size)) => {
                        format!("0x{}", self.dap.memory(s.as_str(), start, size).await?)
                    }
                    Instruction::Exec(op) => match self.dap.execute(s.as_str(), op).await {
                        Ok(true) => "".to_owned(),
                        _ => "Failed to execute provided command!".to_owned(),
                    },
                    Instruction::Quit => match &self.session {
                        Some(s) => {
                            debug!("Received 'quit' instruction");
                            self.dap.end_session(s.as_str()).await?;
                            proceed = false;
                            "Bye!".to_owned()
                        }
                        None => "Received 'quit' instruction with no active session".to_owned(),
                    },
                };

                if !result.is_empty() {
                    Output::new(
                        request,
                        Status::Success,
                        OutputVariant::EvaluateResponse(result),
                    )
                    .send(output)?
                }
            }

            _ => warn!("Request '{}' not implemented", request.command()),
        }

        Ok(proceed)
    }
}
