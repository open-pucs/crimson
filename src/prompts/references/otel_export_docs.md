
# OpenTelemetry OTLP Exporter



The OTLP Exporter enables exporting telemetry data (logs, metrics, and traces) in the

OpenTelemetry Protocol (OTLP) format to compatible backends. These backends include:



- OpenTelemetry Collector

- Open-source observability tools (Prometheus, Jaeger, etc.)

- Vendor-specific monitoring platforms



This crate supports sending OTLP data via:

- gRPC

- HTTP (binary protobuf or JSON)



## Quickstart with OpenTelemetry Collector



### HTTP Transport (Port 4318)



Run the OpenTelemetry Collector:



```shell

$ docker run -p 4318:4318 otel/opentelemetry-collector:latest

```



Configure your application to export traces via HTTP:



```rs

# #[cfg(all(feature = "trace", feature = "http-proto"))]

# {

use opentelemetry::global;

use opentelemetry::trace::Tracer;

use opentelemetry_otlp::Protocol;

use opentelemetry_otlp::WithExportConfig;



fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {

    // Initialize OTLP exporter using HTTP binary protocol

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()

        .with_http()

        .with_protocol(Protocol::HttpBinary)

        .build()?;



    // Create a tracer provider with the exporter

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()

        .with_simple_exporter(otlp_exporter)

        .build();



    // Set it as the global provider

    global::set_tracer_provider(tracer_provider);



    // Get a tracer and create spans

    let tracer = global::tracer("my_tracer");

    tracer.in_span("doing_work", |_cx| {

        // Your application logic here...

    });



    Ok(())

# }

}

```



### gRPC Transport (Port 4317)



Run the OpenTelemetry Collector:



```shell

$ docker run -p 4317:4317 otel/opentelemetry-collector:latest

```



Configure your application to export traces via gRPC (the tonic client requires a Tokio runtime):



- With `[tokio::main]`



```rs

# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]

# {

use opentelemetry::{global, trace::Tracer};



#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {

    // Initialize OTLP exporter using gRPC (Tonic)

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()

        .with_tonic()

        .build()?;



    // Create a tracer provider with the exporter

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()

        .with_simple_exporter(otlp_exporter)

        .build();



    // Set it as the global provider

    global::set_tracer_provider(tracer_provider);



    // Get a tracer and create spans

    let tracer = global::tracer("my_tracer");

    tracer.in_span("doing_work", |_cx| {

        // Your application logic here...

    });



    Ok(())

# }

}

```



- Without `[tokio::main]`



 ```rs

# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]

# {

use opentelemetry::{global, trace::Tracer};



fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {

    // Initialize OTLP exporter using gRPC (Tonic)

    let rt = tokio::runtime::Runtime::new()?;

    let tracer_provider = rt.block_on(async {

        let exporter = opentelemetry_otlp::SpanExporter::builder()

            .with_tonic()

            .build()

            .expect("Failed to create span exporter");

        opentelemetry_sdk::trace::SdkTracerProvider::builder()

            .with_simple_exporter(exporter)

            .build()

    });



    // Set it as the global provider

    global::set_tracer_provider(tracer_provider);



    // Get a tracer and create spans

    let tracer = global::tracer("my_tracer");

    tracer.in_span("doing_work", |_cx| {

        // Your application logic here...

    });



    // Ensure the runtime (`rt`) remains active until the program ends

    Ok(())

# }

}
