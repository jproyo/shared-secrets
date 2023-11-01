use super::auth::validator;
use super::context::AppContext;
use crate::consensus::raft::HashStore;
use crate::domain::model::ClientId;
use actix_web::dev::Server;
use actix_web_httpauth::middleware::HttpAuthentication;
use log::info;
use riteraft::Mailbox;
use sss_wrap::secret::secret::Share;

use crate::domain::error::SecretServerError;
use actix_web::{get, post, web, App, HttpServer, Responder, Result};
use std::io;
use std::ops::Deref;
use std::sync::Arc;

#[post("/{client_id}/secret")]
async fn create_share(
    data: web::Data<AppContext>,
    path: web::Path<ClientId>,
    share: web::Json<Share>,
) -> Result<impl Responder, SecretServerError> {
    info!(
        "Creating new share from client {:?} with value {:?}",
        path, share
    );
    let client_id = path.into_inner();
    data.store().insert(client_id, share.deref().clone())?;
    Ok(web::Json(share))
}

#[get("/{id}/share")]
async fn get_share(
    data: web::Data<AppContext>,
    path: web::Path<ClientId>,
) -> Result<impl Responder, SecretServerError> {
    let id = path.into_inner();
    let result = data.store().get(id)?;
    Ok(web::Json(result))
}

#[get("/leave")]
async fn leave(data: web::Data<AppContext>) -> impl Responder {
    data.mailbox().leave().await.unwrap();
    "OK".to_string()
}

pub async fn run(
    web_server: String,
    mailbox: Arc<Mailbox>,
    store: HashStore,
) -> io::Result<Server> {
    Ok(HttpServer::new(move || {
        let app_context = AppContext::new(mailbox.clone(), store.clone(), "123456");
        let auth_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(web::Data::new(app_context))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(auth_middleware)
            .service(create_share)
            .service(get_share)
            .service(leave)
    })
    .bind(web_server.clone())?
    .run())
}
