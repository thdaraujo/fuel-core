use cynic::http::SurfExt;
use cynic::{MutationBuilder, Operation, QueryBuilder};

use fuel_vm::prelude::*;
use std::str::{self, FromStr};
use std::{io, net};

mod schema {
    cynic::use_schema!("../dap/assets/debug.sdl");
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "../dap/assets/debug.sdl", graphql_type = "MutationRoot")]
struct StartSession {
    pub start_session: cynic::Id,
}

#[derive(cynic::FragmentArguments)]
struct IdArg {
    id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../dap/assets/debug.sdl",
    graphql_type = "MutationRoot",
    argument_struct = "IdArg"
)]
struct EndSession {
    #[arguments(id = &args.id)]
    pub end_session: bool,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../dap/assets/debug.sdl",
    graphql_type = "MutationRoot",
    argument_struct = "IdArg"
)]
struct Reset {
    #[arguments(id = &args.id)]
    pub reset: bool,
}

#[derive(cynic::FragmentArguments)]
struct ExecuteArgs {
    id: cynic::Id,
    op: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../dap/assets/debug.sdl",
    graphql_type = "MutationRoot",
    argument_struct = "ExecuteArgs"
)]
struct Execute {
    #[arguments(id = &args.id, op = &args.op)]
    pub execute: bool,
}

#[derive(cynic::FragmentArguments)]
struct RegisterArgs {
    id: cynic::Id,
    register: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../dap/assets/debug.sdl",
    graphql_type = "QueryRoot",
    argument_struct = "RegisterArgs"
)]
struct Register {
    #[arguments(id = &args.id, register = &args.register)]
    pub register: i32,
}

#[derive(cynic::FragmentArguments)]
struct MemoryArgs {
    id: cynic::Id,
    start: i32,
    size: i32,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../dap/assets/debug.sdl",
    graphql_type = "QueryRoot",
    argument_struct = "MemoryArgs"
)]
struct Memory {
    #[arguments(id = &args.id, start = &args.start, size = &args.size)]
    pub memory: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DapClient {
    url: surf::Url,
}

impl FromStr for DapClient {
    type Err = net::AddrParseError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        str.parse().map(|s: net::SocketAddr| s.into())
    }
}

impl<S> From<S> for DapClient
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

impl DapClient {
    pub fn new(url: impl AsRef<str>) -> Result<Self, net::AddrParseError> {
        Self::from_str(url.as_ref())
    }

    async fn query<'a, R: 'a>(&self, q: Operation<'a, R>) -> io::Result<R> {
        surf::post(&self.url)
            .run_graphql(q)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            .data
            .ok_or(io::Error::new(io::ErrorKind::NotFound, "Invalid response"))
    }

    pub async fn start_session(&self) -> io::Result<String> {
        let query = StartSession::build(&());

        self.query(query)
            .await
            .map(|r| r.start_session.into_inner())
    }

    pub async fn end_session(&self, id: &str) -> io::Result<bool> {
        let query = EndSession::build(&IdArg { id: id.into() });

        self.query(query).await.map(|r| r.end_session)
    }

    pub async fn reset(&self, id: &str) -> io::Result<bool> {
        let query = Reset::build(&IdArg { id: id.into() });

        self.query(query).await.map(|r| r.reset)
    }

    pub async fn execute(&self, id: &str, op: &Opcode) -> io::Result<bool> {
        let op = hex::encode(&op.to_bytes());
        let query = Execute::build(&ExecuteArgs { id: id.into(), op });

        self.query(query).await.map(|r| r.execute)
    }

    pub async fn register(&self, id: &str, register: RegisterId) -> io::Result<Word> {
        let query = Register::build(&RegisterArgs {
            id: id.into(),
            register: register as i32,
        });

        Ok(self.query(query).await?.register as Word)
    }

    pub async fn memory(&self, id: &str, start: usize, size: usize) -> io::Result<Vec<u8>> {
        let query = Memory::build(&MemoryArgs {
            id: id.into(),
            start: start as i32,
            size: size as i32,
        });

        let memory = self.query(query).await?.memory;

        hex::decode(memory).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
