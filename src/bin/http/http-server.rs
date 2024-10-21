use std::sync::Arc;

mod controller;
mod service;
mod repository;

// http based KV 
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { run_server().await })
}

async fn run_server() -> () {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let repository = repository::Repository::new().await;

    // since the service has stateful client under the hood which we're fine to share
    // using Atomic reference counter wrapper
    let service = Arc::new(service::KeyValueService::new(repository));

    //@todo add layers
    let app = controller::Controller::new(service).router();

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


