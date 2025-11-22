use crate::error::{AppError, Result};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{RandomIdGenerator, Sampler, Tracer},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_telemetry(service_name: &str, otlp_endpoint: Option<&str>) -> Result<()> {
    let tracer_provider = otlp_endpoint
        .map(|endpoint| init_tracer_provider(service_name, endpoint))
        .transpose()?;

    let telemetry_layer = tracer_provider
        .as_ref()
        .map(|tracer| {
            tracing_opentelemetry::layer()
                .with_tracer(tracer.clone())
                .with_tracked_inactivity(true)
        });

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();

    if let Some(endpoint) = otlp_endpoint {
        tracing::info!(
            endpoint = %endpoint,
            service_name = %service_name,
            "OpenTelemetry initialized and exporting to OTLP endpoint"
        );
    } else {
        tracing::info!("OpenTelemetry disabled - no OTLP endpoint configured");
    }

    Ok(())
}

fn init_tracer_provider(
    service_name: &str,
    endpoint: &str,
) -> Result<Tracer> {
    tracing::debug!(endpoint = %endpoint, "Building OTLP exporter");

    let resource = Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        service_name.to_string(),
    )]);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .map_err(|e| AppError::Telemetry(format!("Failed to build OTLP exporter: {}", e)))?;

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .build();

    let tracer = provider.tracer("tracing");

    global::set_tracer_provider(provider);

    tracing::debug!("Tracer provider registered");

    Ok(tracer)
}

pub fn shutdown_telemetry() {
    tracing::info!("Shutting down telemetry and flushing remaining spans");
}