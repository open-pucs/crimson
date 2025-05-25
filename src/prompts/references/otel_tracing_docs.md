# Tracing OpenTelemetry


[`tracing`] is a framework for instrumenting Rust programs to collect structured, event-based diagnostic information. This crate provides a layer that connects spans from multiple systems into a trace and emits them to [OpenTelemetry]-compatible distributed tracing systems for processing and visualization.



[OpenTelemetry]: https://opentelemetry.io

[`tracing`]: https://github.com/tokio-rs/tracing



*Compiler support: [requires `rustc` 1.65+][msrv]*



[msrv]: #supported-rust-versions



### Special Fields



Fields with an `otel.` prefix are reserved for this crate and have specific

meaning. They are treated as ordinary fields by other layers. The current

special fields are:



* `otel.name`: Override the span name sent to OpenTelemetry exporters.

   Setting this field is useful if you want to display non-static information

   in your span name.

* `otel.kind`: Set the span kind to one of the supported OpenTelemetry [span kinds].

* `otel.status_code`: Set the span status code to one of the supported OpenTelemetry [span status codes].

* `otel.status_message`: Set the span status message.



[span kinds]: opentelemetry::trace::SpanKind

[span status codes]: opentelemetry::trace::Status



### Semantic Conventions



OpenTelemetry defines conventional names for attributes of common

operations. These names can be assigned directly as fields, e.g.

`trace_span!("request", "otel.kind" = %SpanKind::Client, "url.full" = ..)`, and they

will be passed through to your configured OpenTelemetry exporter. You can

find the full list of the operations and their expected field names in the

[semantic conventions] spec.



[semantic conventions]: https://github.com/open-telemetry/semantic-conventions



### Stability Status



The OpenTelemetry specification is currently in beta so some breaking

changes may still occur on the path to 1.0. You can follow the changes via

the [spec repository] to track progress toward stabilization.



[spec repository]: https://github.com/open-telemetry/opentelemetry-specification



## Examples



```rs

use opentelemetry_sdk::trace::SdkTracerProvider;

use opentelemetry::trace::{Tracer, TracerProvider as _};

use tracing::{error, span};

use tracing_subscriber::layer::SubscriberExt;

use tracing_subscriber::Registry;



// Create a new OpenTelemetry trace pipeline that prints to stdout

let provider = SdkTracerProvider::builder()

    .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())

    .build();

let tracer = provider.tracer("readme_example");



// Create a tracing layer with the configured tracer

let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);



// Use the tracing subscriber `Registry`, or any other subscriber

// that impls `LookupSpan`

let subscriber = Registry::default().with(telemetry);



// Trace executed code

tracing::subscriber::with_default(subscriber, || {

    // Spans will be sent to the configured OpenTelemetry exporter

    let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);

    let _enter = root.enter();



    error!("This event will be logged in the root span.");

});

```



## Feature Flags



- `metrics`: Enables the [`MetricsLayer`] type, a [layer] that

  exports OpenTelemetry metrics from specifically-named events. This enables

  the `metrics` feature flag on the `opentelemetry` crate.  *Enabled by

  default*.



[layer]: tracing_subscriber::layer



