use serde_json::Value;

use std::str::{self, FromStr};
use std::{io, net};

use fuel_vm::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    url: surf::Url,
}

impl Client {
    pub fn new(url: impl AsRef<str>) -> Result<Self, net::AddrParseError> {
        Self::from_str(url.as_ref())
    }

    async fn query(&self, q: &str) -> io::Result<Value> {
        let reply = surf::post(&self.url)
            .header("Content-Type", "application/json")
            .body(q)
            .recv_string()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(serde_json::from_str(reply.as_str())?)
    }

    pub async fn start_session(&self) -> io::Result<String> {
        let reply = self
            .query(r#"{"query": "mutation { startSession }"}"#)
            .await?;

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

    pub async fn end_session(&self, id: &str) -> io::Result<bool> {
        let query = format!("{{\"query\": \"mutation($id:String!) {{ endSession(id:$id) }}\", \"variables\": {{\"id\": \"{}\"}}}}", id);
        let reply = self.query(query.as_str()).await?;

        reply
            .get("data")
            .and_then(|d| d.get("endSession"))
            .map(|d| d == &Value::Bool(true))
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Failed to fetch response from endSession query",
            ))
    }

    pub async fn reset(&self, id: &str) -> io::Result<bool> {
        let query = format!("{{\"query\": \"mutation($id:String!) {{ reset(id:$id) }}\", \"variables\": {{\"id\": \"{}\"}}}}", id);
        let reply = self.query(query.as_str()).await?;

        reply
            .get("data")
            .and_then(|d| d.get("endSession"))
            .map(|d| d == &Value::Bool(true))
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Failed to fetch response from endSession query",
            ))
    }

    pub async fn register(&self, id: &str, register: RegisterId) -> io::Result<Word> {
        let query = format!("{{\"query\": \"query($id:String!, $register:Int!) {{ register(id:$id, register:$register) }}\", \"variables\": {{\"id\": \"{}\", \"register\": {}}}}}", id, register);
        let reply = self.query(query.as_str()).await?;

        Ok(reply
            .get("data")
            .and_then(|d| d.get("register").cloned())
            .map(|v| serde_json::from_value::<Word>(v))
            .transpose()?
            .unwrap_or(0))
    }

    pub async fn memory(&self, id: &str, start: usize, size: usize) -> io::Result<String> {
        let query = format!("{{\"query\": \"query($id:String!, $start:Int!, $size:Int!) {{ memory(id:$id, start:$start, size:$size) }}\", \"variables\": {{\"id\": \"{}\", \"start\": {}, \"size\": {}}}}}", id, start, size);
        let reply = self.query(query.as_str()).await?;

        Ok(reply
            .get("data")
            .and_then(|d| d.get("memory"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_owned())
            .unwrap_or_default())
    }

    pub async fn execute(&self, id: &str, op: Opcode) -> io::Result<bool> {
        let op = hex::encode(&op.to_bytes());
        let query = format!("{{\"query\": \"mutation($id:String!, $op:String!) {{ execute(id:$id, op:$op) }}\", \"variables\": {{\"id\": \"{}\", \"op\": \"{}\"}}}}", id, op);
        let reply = self.query(query.as_str()).await?;

        Ok(reply
            .get("data")
            .and_then(|d| d.get("execute"))
            .map(|d| d == &Value::Bool(true))
            .unwrap_or(false))
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
