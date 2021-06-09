use crate::schema::{self, DebugSchema};

use actix_web::{guard, web, HttpRequest, HttpResponse};
use async_graphql::Schema;
use async_graphql_actix_web::{Request, Response, WSSubscription};

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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.data(schema::debug_schema())
        .service(web::resource("/sway-dap").guard(guard::Post()).to(sway_dap))
        .service(
            web::resource("/sway-dap/subscribe")
                .guard(guard::Get())
                .guard(guard::Header("upgrade", "websocket"))
                .to(sway_dap_subscribe),
        );
}
