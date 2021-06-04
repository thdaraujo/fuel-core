use serde_json::Value;

use std::str::{self, FromStr};
use std::{io, net};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    url: surf::Url,
}

impl Client {
    pub fn new(url: impl AsRef<str>) -> Result<Self, net::AddrParseError> {
        Self::from_str(url.as_ref())
    }

    pub async fn start_session(&self) -> io::Result<String> {
        let reply = surf::post(&self.url)
            .header("Content-Type", "application/json")
            .body(r#"{"query": "mutation { startSession }"}"#)
            .recv_string()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let reply: Value = serde_json::from_str(reply.as_str())?;
        let id = reply
            .get("data")
            .and_then(|d| d.get("startSession"))
            .and_then(|s| s.as_str())
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Failed to fetch session ID from response",
            ))?;

        Ok(id.to_owned())
    }
}

impl FromStr for Client {
    type Err = net::AddrParseError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        str.parse().map(|s: net::SocketAddr| s.into())
    }
}

impl<S> From<S> for Client
where
    S: Into<net::SocketAddr>,
{
    fn from(socket: S) -> Self {
        let url = format!("http://{}/sway-dap", socket.into())
            .as_str()
            .parse()
            .unwrap();

        Self { url }
    }
}
