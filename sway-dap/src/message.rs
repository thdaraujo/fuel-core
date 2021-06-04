use crate::schema;
use std::io::{self, BufRead};

use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MessageClass {
    Request,
    Unimplemented,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    request: String,
    class: MessageClass,
    message: schema::ProtocolMessage,
}

impl Message {
    pub const fn schema(&self) -> &schema::ProtocolMessage {
        &self.message
    }

    pub const fn class(&self) -> MessageClass {
        self.class
    }

    pub const fn seq(&self) -> i64 {
        self.message.seq
    }

    pub fn schema_type(&self) -> &str {
        self.message.type_.as_str()
    }

    pub fn request(&self) -> &str {
        self.request.as_str()
    }

    pub fn from_buffer<I>(mut input: I) -> Result<Self, io::Error>
    where
        I: BufRead,
    {
        let mut buffer = String::new();

        input.read_line(&mut buffer)?;

        trace!("Received header: {}", buffer);

        let mut h = buffer.as_str().split_whitespace();
        let size = match (h.next(), h.next()) {
            (Some(p), Some(v)) if p.to_lowercase().as_str() == "content-length:" => {
                usize::from_str_radix(v, 10)
            }

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Malformed DAP header!",
            ))?,
        }
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // TODO check a better strategy to skip from headers to content
        input.read_line(&mut buffer)?;

        trace!("Reading {} bytes", size);

        let mut request = vec![0u8; size];
        let n = input.read(request.as_mut())?;

        if size != n {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Failed to read the whole request",
            ))?;
        }

        let request = String::from_utf8(request)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        debug!("RECV {}", request);

        let message: schema::ProtocolMessage = serde_json::from_str(request.as_str())?;
        let class = match message.type_.as_str() {
            "request" => MessageClass::Request,
            _ => MessageClass::Unimplemented,
        };

        trace!("Parsed message");

        Ok(Self {
            request,
            class,
            message,
        })
    }
}
