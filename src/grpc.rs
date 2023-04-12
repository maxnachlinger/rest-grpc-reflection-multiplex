use proto::{
    echo_server::{Echo, EchoServer},
    EchoReply, EchoRequest,
};
use std::time::Duration;
use tonic::transport::{server::Routes, Server};
use tonic::{Code, Request, Response, Status};
use tower_http::classify::{GrpcErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultOnRequest, MakeSpan, OnResponse, Trace, TraceLayer};
use tracing::Span;

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

#[derive(Debug, Clone)]
pub struct MakeGrpcSpan;

impl MakeGrpcSpan {
    pub fn new() -> Self {
        Self {}
    }
}

impl<B> MakeSpan<B> for MakeGrpcSpan {
    fn make_span(&mut self, request: &hyper::Request<B>) -> Span {
        // TODO - get otel headers, create otel span
        tracing::info_span!(
            "request",
            method = %request.method(),
            path = %request.uri().path(),
            version = ?request.version(),
            headers = ?request.headers(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct OnGrpcResponse;

impl OnGrpcResponse {
    pub fn new() -> Self {
        Self {}
    }
}

impl<B> OnResponse<B> for OnGrpcResponse {
    fn on_response(self, response: &hyper::Response<B>, latency: Duration, _span: &Span) {
        let latency = latency.as_millis();

        let code = Status::from_header_map(response.headers())
            .unwrap_or_else(|| Status::ok("ok"))
            .code();

        // bump time histogram
        match code {
            Code::Ok => {
                println!("ok! {latency}");
            }
            // TODO - expand this set
            Code::NotFound | Code::InvalidArgument => {
                println!("4xx {latency}");
            }
            _ => {
                println!("5xx {latency}");
            }
        }
    }
}

pub fn setup_grpc() -> Trace<
    Routes,
    SharedClassifier<GrpcErrorsAsFailures>,
    MakeGrpcSpan,
    DefaultOnRequest,
    OnGrpcResponse,
    (),
    (),
    (),
> {
    let greeter_service = EchoServer::new(GrpcServiceImpl::default());

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let tracing_layer = TraceLayer::new_for_grpc()
        .make_span_with(MakeGrpcSpan::new())
        .on_response(OnGrpcResponse::new())
        .on_body_chunk(())
        .on_eos(())
        .on_failure(());

    Server::builder()
        .layer(tracing_layer)
        .add_service(reflection_service)
        .add_service(greeter_service)
        .into_service()
}
