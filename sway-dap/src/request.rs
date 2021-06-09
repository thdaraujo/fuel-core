use crate::Message;
use std::convert::TryFrom;
use std::io;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::trace;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    Initialize(Message, dap::Request),
    Launch(Message, dap::Request),
    Disconnect(Message, dap::Request, dap::DisconnectArguments),
    Evaluate(Message, dap::Request, dap::EvaluateArguments),
    Unimplemented(Message, dap::Request),
}

impl Request {
    pub const fn schema(&self) -> Option<&dap::Request> {
        match &self {
            Self::Initialize(_, r) => Some(r),
            Self::Launch(_, r) => Some(r),
            Self::Disconnect(_, r, _) => Some(r),
            Self::Evaluate(_, r, _) => Some(r),
            Self::Unimplemented(_, r) => Some(r),
        }
    }

    pub const fn message(&self) -> &Message {
        match &self {
            Self::Initialize(m, _) => m,
            Self::Launch(m, _) => m,
            Self::Disconnect(m, _, _) => m,
            Self::Evaluate(m, _, _) => m,
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

    pub fn args<A>(request: &dap::Request) -> io::Result<A>
    where
        A: DeserializeOwned,
    {
        let args = request.clone().arguments.ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "The arguments are mandatory!",
        ))?;

        Ok(serde_json::from_value(args)?)
    }
}

impl TryFrom<Message> for Request {
    type Error = io::Error;

    fn try_from(m: Message) -> io::Result<Self> {
        trace!("Request received");

        let request: dap::Request = serde_json::from_str(m.request())?;

        let request = match request.command.as_str() {
            "initialize" => Self::Initialize(m, request),
            "launch" => Self::Launch(m, request),
            "disconnect" => Self::args(&request).map(|args| Self::Disconnect(m, request, args))?,
            "evaluate" => Self::args(&request).map(|args| Self::Evaluate(m, request, args))?,

            _ => Self::Unimplemented(m, request),
        };

        Ok(request)
    }
}
