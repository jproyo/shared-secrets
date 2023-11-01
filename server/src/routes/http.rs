use super::auth::validator;
use super::context::AppContext;
use crate::conf::settings::Settings;
use crate::consensus::handler::ConsensusHandler;
use crate::domain::model::ClientId;
use actix_web::dev::Server;
use actix_web_httpauth::middleware::HttpAuthentication;
use log::info;
use sss_wrap::secret::secret::ShareMeta;

use crate::domain::error::SecretServerError;
use actix_web::{get, post, web, App, HttpServer, Responder, Result};
use std::io;
use std::ops::Deref;

#[post("/{client_id}/secret")]
async fn create_share(
    data: web::Data<AppContext>,
    path: web::Path<ClientId>,
    share: web::Json<ShareMeta>,
) -> Result<impl Responder, SecretServerError> {
    info!(
        "Creating new share from client {:?} with value {:?}",
        path, share
    );
    let client_id = path.into_inner();
    data.consensus_handler()
        .insert(client_id, share.deref().clone())?;
    Ok(web::Json(share))
}

#[get("/{id}/share")]
async fn get_share(
    data: web::Data<AppContext>,
    path: web::Path<ClientId>,
) -> Result<impl Responder, SecretServerError> {
    let id = path.into_inner();
    let result = data.consensus_handler().get(id)?;
    Ok(web::Json(result.map(|share| share.share.clone())))
}

#[get("/leave")]
async fn leave(data: web::Data<AppContext>) -> impl Responder {
    data.consensus_handler().leave().await.unwrap();
    "OK".to_string()
}

pub async fn run(settings: &Settings, consensus_handler: ConsensusHandler) -> io::Result<Server> {
    let api_key = settings.api_key().to_string();
    let web_server = settings.web_server();
    Ok(HttpServer::new(move || {
        let app_context = AppContext::new(consensus_handler.clone(), &api_key);
        let auth_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(web::Data::new(app_context))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(auth_middleware)
            .service(create_share)
            .service(get_share)
            .service(leave)
    })
    .bind(web_server)?
    .run())
}
