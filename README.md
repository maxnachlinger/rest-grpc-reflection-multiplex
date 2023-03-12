Fork of
the [rest-grpc-multiplex example](https://github.com/tokio-rs/axum/tree/main/examples/rest-grpc-multiplex)
outlining an issue I encountered using `tonic-reflection`.

```shell
   Compiling axum-tonic-multiplex v0.1.0 (/mypathhere/axum-tonic-multiplex)
error[E0271]: type mismatch resolving `<Routes as Service<hyper::Request<hyper::Body>>>::Error == Infallible`
  --> src/main.rs:79:10
   |
79 |         .serve(tower::make::Shared::new(service))
   |          ^^^^^ expected struct `Box`, found enum `Infallible`
   |
   = note: expected struct `Box<(dyn std::error::Error + std::marker::Send + Sync + 'static)>`
                found enum `Infallible`
note: required for `MultiplexService<axum::Router, Routes>` to implement `Service<hyper::Request<hyper::Body>>`
  --> src/multiplex_service.rs:44:12
   |
44 | impl<A, B> Service<Request<Body>> for MultiplexService<A, B>
   |            ^^^^^^^^^^^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^^^^
   = note: required for `MultiplexService<axum::Router, Routes>` to implement `hyper::service::http::HttpService<hyper::Body>`
   = note: required for `tower::make::Shared<MultiplexService<axum::Router, Routes>>` to implement `hyper::service::make::MakeServiceRef<AddrStream, hyper::Body>`

error[E0277]: the trait bound `hyper::common::exec::Exec: hyper::common::exec::NewSvcExec<AddrStream, SharedFuture<MultiplexService<axum::Router, Routes>>, MultiplexService<axum::Router, Routes>, hyper::common::exec::Exec, hyper::server::server::NoopWatcher>` is not satisfied
  --> src/main.rs:80:9
   |
80 |         .await
   |         ^^^^^^
   |         |
   |         the trait `hyper::common::exec::NewSvcExec<AddrStream, SharedFuture<MultiplexService<axum::Router, Routes>>, MultiplexService<axum::Router, Routes>, hyper::common::exec::Exec, hyper::server::server::NoopWatcher>` is not implemented for `hyper::common::exec::Exec`
   |         help: remove the `.await`
   |
   = help: the trait `hyper::common::exec::NewSvcExec<I, N, S, E, W>` is implemented for `hyper::common::exec::Exec`
   = note: required for `axum::Server<AddrIncoming, tower::make::Shared<MultiplexService<axum::Router, Routes>>>` to implement `futures::Future`
   = note: required for `axum::Server<AddrIncoming, tower::make::Shared<MultiplexService<axum::Router, Routes>>>` to implement `std::future::IntoFuture`

Some errors have detailed explanations: E0271, E0277.
For more information about an error, try `rustc --explain E0271`.
error: could not compile `axum-tonic-multiplex` due to 2 previous errors
```
