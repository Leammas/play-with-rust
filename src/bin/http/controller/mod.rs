use crate::service::ServiceError;

use super::service::KeyValueService;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{
    http::StatusCode,
    response::Response,
    routing::{get, put},
    Router,
};
use std::sync::Arc;

pub struct Controller {
    service: Arc<KeyValueService>,
}

impl Controller {
    pub fn new(service: Arc<KeyValueService>) -> Self {
        Controller { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/:key",
                get({
                    let service = Arc::clone(&self.service);
                    move |Path(id): Path<String>| {
                        let service = Arc::clone(&service);
                        async move {
                            match service.get_key(id).await {
                                Ok(v) => match v {
                                    Some(value) => (StatusCode::OK, value).into_response(),
                                    None => (StatusCode::NOT_FOUND, "Not Found".to_string()).into_response(),
                                },
                                Err(err) => err.into_response(), 
                            }
                        }
                    }
                }),
            )
            .route(
                "/:key",
                put({
                    let service = Arc::clone(&self.service);
                    move |Path(id): Path<String>, payload: String| {
                        let service = Arc::clone(&service);
                        async move {
                            service.set_key(id, payload).await.map_or_else(
                                |err| err.into_response(),
                                |_| (StatusCode::OK, "Ok".to_string()).into_response(),
                            )
                        }
                    }
                }),
            )
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            // Match on different ServiceErrors if you have specific handling
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            ),
        };
        (status, body).into_response()
    }
}
