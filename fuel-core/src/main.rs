use schema::{DebugSchema, MutationRoot, QueryRoot, Storage, SubscriptionRoot};

use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};
use async_graphql::Schema;
use async_graphql_actix_web::{Request, Response, WSSubscription};
use structopt::StructOpt;
use tracing::{info, trace};

use std::io;

mod args;
mod schema;

async fn sway_dap(schema: web::Data<DebugSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

async fn sway_dap_subscribe(
    schema: web::Data<DebugSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> actix_web::Result<HttpResponse> {
    WSSubscription::start(Schema::clone(&*schema), &req, payload)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let addr = args::Opt::from_args().exec()?;

    trace!("Initializing in TRACE mode");

    let subscription = SubscriptionRoot::default();
    let storage = Storage::default();
    let schema = Schema::build(QueryRoot, MutationRoot, subscription)
        .data(storage)
        .finish();

    let app = move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/sway-dap").guard(guard::Post()).to(sway_dap))
            .service(
                web::resource("/sway-dap/subscribe")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(sway_dap_subscribe),
            )
    };

    trace!("GraphQL primitives initialized");

    info!("Binding GraphQL provider to {}", addr);
    HttpServer::new(app).bind(addr)?.run().await?;

    info!("Graceful shutdown");

    Ok(())
}
