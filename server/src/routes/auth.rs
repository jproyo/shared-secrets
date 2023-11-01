use actix_web::dev::ServiceRequest;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use super::context::AppContext;

/// Validator function for validating bearer token.
///
/// This function is used as a middleware to validate the bearer token
/// provided in the request. It checks if the token is valid by calling
/// the `validate_key` method on the `AppContext` configuration.
///
/// # Arguments
///
/// * `req` - The `ServiceRequest` object representing the incoming request.
/// * `credentials` - The `BearerAuth` object representing the bearer token.
///
/// # Returns
///
/// Returns a `Result` containing `ServiceRequest` on success, or an `Error`
/// along with the `ServiceRequest` on failure. The error can be either an
/// unauthorized error if the token is invalid, or an internal server error
/// if the application configuration is not found.
///
/// # Examples
///
/// ```
/// use actix_web::dev::ServiceRequest;
/// use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
/// use actix_web::{web, Error};
/// use actix_web_httpauth::extractors::bearer::BearerAuth;
///
/// use super::context::AppContext;
///
/// async fn validator(
///     req: ServiceRequest,
///     credentials: BearerAuth,
/// ) -> Result<ServiceRequest, (Error, ServiceRequest)> {
///     let config = req.app_data::<web::Data<AppContext>>();
///     match config {
///         Some(config) => {
///             if config.validate_key(credentials.token()) {
///                 Ok(req)
///             } else {
///                 Err((ErrorUnauthorized("Unauthorized"), req))
///             }
///         }
///         None => Err((ErrorInternalServerError("Internal Error"), req)),
///     }
/// }
/// ```
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
