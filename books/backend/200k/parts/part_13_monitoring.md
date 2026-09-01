# Part 13: Monitoring and Observability

---

# Chapter 65: Structured Logging with Tracing

Structured logging creates machine-readable events that can be queried and analyzed.

## Why Structured Logging?

Traditional logging produces unstructured text:

```
2024-01-15 10:30:45 INFO Request received for story 123456
2024-01-15 10:30:46 INFO Cache hit for story 123456
2024-01-15 10:30:46 INFO Response sent in 1.2 seconds
```

Structured logging produces structured events:

```json
{
  "timestamp": "2024-01-15T10:30:45Z",
  "level": "INFO",
  "target": "fichub::routes::export",
  "message": "Request received",
  "url_id": "123456",
  "ip": "192.168.1.1"
}
```

## Tracing Setup

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,fichub=debug".into()))
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true))
        .init();
}
```

## Log Levels

```rust
use tracing::{trace, debug, info, warn, error};

async fn process_request(url: &str) -> Result<(), AppError> {
    trace!(url = %url, "Entering process_request");
    
    debug!("Validating URL");
    validate_url(url)?;
    
    info!(url = %url, "Fetching story metadata");
    let meta = fetch_metadata(url).await?;
    
    if meta.words > 100_000 {
        warn!(url_id = %meta.url_id, words = meta.words, "Large story detected");
    }
    
    match export_story(&meta).await {
        Ok(hash) => {
            info!(url_id = %meta.url_id, hash = %hash, "Export complete");
            Ok(())
        }
        Err(e) => {
            error!(url_id = %meta.url_id, error = %e, "Export failed");
            Err(e)
        }
    }
}
```

## Spans

Spans create a context for related events:

```rust
use tracing::{info_span, Instrument};

async fn process_request(url: &str) -> Result<(), AppError> {
    let span = info_span!("request", url = %url);
    let _guard = span.enter();
    
    // All events within this scope are associated with this span
    info!("Processing started");
    
    // For async code, use .instrument()
    let result = async {
        info!("Fetching metadata");
        let meta = fetch_metadata(url).await?;
        info!("Generating export");
        export_story(&meta).await
    }.instrument(span).await;
    
    result
}
```

## JSON Logging for Production

```rust
use tracing_subscriber::fmt::format::FmtSpan;

tracing_subscriber::fmt()
    .json()
    .with_max_level(tracing::Level::INFO)
    .with_current_span(true)
    .with_span_list(true)
    .with_target(false)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

## 📝 Practice Exercises

1. **Structured Logging:** Add structured logging to 5 different functions in FicHub.

2. **Span Creation:** Create spans for the complete export flow and verify they nest correctly.

3. **Log Analysis:** Write a script that parses JSON logs and extracts performance metrics.

---

# Chapter 66: Metrics Collection with Prometheus

Prometheus is a metrics collection system. This chapter covers how to instrument FicHub with Prometheus metrics.

## Setting Up Prometheus Metrics

```rust
use prometheus::{
    Encoder, IntCounter, IntGauge, Registry, TextEncoder,
    opts, register_int_counter_with_registry, register_int_gauge_with_registry,
};

lazy_static::lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: IntCounter = register_int_counter_with_registry!(
        opts!("fichub_http_requests_total", "Total HTTP requests"),
        registry
    ).unwrap();
    
    static ref HTTP_REQUEST_DURATION: Histogram = register_histogram_with_registry!(
        opts!("fichub_http_request_duration_seconds", "HTTP request duration"),
        registry
    ).unwrap();
    
    static ref EXPORTS_TOTAL: IntCounter = register_int_counter_with_registry!(
        opts!("fichub_exports_total", "Total exports"),
        registry
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge_with_registry!(
        opts!("fichub_active_connections", "Active database connections"),
        registry
    ).unwrap();
}
```

## Instrumenting Handlers

```rust
async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let timer = HTTP_REQUEST_DURATION.start_timer();
    
    // Process request
    let result = process_export(&state, &query).await;
    
    timer.observe_duration();
    HTTP_REQUESTS_TOTAL.inc();
    
    result
}
```

## Metrics Endpoint

```rust
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    (
        [("Content-Type", encoder.format_type())],
        buffer,
    )
}
```

## Key Metrics to Track

### Request Metrics

- `fichub_http_requests_total` — Total requests by endpoint and status
- `fichub_http_request_duration_seconds` — Request latency histogram
- `fichub_http_requests_in_flight` — Currently processing requests

### Business Metrics

- `fichub_exports_total` — Total exports by format
- `fichub_export_duration_seconds` — Export generation time
- `fichub_cache_hits_total` — Cache hit count
- `fichub_cache_misses_total` — Cache miss count

### System Metrics

- `fichub_database_connections_active` — Active database connections
- `fichub_redis_connections_active` — Active Redis connections
- `fichub_memory_usage_bytes` — Process memory usage

## 📝 Practice Exercises

1. **Metrics Setup:** Add Prometheus metrics to FicHub's main endpoints.

2. **Grafana Dashboard:** Create a Grafana dashboard with key FicHub metrics.

3. **Alerting Rules:** Define Prometheus alerting rules for error rate and latency.

---

# Chapter 67: Distributed Tracing

Distributed tracing tracks requests across multiple services.

## OpenTelemetry Setup

```rust
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry_sdk::{runtime, trace::TracerProvider};

fn init_tracing() {
    let provider = TracerProvider::builder()
        .with_batch_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .install_batch(runtime::Tokio)
                .unwrap(),
        )
        .build();
    
    global::set_tracer_provider(provider);
}
```

## Trace Context Propagation

```rust
use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer};
use opentelemetry::Context;

async fn process_request(url: &str) -> Result<(), AppError> {
    let tracer = global::tracer("fichub");
    
    let mut span = tracer
        .span_builder("process_request")
        .with_kind(SpanKind::Server)
        .start(&tracer);
    
    span.set_attribute("url", url.to_string());
    
    let cx = Context::current_with_span(span);
    
    // Process with context
    let result = async {
        fetch_and_export(url).await
    }.instrument(cx.span())
     .await;
    
    result
}
```

## Trace Visualization

Traces can be visualized in:
- **Jaeger** — Open-source distributed tracing
- **Zipkin** — Distributed tracing system
- **Grafana Tempo** — High-scale distributed tracing

## 📝 Practice Exercises

1. **OpenTelemetry Setup:** Set up OpenTelemetry in FicHub and export traces to Jaeger.

2. **Trace Context:** Propagate trace context across async tasks.

3. **Span Attributes:** Add meaningful attributes to spans for debugging.

---

# Chapter 68: Health Checks and Readiness Probes

Health checks verify that your application is running correctly.

## Health Check Endpoint

```rust
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let mut checks = HashMap::new();
    
    // Database check
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();
    checks.insert("database", db_ok);
    
    // Redis check
    let redis_ok = redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
        .is_ok();
    checks.insert("redis", redis_ok);
    
    let all_healthy = checks.values().all(|&v| v);
    
    Ok(Json(json!({
        "status": if all_healthy { "healthy" } else { "unhealthy" },
        "checks": checks
    })))
}
```

## Readiness Probe

```rust
async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Check database connectivity
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|_| AppError::Internal("database not ready".into()))?;
    
    // Check Redis connectivity
    redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
        .map_err(|_| AppError::Internal("redis not ready".into()))?;
    
    Ok(StatusCode::OK)
}
```

## Kubernetes Health Checks

```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: fichub
    livenessProbe:
      httpGet:
        path: /health
        port: 3000
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: 3000
      initialDelaySeconds: 5
      periodSeconds: 5
```

## 📝 Practice Exercises

1. **Health Check:** Implement health checks for all external dependencies.

2. **Readiness Probe:** Add a readiness probe that checks database migrations are up to date.

3. **Startup Probe:** Add a startup probe for slow-starting containers.

---

# Chapter 69: Alerting and On-Call

Alerting notifies you when something goes wrong.

## Prometheus Alerting Rules

```yaml
groups:
- name: fichub
  rules:
  - alert: HighErrorRate
    expr: rate(fichub_http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      description: "Error rate is {{ $value }} per second"
  
  - alert: HighLatency
    expr: histogram_quantile(0.95, rate(fichub_http_request_duration_seconds_bucket[5m])) > 2
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High latency detected"
      description: "95th percentile latency is {{ $value }} seconds"
  
  - alert: DatabaseConnectionPoolExhausted
    expr: fichub_database_connections_active / fichub_database_connections_max > 0.9
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Database connection pool nearly exhausted"
```

## Alertmanager Configuration

```yaml
route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
- name: 'web.hook'
  webhook_configs:
  - url: 'http://localhost:5001/'
```

## On-Call Rotation

- **PagerDuty** — Commercial on-call management
- **Opsgenie** — Atlassian's incident management
- **Grafana OnCall** — Open-source on-call management

## Incident Response

1. **Detect** — Alert fires
2. **Acknowledge** — On-call engineer acknowledges
3. **Investigate** — Check dashboards and logs
4. **Mitigate** — Fix the immediate issue
5. **Resolve** — Root cause is fixed
6. **Review** — Post-incident review

## 📝 Practice Exercises

1. **Alert Rules:** Define alert rules for 5 different failure scenarios.

2. **Dashboard:** Create a Grafana dashboard that shows all key metrics.

3. **Incident Runbook:** Write an incident response runbook for database connection pool exhaustion.

