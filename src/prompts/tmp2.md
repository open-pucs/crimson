I am running into an error with my rust project when it comes to emmitting proper otel traces here is the console log:

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running `target/debug/crimson`
Started Program
Exporting on:http://localhost:4317
Created otel exporter
Created otel provider
Created Tracer
2025-05-27T13:56:51.545894Z  INFO crimson: Tracing Subscriber is up and running, trying to c
2025-05-27T13:56:51.546517Z DEBUG get: opentelemetry-otlp:  name="TonicsTracesClient.Calling
2025-05-27T13:56:51.546600Z TRACE get: tower::buffer::service: sending request to buffer wor
2025-05-27T13:56:51.546679Z TRACE tower::buffer::worker: worker polling for next message
2025-05-27T13:56:51.546716Z TRACE tower::buffer::worker: processing new request
2025-05-27T13:56:51.546774Z TRACE get: tower::buffer::worker: resumed=false worker received
2025-05-27T13:56:51.546811Z TRACE get: tonic::transport::channel::service::reconnect: poll_r
2025-05-27T13:56:51.546840Z TRACE get: tonic::transport::channel::service::reconnect: poll_r
2025-05-27T13:56:51.546895Z TRACE get: hyper_util::client::legacy::connect::http: Http::conn
2025-05-27T13:56:51.547010Z TRACE get: tonic::transport::channel::service::reconnect: poll_r
2025-05-27T13:56:51.547029Z TRACE get: tower::buffer::worker: service.ready=false delay
2025-05-27T13:56:51.547257Z TRACE tower::buffer::worker: worker polling for next message
2025-05-27T13:56:51.547266Z TRACE tower::buffer::worker: resuming buffered request
2025-05-27T13:56:51.547279Z TRACE get: tower::buffer::worker: resumed=true worker received r
2025-05-27T13:56:51.547300Z TRACE get: tonic::transport::channel::service::reconnect: poll_r
2025-05-27T13:56:51.547357Z DEBUG get: hyper_util::client::legacy::connect::http: connecting
2025-05-27T13:56:51.547543Z TRACE get: tonic::transport::channel::service::reconnect: poll_r
2025-05-27T13:56:51.547561Z TRACE get: tower::buffer::worker: service.ready=false delay
2025-05-27T13:56:51.547598Z TRACE tower::buffer::worker: worker polling for next message
2025-05-27T13:56:51.547608Z TRACE tower::buffer::worker: resuming buffered request
2025-05-27T13:56:51.547621Z TRACE get: tower::buffer::worker: resumed=true worker received r
2025-05-27T13:56:51.547641Z TRACE get: tonic::transport::channel::service::reconnect: poll_r
2025-05-27T13:56:51.547708Z TRACE get: hyper_util::client::legacy::connect::http: connect error for [::1]:4317: ConnectError("tcp connect error", Os { code: 111, kind: ConnectionRefused, message: "Connection refused" })
2025-05-27T13:56:51.547726Z DEBUG get: hyper_util::client::legacy::connect::http: connecting to 127.0.0.1:4317
2025-05-27T13:56:51.547875Z TRACE get: tonic::transport::channel::service::reconnect: poll_ready; not ready
2025-05-27T13:56:51.547895Z TRACE get: tower::buffer::worker: service.ready=false delay
2025-05-27T13:56:51.547927Z TRACE tower::buffer::worker: worker polling for next message
2025-05-27T13:56:51.547936Z TRACE tower::buffer::worker: resuming buffered request
2025-05-27T13:56:51.547951Z TRACE get: tower::buffer::worker: resumed=true worker received request; waiting for service readiness
2025-05-27T13:56:51.547971Z TRACE get: tonic::transport::channel::service::reconnect: poll_ready; connecting
2025-05-27T13:56:51.547996Z DEBUG get: hyper_util::client::legacy::connect::http: connected to 127.0.0.1:4317
2025-05-27T13:56:51.548070Z DEBUG get: h2::client: binding client connection
2025-05-27T13:56:51.548143Z DEBUG get: h2::client: client connection bound
2025-05-27T13:56:51.548308Z DEBUG get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::codec::framed_write: send frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }
2025-05-27T13:56:51.548344Z TRACE get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::frame::settings: encoding SETTINGS; len=24
2025-05-27T13:56:51.548373Z TRACE get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::frame::settings: encoding setting; val=EnablePush(0)
2025-05-27T13:56:51.548398Z TRACE get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::frame::settings: encoding setting; val=InitialWindowSize(2097152)
2025-05-27T13:56:51.548419Z TRACE get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::frame::settings: encoding setting; val=MaxFrameSize(16384)
2025-05-27T13:56:51.548440Z TRACE get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::frame::settings: encoding setting; val=MaxHeaderListSize(16384)
2025-05-27T13:56:51.548465Z TRACE get:FramedWrite::buffer{frame=Settings { flags: (0x0), enable_push: 0, initial_window_size: 2097152, max_frame_size: 16384, max_header_list_size: 16384 }}: h2::codec::framed_write: encoded settings rem=33
```
and here it just stops and never finishes the next step: here is the offending code in
/home/nicole/Documents/mycorrhizae/crimson/src/main.rs

```rs

    let _otel_subscriber = Registry::default()
        .with(tracing_opentelemetry::layer().with_tracer(otel_tracer))
        .with(tracing_subscriber::fmt::layer());

    let _stdout_subscriber = Registry::default()
        .with(tracing_opentelemetry::layer().with_tracer(stdout_tracer))
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(_otel_subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Tracing Subscriber is up and running, trying to create app");
    // initialise our subscriber
    let app = ApiRouter::new()
        // Add HTTP tracing layer
        .layer(OtelAxumLayer::default())
        .api_route("/v1/health", get(health))
        .route("/api.json", get(serve_api))
        .route("/swagger", Swagger::new("/api.json").axum_route())
        .nest("/v1/", api::router())
        .nest("/admin/", admin::router());

    // Spawn background worker to process PDF tasks
    // This worker runs indefinitely
    info!("App Created, spawning background process:");
    tokio::spawn(async move {
        processing::worker::start_worker().await;
    });

    // bind and serve
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8080);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);
```

However when I swap out the otel subscriber for the stdout subscriber everything works as expected. Do you know what could be going wrong here?


I think the best thing to 


Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes. Also try to avoid modifying any code that isnt absolutely essential to implement your feature.
