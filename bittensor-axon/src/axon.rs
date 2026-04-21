//! Axon server — HTTP server with middleware chain for the Bittensor network.

use crate::config::AxonConfig;
use crate::middleware::{
    MiddlewareState, blacklist_middleware, body_hash_middleware, priority_middleware,
    verification_middleware,
};
use crate::router::register_synapse_route;
use axum::Router;
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, broadcast};

/// Axon HTTP server with middleware chain.
///
/// Mirrors the Python SDK's `Axon` class:
/// - `new(config)` — build with configuration
/// - `attach(synapse_name, handler)` — register a route
/// - `start()` — bind and serve
/// - `stop()` — graceful shutdown
/// - `forward()` — default pass-through
pub struct Axon {
    config: AxonConfig,
    router: Router,
    state: MiddlewareState,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl Axon {
    /// Build a new Axon server from the given configuration.
    ///
    /// Sets up the middleware chain (verification → blacklist → priority → body-hash)
    /// and a 404 fallback for unregistered routes.
    pub fn new(config: AxonConfig) -> Self {
        let blacklist = Arc::new(RwLock::new(HashSet::new()));
        let priority_map = Arc::new(RwLock::new(HashMap::new()));

        let state = MiddlewareState {
            axon_hotkey: config.hotkey.clone(),
            blacklist: blacklist.clone(),
            priority_map: priority_map.clone(),
        };

        let router = Router::new()
            .fallback(|| async { axum::http::StatusCode::NOT_FOUND.into_response() })
            .layer(middleware::from_fn(body_hash_middleware))
            .layer(middleware::from_fn(priority_middleware))
            .layer(middleware::from_fn(blacklist_middleware))
            .layer(middleware::from_fn(verification_middleware))
            .layer(axum::Extension(state.clone()));

        Self { config, router, state, shutdown_tx: None }
    }

    /// Register a handler for a named synapse route (e.g. `"TextPrompt"`).
    ///
    /// The route is added as a `POST /{synapse_name}` endpoint.
    pub fn attach<H, T>(mut self, synapse_name: &str, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        self.router = register_synapse_route(self.router, synapse_name, handler);
        self
    }

    /// Bind to the configured address and start serving in a background task.
    ///
    /// Returns the actual `SocketAddr` (useful when `port: 0` for OS-assigned port).
    pub async fn start(&mut self) -> Result<SocketAddr, AxonError> {
        let addr: SocketAddr = self
            .config
            .bind_addr()
            .parse()
            .map_err(|e: std::net::AddrParseError| AxonError::Bind(e.to_string()))?;

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e: std::io::Error| AxonError::Bind(e.to_string()))?;

        let actual_addr =
            listener.local_addr().map_err(|e: std::io::Error| AxonError::Bind(e.to_string()))?;

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let router = self.router.clone();

        tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(shutdown_signal(shutdown_rx))
                .await;
        });

        Ok(actual_addr)
    }

    /// Signal the server to shut down gracefully.
    pub fn stop(&self) -> Result<(), AxonError> {
        if let Some(ref tx) = self.shutdown_tx {
            tx.send(()).map_err(|_| AxonError::Shutdown("channel closed".to_string()))?;
        }
        Ok(())
    }

    /// Default handler for unregistered routes — returns 404.
    pub async fn forward(_request: axum::extract::Request) -> Response {
        (StatusCode::NOT_FOUND, "no handler registered").into_response()
    }

    /// Access the shared middleware state (blacklist, priority map, etc.).
    pub fn middleware_state(&self) -> &MiddlewareState {
        &self.state
    }

    /// Access the original [`AxonConfig`] used to build this axon.
    pub fn config(&self) -> &AxonConfig {
        &self.config
    }

    /// Add a hotkey to the blacklist (requests from this key will be rejected).
    pub async fn blacklist(&self, hotkey: &str) {
        self.state.blacklist.write().await.insert(hotkey.to_string());
    }

    /// Remove a hotkey from the blacklist.
    pub async fn unblacklist(&self, hotkey: &str) {
        self.state.blacklist.write().await.remove(hotkey);
    }

    /// Set the priority for a given hotkey (higher = served first).
    pub async fn set_priority(&self, hotkey: &str, priority: u32) {
        self.state.priority_map.write().await.insert(hotkey.to_string(), priority);
    }
}

async fn shutdown_signal(mut rx: broadcast::Receiver<()>) {
    let _ = rx.recv().await;
}

/// Errors that can occur when starting or stopping an Axon.
#[derive(Debug, thiserror::Error)]
pub enum AxonError {
    /// Failed to bind to the configured address.
    #[error("bind error: {0}")]
    Bind(String),
    /// Failed to send shutdown signal.
    #[error("shutdown error: {0}")]
    Shutdown(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn new_axon_with_defaults() {
        let config = AxonConfig::default();
        let axon = Axon::new(config);
        assert!(axon.shutdown_tx.is_none());
    }

    #[tokio::test]
    async fn attach_handler() {
        let config = AxonConfig::default();
        let axon = Axon::new(config);
        let _axon = axon.attach("TextPrompt", || async { "hello" });
    }

    #[tokio::test]
    async fn start_and_stop() {
        let config = AxonConfig { port: 0, ..Default::default() };
        let mut axon = Axon::new(config);
        let addr = axon.start().await.expect("should bind");
        assert!(addr.port() > 0);
        axon.stop().expect("should shutdown");
    }

    #[tokio::test]
    async fn blacklist_and_unblacklist() {
        let config = AxonConfig::default();
        let axon = Axon::new(config);
        axon.blacklist("5BadKey").await;
        assert!(axon.middleware_state().blacklist.read().await.contains("5BadKey"));
        axon.unblacklist("5BadKey").await;
        assert!(!axon.middleware_state().blacklist.read().await.contains("5BadKey"));
    }

    #[tokio::test]
    async fn set_priority() {
        let config = AxonConfig::default();
        let axon = Axon::new(config);
        axon.set_priority("5HighKey", 10).await;
        assert_eq!(*axon.middleware_state().priority_map.read().await.get("5HighKey").unwrap(), 10);
    }

    #[tokio::test]
    async fn attached_route_responds() {
        let config = AxonConfig::default();
        let axon = Axon::new(config).attach("TextPrompt", || async { "hello" });

        let app = axon.router.clone();

        let req =
            HttpRequest::builder().method("POST").uri("/TextPrompt").body(Body::empty()).unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unregistered_route_returns_404() {
        let config = AxonConfig::default();
        let axon = Axon::new(config);

        let app = axon.router.clone();

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/UnknownSynapse")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
