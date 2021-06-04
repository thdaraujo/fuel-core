use crate::{schema, Message};
use std::convert::TryFrom;
use std::io;

use serde::{Deserialize, Serialize};
use tracing::trace;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    Initialize(Message, schema::Request),
    Launch(Message, schema::Request),
    Disconnect(Message, schema::Request, schema::DisconnectArguments),
    Unimplemented(Message, schema::Request),
}

impl Request {
    pub const fn schema(&self) -> Option<&schema::Request> {
        match &self {
            Self::Initialize(_, r) => Some(r),
            Self::Launch(_, r) => Some(r),
            Self::Disconnect(_, r, _) => Some(r),
            Self::Unimplemented(_, r) => Some(r),
        }
    }

    pub const fn message(&self) -> &Message {
        match &self {
            Self::Initialize(m, _) => m,
            Self::Launch(m, _) => m,
            Self::Disconnect(m, _, _) => m,
            Self::Unimplemented(m, _) => m,
        }
    }

    pub const fn seq(&self) -> i64 {
        self.message().seq()
    }

    pub fn command(&self) -> &str {
        self.schema()
            .map(|s| s.command.as_str())
            .unwrap_or("invalid")
    }
}

impl TryFrom<Message> for Request {
    type Error = io::Error;

    fn try_from(m: Message) -> io::Result<Self> {
        trace!("Request received");

        let request: schema::Request = serde_json::from_str(m.request())?;

        let request = match request.command.as_str() {
            "initialize" => Self::Initialize(m, request),
            "launch" => Self::Launch(m, request),

            "disconnect" => {
                let args = request.clone().arguments.ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "The arguments are mandatory for disconnect request!",
                ))?;
                let args = serde_json::from_value(args)?;

                Self::Disconnect(m, request, args)
            }

            _ => Self::Unimplemented(m, request),
        };

        Ok(request)
    }
}
