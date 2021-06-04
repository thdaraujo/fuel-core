use crate::{Client, Message, MessageClass, Request, Response, ResponseVariant, Status};
use std::convert::TryFrom;
use std::io::{self, BufRead, Write};
use std::net;

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

    async fn handle_request<O>(&mut self, output: O, message: Message) -> io::Result<bool>
    where
        O: Write,
    {
        let request = Request::try_from(message)?;
        let mut proceed = true;

        debug!("{} request accepted", request.command());
        let response = match &request {
            Request::Initialize(_, _) => Response::new(
                &request,
                Status::Success,
                ResponseVariant::InitializeResponse(None),
            ),

            Request::Launch(_, _) => {
                let session = self.dap.start_session().await?;
                info!("Debug session {} started", session);

                self.session.replace(session);

                Response::new(&request, Status::Success, ResponseVariant::LaunchResponse)
            }

            Request::Disconnect(_, _, args) => {
                proceed = args.restart.unwrap_or(true);
                Response::new(
                    &request,
                    Status::Success,
                    ResponseVariant::DisconnectResponse,
                )
            }

            _ => {
                warn!("Request '{}' not implemented", request.command());

                return Ok(true);
            }
        };

        response.send(output)?;

        Ok(proceed)
    }
}
