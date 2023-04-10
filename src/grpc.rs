use proto::{
    echo_server::{Echo, EchoServer},
    EchoReply, EchoRequest,
};
use tonic::transport::{server::Routes, Server};
use tonic::{Request, Response, Status};
use tower_http::classify::{GrpcErrorsAsFailures, SharedClassifier};
use tower_http::trace::{Trace, TraceLayer};

mod proto {
    tonic::include_proto!("echo");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("echo_descriptor");
}

#[derive(Default)]
struct GrpcServiceImpl {}

#[tonic::async_trait]
impl Echo for GrpcServiceImpl {
    async fn echo(&self, request: Request<EchoRequest>) -> Result<Response<EchoReply>, Status> {
        let reply = EchoReply {
            message: request.into_inner().message,
        };
        Ok(Response::new(reply))
    }

    async fn not_found(
        &self,
        _request: Request<EchoRequest>,
    ) -> Result<Response<EchoReply>, Status> {
        Err(Status::not_found("Not found"))
    }

    async fn internal_error(
        &self,
        _request: Request<EchoRequest>,
    ) -> Result<Response<EchoReply>, Status> {
        Err(Status::internal("Internal Error"))
    }
}

pub fn setup_grpc() -> Trace<Routes, SharedClassifier<GrpcErrorsAsFailures>> {
    let greeter_service = EchoServer::new(GrpcServiceImpl::default());

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
