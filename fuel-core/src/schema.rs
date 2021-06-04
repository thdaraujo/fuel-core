use async_graphql::{Context, Object, Schema, Subscription, ID};
use futures::Stream;
use tracing::trace;
use uuid::Uuid;

use std::{pin, task};

#[derive(Debug, Clone, Copy)]
pub enum DebugEvent {
    Mock,
}

#[Object]
impl DebugEvent {
    async fn description(&self) -> &str {
        "Mock debug event"
    }
}

pub type Storage = ();
pub struct QueryRoot;
pub struct MutationRoot;

#[derive(Debug, Default)]
pub struct SubscriptionRoot {}

pub type DebugSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(Clone)]
pub struct DebugBroker {
    session: ID,
    sent: bool,
}

impl DebugBroker {
    pub fn new(session: ID) -> Self {
        Self {
            session,
            sent: false,
        }
    }
}

impl Stream for DebugBroker {
    type Item = DebugEvent;

    fn poll_next(
        mut self: pin::Pin<&mut Self>,
        _ctx: &mut task::Context<'_>,
    ) -> task::Poll<Option<DebugEvent>> {
        trace!("Subscription queried");

        if self.sent {
            self.sent = false;
            task::Poll::Ready(Some(DebugEvent::Mock))
        } else {
            task::Poll::Ready(None)
        }
    }
}

#[Object]
impl QueryRoot {
    async fn events(&self, _ctx: &Context<'_>) -> async_graphql::Result<Vec<DebugEvent>> {
        Ok(vec![DebugEvent::Mock])
    }
}

#[Object]
impl MutationRoot {
    async fn start_session(&self, _ctx: &Context<'_>) -> async_graphql::Result<ID> {
        Ok(Uuid::new_v4().into())
    }
}

#[Subscription]
impl SubscriptionRoot {
    // TODO use std::stream::Stream
    // Nightly-only https://github.com/rust-lang/rust/issues/79024
    async fn events(
        &self,
        _ctx: &Context<'_>,
        session: ID,
    ) -> async_graphql::Result<impl Stream<Item = DebugEvent>> {
        Ok(DebugBroker::new(session))
    }
}
