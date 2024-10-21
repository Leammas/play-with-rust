use std::sync::Arc;
use tokio_postgres::{NoTls};
mod controller;
mod service;

// http based KV 
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { run_server().await })
}

async fn run_server() -> () {
    // Connect to the database.
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=mysecretpassword port=5432",
        NoTls,
    )
    .await
    .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .batch_execute(
            "
CREATE TABLE kv (
    key TEXT PRIMARY KEY,
    value TEXT
);
",
        )
        .await
        .ok();

    // initialize tracing
    tracing_subscriber::fmt::init();

    // since the service has stateful client under the hood which we're fine to share
    // using Atomic reference counter wrapper
    let service = Arc::new(service::KeyValueService::new(client));

    //@todo add layers
    let app = controller::Controller::new(service).router();

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


