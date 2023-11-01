use actix_web::dev::ServiceRequest;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use super::context::AppContext;

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req.app_data::<web::Data<AppContext>>();
    match config {
        Some(config) => {
            if config.validate_key(credentials.token()) {
                Ok(req)
            } else {
                Err((ErrorUnauthorized("Unauthorized"), req))
            }
        }
        None => Err((ErrorInternalServerError("Internal Error"), req)),
    }
}
