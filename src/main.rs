use self::multiplex_service::MultiplexService;
use crate::grpc::setup_grpc;
use crate::rest::setup_rest;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod grpc;
mod multiplex_service;
mod rest;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example-rest-grpc-multiplex=DEBUG".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rest = setup_rest();
    let grpc = setup_grpc();

    let service = MultiplexService::new(rest, grpc);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();
}
