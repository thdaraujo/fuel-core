use crate::{schema, Request};
use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Success,
    Error,
    ErrorMessage(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseVariant {
    InitializeResponse(Option<schema::Capabilities>),
    LaunchResponse,
    DisconnectResponse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    variant: ResponseVariant,
    command: String,
    message: Option<String>,
    request_seq: i64,
    seq: i64,
    success: bool,
    type_: String,
}

impl Response {
    pub fn new(request: &Request, status: Status, variant: ResponseVariant) -> Self {
        let command = request.command().to_owned();
        let seq = request.seq();
        let request_seq = seq;
        let type_ = "response".to_owned();

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
            type_,
        }
    }

    pub fn send<O>(self, mut output: O) -> io::Result<()>
    where
        O: Write,
    {
        let command = self.command;
        let message = self.message;
        let request_seq = self.request_seq;
        let seq = self.seq;
        let success = self.success;
        let type_ = self.type_;

        let response = match self.variant {
            ResponseVariant::InitializeResponse(body) => {
                serde_json::to_string(&schema::InitializeResponse {
                    body,
                    command,
                    message,
                    request_seq,
                    seq,
                    success,
                    type_,
                })?
            }

            ResponseVariant::LaunchResponse => serde_json::to_string(&schema::LaunchResponse {
                body: None,
                command,
                message,
                request_seq,
                seq,
                success,
                type_,
            })?,

            ResponseVariant::DisconnectResponse => {
                serde_json::to_string(&schema::DisconnectResponse {
                    body: None,
                    command,
                    message,
                    request_seq,
                    seq,
                    success,
                    type_,
                })?
            }
        };

        debug!("SEND {}", response.as_str());

        write!(
            output,
            "Content-Length: {}\r\n\r\n{}",
            response.len(),
            response
        )?;

        output.flush()?;

        Ok(())
    }
}
