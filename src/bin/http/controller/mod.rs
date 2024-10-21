use super::service::KeyValueService;
use axum::extract::Path;
use axum::{
    http::StatusCode,
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
                                    Some(v) => (StatusCode::OK, v),
                                    None => (StatusCode::NOT_FOUND, "Not Found".to_string()),
                                },
                                Err(_) => (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "Internal error".to_string(),
                                ),
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
                            match service.set_key(id, payload).await {
                                Ok(_) => (StatusCode::OK, "Ok".to_string()),
                                Err(_) => (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "Internal error".to_string(),
                                ),
                            }
                        }
                    }
                }),
            )
    }
}
