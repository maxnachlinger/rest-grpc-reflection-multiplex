use proto::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use tonic::transport::{server::Routes, Server};
use tonic::{Response, Status};
use tower_http::classify::{GrpcErrorsAsFailures, SharedClassifier};
use tower_http::trace::{Trace, TraceLayer};

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
    ) -> Result<Response<HelloReply>, Status> {
        tracing::info!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

pub fn setup_grpc() -> Trace<Routes, SharedClassifier<GrpcErrorsAsFailures>> {
    let greeter_service = GreeterServer::new(GrpcServiceImpl::default());

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .layer(TraceLayer::new_for_grpc())
        .add_service(reflection_service)
        .add_service(greeter_service)
        .into_service()
}

// TODO - make a classifier - https://docs.rs/tower-http/latest/tower_http/trace/index.html
