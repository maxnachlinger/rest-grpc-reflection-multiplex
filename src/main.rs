use crate::http_body::EitherBody;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::{routing::get, Router};
use futures::TryFutureExt;
use proto::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Response as TonicResponse, Status};
use tower::make::Shared;
use tower::util::Either;
use tower::{service_fn, Service};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod http_body;

mod proto {
    tonic::include_proto!("helloworld");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("helloworld_descriptor");
}

#[derive(Default)]
struct GrpcServiceImpl {}

#[tonic::async_trait]
impl Greeter for GrpcServiceImpl {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<TonicResponse<HelloReply>, Status> {
        tracing::info!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(TonicResponse::new(reply))
    }
}

async fn web_root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum-tonic-multiplex=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut rest = Router::new().route("/", get(web_root));

    let greeter_service = GreeterServer::new(GrpcServiceImpl::default());

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let mut grpc = Server::builder()
        .add_service(greeter_service)
        .add_service(reflection_service)
        .into_service();

    let service = Shared::new(service_fn(move |req| {
        if is_grpc_request(&req) {
            return Either::A(grpc.call(req).map_ok(|res| res.map(EitherBody::A)));
        }
        Either::B(rest.call(req).map_ok(|res| res.map(EitherBody::B)))
    }));

    let socket_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", socket_addr);

    let server = axum::Server::bind(&socket_addr).serve(service);

    server.await.unwrap();

    tracing::info!("Server shutdown");
}

fn is_grpc_request<B>(req: &Request<B>) -> bool {
    req.headers()
        .get(CONTENT_TYPE)
        .map(|content_type| content_type.as_bytes())
        .filter(|content_type| content_type.starts_with(b"application/grpc"))
        .is_some()
}
