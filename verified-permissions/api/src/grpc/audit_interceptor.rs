//! gRPC Audit Interceptor - Automatically captures all API calls
//!
//! This module implements a gRPC interceptor that automatically captures
//! all API calls and publishes audit events following AWS CloudTrail patterns.

use hodei_domain::events::{ApiCalled, ApiCompleted, DomainEvent, EventDispatcher};
use std::sync::Arc;
use std::time::Instant;
use tonic::{Request, Response, Status};
use tower::{Layer, Service};

/// Audit Interceptor Layer
/// Automatically captures all gRPC API calls and publishes audit events
#[derive(Clone)]
pub struct AuditInterceptorLayer<
    D: EventDispatcher<hodei_domain::events::InMemoryEventBus, hodei_domain::events::SqliteEventStore>,
> {
    dispatcher: Arc<D>,
    service_name: String,
}

impl<
    D: EventDispatcher<
            hodei_domain::events::InMemoryEventBus,
            hodei_domain::events::SqliteEventStore,
        > + Send
        + Sync
        + 'static,
> AuditInterceptorLayer<D>
{
    pub fn new(dispatcher: Arc<D>, service_name: String) -> Self {
        Self {
            dispatcher,
            service_name,
        }
    }
}

impl<S, D> Layer<S> for AuditInterceptorLayer<D>
where
    D: EventDispatcher<
            hodei_domain::events::InMemoryEventBus,
            hodei_domain::events::SqliteEventStore,
        > + Send
        + Sync
        + 'static,
    S: Service<tonic::transport::Route, Response = ()> + Send + Sync + Clone + 'static,
{
    type Service = AuditInterceptorService<S, D>;

    fn layer(&self, inner: S) -> Self::Service {
        AuditInterceptorService {
            inner,
            dispatcher: Arc::clone(&self.dispatcher),
            service_name: self.service_name.clone(),
        }
    }
}

/// Audit Interceptor Service
/// Intercepts gRPC calls to publish audit events
#[derive(Clone)]
pub struct AuditInterceptorService<S, D>
where
    D: EventDispatcher<
            hodei_domain::events::InMemoryEventBus,
            hodei_domain::events::SqliteEventStore,
        > + Send
        + Sync
        + 'static,
{
    inner: S,
    dispatcher: Arc<D>,
    service_name: String,
}

impl<S, D, B> Service<Request<B>> for AuditInterceptorService<S, D>
where
    D: EventDispatcher<
            hodei_domain::events::InMemoryEventBus,
            hodei_domain::events::SqliteEventStore,
        > + Send
        + Sync
        + 'static,
    S: Service<Request<B>, Response = Response<B>, Error = Status> + Send + Sync + 'static,
    B: Send + 'static,
{
    type Response = Response<B>;
    type Error = Status;
    type Future = impl std::future::Future<Output = Result<Self::Response, Self::Error>> + Send;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let start_time = Instant::now();
        let dispatcher = Arc::clone(&self.dispatcher);
        let service_name = self.service_name.clone();

        // Extract request metadata
        let method_name = request
            .metadata()
            .get("method")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string().into());
        let client_ip = extract_client_ip(&request);
        let user_agent = request
            .metadata()
            .get("user-agent")
            .cloned()
            .map(|s| s.to_str().unwrap_or("unknown").to_string());
        let request_id = generate_request_id();

        // Publish ApiCalled event
        let called_event = ApiCalled {
            event_id: uuid::Uuid::new_v4().to_string(),
            service_name,
            method_name: method_name.to_str().unwrap_or("unknown").to_string(),
            client_ip,
            user_agent,
            request_id: request_id.clone(),
            request_size_bytes: estimate_request_size(&request),
            occurred_at: chrono::Utc::now(),
            version: 1,
        };

        // Spawn async task to publish event (don't wait for it)
        let dispatch_future = async move {
            let _ = dispatcher
                .dispatch(
                    &called_event.aggregate_id(),
                    vec![Box::new(called_event) as Box<dyn DomainEvent>],
                    0,
                )
                .await;
        };
        tokio::spawn(dispatch_future);

        let future = self.inner.call(request);

        async move {
            let result = future.await;

            // Publish ApiCompleted event
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let (status_code, error_message, response_size) = match &result {
                Ok(_) => (0, None, 0),
                Err(status) => (status.code() as i32, Some(status.message().to_string()), 0),
            };

            let completed_event = ApiCompleted {
                event_id: uuid::Uuid::new_v4().to_string(),
                request_id,
                service_name,
                method_name: method_name.to_str().unwrap_or("unknown").to_string(),
                status_code,
                error_message,
                response_size_bytes: response_size,
                duration_ms,
                occurred_at: chrono::Utc::now(),
                version: 1,
            };

            // Spawn async task to publish completion event
            let dispatch_future = async move {
                let _ = dispatcher
                    .dispatch(
                        &completed_event.aggregate_id(),
                        vec![Box::new(completed_event) as Box<dyn DomainEvent>],
                        0,
                    )
                    .await;
            };
            tokio::spawn(dispatch_future);

            result
        }
    }
}

/// Extract client IP from request metadata
fn extract_client_ip<B>(request: &Request<B>) -> Option<String> {
    // Try to get from grpc-remote-addr header
    if let Some(addr) = request.metadata().get("grpc-remote-addr") {
        return Some(addr.to_str().unwrap_or("unknown").to_string());
    }

    // Try to get from x-forwarded-for header
    if let Some(addr) = request.metadata().get("x-forwarded-for") {
        return Some(addr.to_str().unwrap_or("unknown").to_string());
    }

    None
}

/// Estimate request size in bytes
fn estimate_request_size<B>(request: &Request<B>) -> i64 {
    // This is a rough estimate - in production you might want to serialize the request
    // For now, we use a heuristic based on the number of metadata headers
    let header_count = request.metadata().len() as i64;
    // Rough estimate: 100 bytes per header + some base size
    (header_count * 100 + 500) as i64
}

/// Generate a unique request ID
fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
