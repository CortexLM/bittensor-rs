//! Route registration for Synapse types.

use axum::Router;
use axum::routing::post;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Stores registered synapse handlers keyed by synapse name.
#[derive(Debug, Clone)]
pub struct SynapseRegistry {
    handlers: Arc<RwLock<HashMap<String, String>>>,
}

impl SynapseRegistry {
    /// Create a new empty synapse registry.
    pub fn new() -> Self {
        Self { handlers: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Register a synapse name to route path mapping.
    pub async fn register(&self, synapse_name: &str, route_path: &str) {
        self.handlers.write().await.insert(synapse_name.to_string(), route_path.to_string());
    }

    /// Look up the route path for a registered synapse name.
    pub async fn get_route(&self, synapse_name: &str) -> Option<String> {
        self.handlers.read().await.get(synapse_name).cloned()
    }

    /// Return the number of registered synapse handlers.
    pub async fn len(&self) -> usize {
        self.handlers.read().await.len()
    }

    /// Return whether the registry is empty.
    pub async fn is_empty(&self) -> bool {
        self.handlers.read().await.is_empty()
    }
}

impl Default for SynapseRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registers a POST route at `/{synapse_name}` on the given Router.
pub fn register_synapse_route<H, T>(router: Router, synapse_name: &str, handler: H) -> Router
where
    H: axum::handler::Handler<T, ()>,
    T: 'static,
{
    let path = format!("/{}", synapse_name);
    router.route(&path, post(handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use tower::ServiceExt;

    async fn echo_handler() -> &'static str {
        "echo"
    }

    #[tokio::test]
    async fn register_synapse_route_creates_path() {
        let app = Router::new();
        let app = register_synapse_route(app, "TextPrompt", echo_handler);

        let req =
            HttpRequest::builder().method("POST").uri("/TextPrompt").body(Body::empty()).unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unregistered_route_returns_404() {
        let app = Router::new().fallback(|| async { StatusCode::NOT_FOUND });

        let req = HttpRequest::builder()
            .method("POST")
            .uri("/UnknownSynapse")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn registry_register_and_lookup() {
        let registry = SynapseRegistry::new();
        registry.register("TextPrompt", "/TextPrompt").await;
        assert_eq!(registry.get_route("TextPrompt").await, Some("/TextPrompt".to_string()));
        assert_eq!(registry.get_route("Unknown").await, None);
    }

    #[tokio::test]
    async fn registry_len() {
        let registry = SynapseRegistry::new();
        registry.register("A", "/A").await;
        registry.register("B", "/B").await;
        assert_eq!(registry.len().await, 2);
    }
}
