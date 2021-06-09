use crate::Request;
use std::io::{self, Write};

use tracing::debug;

macro_rules! send {
    ($b:expr, $o:expr) => {
        let response = $o.prepare()?;

        debug!("SEND {}", response.as_str());

        write!($b, "Content-Length: {}\r\n\r\n{}", response.len(), response)?;
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Success,
    Error,
    ErrorMessage(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputVariant {
    InitializeResponse(Option<dap::Capabilities>),
    LaunchResponse,
    DisconnectResponse,
    Output(String),
    EvaluateResponse(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputBatch {
    outputs: Vec<Output>,
}

impl From<Output> for OutputBatch {
    fn from(output: Output) -> Self {
        Self {
            outputs: vec![output],
        }
    }
}

impl OutputBatch {
    pub fn batch(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn send<O>(self, mut output: O) -> io::Result<()>
    where
        O: Write,
    {
        for o in self.outputs {
            send!(output, o);
        }

        output.flush()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    variant: OutputVariant,
    command: String,
    message: Option<String>,
    request_seq: i64,
    seq: i64,
    success: bool,
}

impl Output {
    pub fn new(request: &Request, status: Status, variant: OutputVariant) -> Self {
        let command = request.command().to_owned();
        let seq = request.seq();
        let request_seq = seq;

        let (success, message) = match status {
            Status::Success => (true, None),
            Status::Error => (false, None),
            Status::ErrorMessage(e) => (false, Some(e)),
        };

        Self {
            variant,
            command,
            message,
            request_seq,
            seq,
            success,
        }
    }

    pub const fn is_event(&self) -> bool {
        match &self.variant {
            &OutputVariant::Output(_) => true,
            _ => false,
        }
    }

    pub fn batch(self, output: Output) -> OutputBatch {
        OutputBatch::from(self).batch(output)
    }

    pub fn prepare(self) -> io::Result<String> {
        let type_ = if self.is_event() { "event" } else { "response" };
        let type_ = type_.to_owned();

        let command = self.command;
        let message = self.message;
        let request_seq = self.request_seq;
        let seq = self.seq;
        let success = self.success;

        match self.variant {
            OutputVariant::InitializeResponse(body) => {
                serde_json::to_string(&dap::InitializeResponse {
                    body,
                    command,
                    message,
                    request_seq,
                    seq,
                    success,
                    type_,
                })
            }

            OutputVariant::LaunchResponse => serde_json::to_string(&dap::LaunchResponse {
                body: None,
                command,
                message,
                request_seq,
                seq,
                success,
                type_,
            }),

            OutputVariant::DisconnectResponse => serde_json::to_string(&dap::DisconnectResponse {
                body: None,
                command,
                message,
                request_seq,
                seq,
                success,
                type_,
            }),

            OutputVariant::Output(output) => serde_json::to_string(&dap::OutputEvent {
                body: dap::OutputEventBody {
                    category: Some("console".to_owned()),
                    column: None,
                    data: None,
                    group: None,
                    line: None,
                    output,
                    source: None,
                    variables_reference: None,
                },
                event: "output".to_owned(),
                seq,
                type_,
            }),

            OutputVariant::EvaluateResponse(result) => {
                serde_json::to_string(&dap::EvaluateResponse {
                    body: dap::EvaluateResponseBody {
                        indexed_variables: None,
                        memory_reference: None,
                        named_variables: None,
                        presentation_hint: None,
                        type_: None,
                        variables_reference: 0,
                        result,
                    },
                    command,
                    message,
                    request_seq,
                    seq,
                    success,
                    type_,
                })
            }
        }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub fn send<O>(self, mut output: O) -> io::Result<()>
    where
        O: Write,
    {
        send!(output, self);

        output.flush()
    }
}
