use std::env;

use actix_web::{
    Error, HttpResponse,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};
use base64::{Engine, engine::general_purpose::STANDARD};

// protect the website using basic http authentication
//
pub async fn basic_auth(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // only in effect if the env var is set
    if let Ok(password) = env::var("MASTER_PASSWORD") {
        if req // turn on inline type hints or this is unfollowable
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Basic "))
            .and_then(|h| STANDARD.decode(h.as_bytes()).ok())
            .and_then(|h| str::from_utf8(&h).map(str::to_owned).ok())
            .is_some_and(|h| h == format!("admin:{password}"))
        {
            Ok(next.call(req).await?.map_into_boxed_body())
        } else {
            // send the WWW-Authenticate header if the Authorization header doesn't match up
            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .insert_header(("WWW-Authenticate", r#"Basic realm="kissasome""#))
                    .finish(),
            ))
        }
    } else {
        Ok(next.call(req).await?.map_into_boxed_body())
    }
}
