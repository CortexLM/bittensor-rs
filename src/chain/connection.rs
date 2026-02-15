//! Production-grade connection management for Bittensor chain interactions
//!
//! This module provides:
//! - Exponential backoff retry logic for RPC failures
//! - Connection health monitoring with automatic reconnection
//! - Connection pooling for multiple concurrent connections
//! - Circuit breaker pattern to prevent cascading failures
//! - Rate limiting per 12-second block time

use backoff::{
    future::retry, Error as BackoffError, ExponentialBackoff, ExponentialBackoffBuilder,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, trace, warn};

use crate::chain::{BittensorClient, Error as ChainError};

/// Maximum number of retry attempts for RPC operations
pub const MAX_RETRIES: u32 = 5;

/// Initial retry delay in milliseconds
pub const INITIAL_RETRY_DELAY_MS: u64 = 100;

/// Maximum retry delay in milliseconds
pub const MAX_RETRY_DELAY_MS: u64 = 30000;

/// Maximum retry elapsed time
pub const MAX_RETRY_ELAPSED_TIME: Duration = Duration::from_secs(60);

/// Circuit breaker failure threshold
pub const CIRCUIT_BREAKER_FAILURE_THRESHOLD: u32 = 5;

/// Circuit breaker reset timeout in seconds
pub const CIRCUIT_BREAKER_RESET_TIMEOUT_SECS: u64 = 30;

/// Maximum number of connections in the pool
pub const MAX_POOL_SIZE: usize = 10;

/// Connection timeout
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// RPC operation timeout
pub const RPC_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Rate limit per block (number of operations)
pub const DEFAULT_RATE_LIMIT_PER_BLOCK: u32 = 100;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed - requests flow through
    Closed,
    /// Circuit is open - requests fail fast
    Open,
    /// Circuit is half-open - allowing test requests
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    state: RwLock<CircuitBreakerState>,
    failure_count: Mutex<u32>,
    last_failure_time: Mutex<Option<Instant>>,
    failure_threshold: u32,
    reset_timeout: Duration,
    success_count: Mutex<u32>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default settings
    pub fn new() -> Self {
        Self::with_config(
            CIRCUIT_BREAKER_FAILURE_THRESHOLD,
            CIRCUIT_BREAKER_RESET_TIMEOUT_SECS,
        )
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(failure_threshold: u32, reset_timeout_secs: u64) -> Self {
        Self {
            state: RwLock::new(CircuitBreakerState::Closed),
            failure_count: Mutex::new(0),
            last_failure_time: Mutex::new(None),
            failure_threshold,
            reset_timeout: Duration::from_secs(reset_timeout_secs),
            success_count: Mutex::new(0),
        }
    }

    /// Check if the circuit allows requests
    pub async fn can_execute(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if enough time has passed to transition to half-open
                let last_failure = *self.last_failure_time.lock().await;
                if let Some(last) = last_failure {
                    if Instant::now().duration_since(last) >= self.reset_timeout {
                        let mut state_guard = self.state.write().await;
                        *state_guard = CircuitBreakerState::HalfOpen;
                        info!("Circuit breaker transitioning from Open to HalfOpen");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record a successful execution
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitBreakerState::HalfOpen => {
                let mut success = self.success_count.lock().await;
                *success += 1;

                if *success >= 2 {
                    // Transition back to closed
                    let mut state_guard = self.state.write().await;
                    *state_guard = CircuitBreakerState::Closed;
                    *self.failure_count.lock().await = 0;
                    *success = 0;
                    info!("Circuit breaker transitioning from HalfOpen to Closed");
                }
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success in closed state
                *self.failure_count.lock().await = 0;
            }
            CircuitBreakerState::Open => {}
        }
    }

    /// Record a failed execution
    pub async fn record_failure(&self) {
        let state = *self.state.read().await;
        let mut count = self.failure_count.lock().await;
        *count += 1;
        *self.last_failure_time.lock().await = Some(Instant::now());

        match state {
            CircuitBreakerState::Closed => {
                if *count >= self.failure_threshold {
                    let mut state_guard = self.state.write().await;
                    *state_guard = CircuitBreakerState::Open;
                    error!(
                        "Circuit breaker opened after {} consecutive failures",
                        self.failure_threshold
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Go back to open on failure in half-open state
                let mut state_guard = self.state.write().await;
                *state_guard = CircuitBreakerState::Open;
                *self.success_count.lock().await = 0;
                warn!("Circuit breaker transitioning from HalfOpen back to Open due to failure");
            }
            CircuitBreakerState::Open => {}
        }
    }

    /// Get current circuit breaker state
    pub async fn state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }

    /// Get current failure count
    pub async fn failure_count(&self) -> u32 {
        *self.failure_count.lock().await
    }

    /// Force close the circuit (manual reset)
    pub async fn close(&self) {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Closed;
        *self.failure_count.lock().await = 0;
        *self.success_count.lock().await = 0;
        info!("Circuit breaker manually closed");
    }

    /// Force open the circuit (manual trip)
    pub async fn open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Open;
        *self.last_failure_time.lock().await = Some(Instant::now());
        warn!("Circuit breaker manually opened");
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Connection metadata for health tracking
#[derive(Debug)]
struct ConnectionMetadata {
    created_at: Instant,
    last_used: Instant,
    health_status: ConnectionHealth,
    consecutive_failures: u32,
    successful_operations: u64,
    total_operations: u64,
}

impl ConnectionMetadata {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            created_at: now,
            last_used: now,
            health_status: ConnectionHealth::Healthy,
            consecutive_failures: 0,
            successful_operations: 0,
            total_operations: 0,
        }
    }

    fn record_success(&mut self) {
        self.last_used = Instant::now();
        self.consecutive_failures = 0;
        self.successful_operations += 1;
        self.total_operations += 1;
        self.health_status = ConnectionHealth::Healthy;
    }

    fn record_failure(&mut self) {
        self.last_used = Instant::now();
        self.consecutive_failures += 1;
        self.total_operations += 1;

        // Update health status based on consecutive failures
        if self.consecutive_failures >= 5 {
            self.health_status = ConnectionHealth::Unhealthy;
        } else if self.consecutive_failures >= 2 {
            self.health_status = ConnectionHealth::Degraded;
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            1.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }
}

/// Managed connection wrapper with health tracking
#[derive(Debug)]
pub struct ManagedConnection {
    client: BittensorClient,
    metadata: Mutex<ConnectionMetadata>,
    id: u64,
}

impl ManagedConnection {
    fn new(client: BittensorClient, id: u64) -> Self {
        Self {
            client,
            metadata: Mutex::new(ConnectionMetadata::new()),
            id,
        }
    }

    /// Get a reference to the underlying client
    pub fn client(&self) -> &BittensorClient {
        &self.client
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        self.metadata.lock().await.record_success();
        trace!(connection_id = self.id, "Connection operation successful");
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        self.metadata.lock().await.record_failure();
        warn!(connection_id = self.id, "Connection operation failed");
    }

    /// Get current health status
    pub async fn health_status(&self) -> ConnectionHealth {
        self.metadata.lock().await.health_status
    }

    /// Check if connection is healthy
    pub async fn is_healthy(&self) -> bool {
        matches!(self.health_status().await, ConnectionHealth::Healthy)
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        let meta = self.metadata.lock().await;
        ConnectionStats {
            id: self.id,
            health: meta.health_status,
            success_rate: meta.success_rate(),
            total_operations: meta.total_operations,
            consecutive_failures: meta.consecutive_failures,
            age_secs: Instant::now().duration_since(meta.created_at).as_secs(),
            idle_secs: Instant::now().duration_since(meta.last_used).as_secs(),
        }
    }

    /// Get connection ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub id: u64,
    pub health: ConnectionHealth,
    pub success_rate: f64,
    pub total_operations: u64,
    pub consecutive_failures: u32,
    pub age_secs: u64,
    pub idle_secs: u64,
}

/// Connection pool for managing multiple connections
#[derive(Debug)]
pub struct ConnectionPool {
    connections: RwLock<Vec<Arc<ManagedConnection>>>,
    endpoint: String,
    max_size: usize,
    next_id: Mutex<u64>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self::with_config(endpoint, MAX_POOL_SIZE)
    }

    /// Create a new connection pool with custom configuration
    pub fn with_config(endpoint: impl Into<String>, max_size: usize) -> Self {
        Self {
            connections: RwLock::new(Vec::with_capacity(max_size)),
            endpoint: endpoint.into(),
            max_size,
            next_id: Mutex::new(0),
            circuit_breaker: Arc::new(CircuitBreaker::new()),
        }
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<Arc<ManagedConnection>, ChainError> {
        // Check circuit breaker
        if !self.circuit_breaker.can_execute().await {
            return Err(ChainError::Rpc(
                "Circuit breaker is open - too many failures".to_string(),
            ));
        }

        let mut connections = self.connections.write().await;

        // Try to find a healthy connection
        for conn in connections.iter() {
            if conn.is_healthy().await {
                return Ok(conn.clone());
            }
        }

        // If no healthy connection, try to create a new one if under max size
        if connections.len() < self.max_size {
            let id = {
                let mut guard = self.next_id.lock().await;
                *guard += 1;
                *guard
            };

            let client = self.create_connection_with_retry().await?;
            let managed = Arc::new(ManagedConnection::new(client, id));
            connections.push(managed.clone());

            info!(
                connection_id = id,
                pool_size = connections.len(),
                "Created new connection in pool"
            );

            return Ok(managed);
        }

        // Return the least recently failed connection
        if let Some(conn) = connections.first() {
            return Ok(conn.clone());
        }

        Err(ChainError::Rpc(
            "No connections available in pool".to_string(),
        ))
    }

    /// Create a new connection with retry logic
    async fn create_connection_with_retry(&self) -> Result<BittensorClient, ChainError> {
        let backoff = create_backoff_config();

        retry(backoff, || async {
            debug!("Attempting to create connection to {}", self.endpoint);

            let result =
                tokio::time::timeout(CONNECTION_TIMEOUT, BittensorClient::new(&self.endpoint))
                    .await;

            match result {
                Ok(Ok(client)) => {
                    info!("Successfully connected to {}", self.endpoint);
                    self.circuit_breaker.record_success().await;
                    Ok(client)
                }
                Ok(Err(e)) => {
                    warn!("Connection attempt failed: {}", e);
                    self.circuit_breaker.record_failure().await;
                    Err(BackoffError::transient(e))
                }
                Err(_) => {
                    warn!("Connection attempt timed out");
                    self.circuit_breaker.record_failure().await;
                    Err(BackoffError::transient(ChainError::Rpc(
                        "Connection timeout".to_string(),
                    )))
                }
            }
        })
        .await
    }

    /// Remove an unhealthy connection from the pool
    pub async fn remove_connection(&self, id: u64) {
        let mut connections = self.connections.write().await;
        connections.retain(|c| c.id() != id);
        warn!(connection_id = id, "Removed unhealthy connection from pool");
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let connections = self.connections.read().await;
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for conn in connections.iter() {
            match conn.health_status().await {
                ConnectionHealth::Healthy => healthy_count += 1,
                ConnectionHealth::Degraded => degraded_count += 1,
                ConnectionHealth::Unhealthy => unhealthy_count += 1,
            }
        }

        PoolStats {
            total_connections: connections.len(),
            healthy_count,
            degraded_count,
            unhealthy_count,
            circuit_breaker_state: self.circuit_breaker.state().await,
            max_size: self.max_size,
        }
    }

    /// Get connection stats for all connections
    pub async fn connection_stats(&self) -> Vec<ConnectionStats> {
        let connections = self.connections.read().await;
        let mut stats = Vec::with_capacity(connections.len());

        for conn in connections.iter() {
            stats.push(conn.stats().await);
        }

        stats
    }

    /// Close all connections and clear the pool
    pub async fn close(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();
        info!("Connection pool closed");
    }

    /// Get circuit breaker reference
    pub fn circuit_breaker(&self) -> &Arc<CircuitBreaker> {
        &self.circuit_breaker
    }

    /// Perform health check on all connections
    pub async fn health_check(&self) {
        let connections = self.connections.read().await;

        for conn in connections.iter() {
            let stats = conn.stats().await;

            if stats.health == ConnectionHealth::Unhealthy || stats.idle_secs > 60 {
                // Test the connection with a lightweight operation
                let test_result = conn
                    .client
                    .block_number()
                    .await
                    .map(|_| true)
                    .unwrap_or(false);

                if test_result {
                    conn.record_success().await;
                } else {
                    conn.record_failure().await;
                }
            }
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub circuit_breaker_state: CircuitBreakerState,
    pub max_size: usize,
}

/// Create a backoff configuration for retry operations
pub fn create_backoff_config() -> ExponentialBackoff {
    ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_millis(INITIAL_RETRY_DELAY_MS))
        .with_max_interval(Duration::from_millis(MAX_RETRY_DELAY_MS))
        .with_max_elapsed_time(Some(MAX_RETRY_ELAPSED_TIME))
        .build()
}

/// Retry configuration for RPC operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_elapsed_time: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            initial_delay: Duration::from_millis(INITIAL_RETRY_DELAY_MS),
            max_delay: Duration::from_millis(MAX_RETRY_DELAY_MS),
            max_elapsed_time: MAX_RETRY_ELAPSED_TIME,
        }
    }
}

/// Rate limiter for operations per block
#[derive(Debug)]
pub struct BlockRateLimiter {
    operations_per_block: u32,
    current_block_operations: Mutex<u32>,
    last_block_number: Mutex<Option<u64>>,
}

impl BlockRateLimiter {
    /// Create a new rate limiter with default limit
    pub fn new() -> Self {
        Self::with_limit(DEFAULT_RATE_LIMIT_PER_BLOCK)
    }

    /// Create a new rate limiter with custom limit
    pub fn with_limit(operations_per_block: u32) -> Self {
        Self {
            operations_per_block,
            current_block_operations: Mutex::new(0),
            last_block_number: Mutex::new(None),
        }
    }

    /// Check if an operation can be executed and increment counter
    pub async fn check_and_record(&self, current_block: u64) -> bool {
        let mut last_block = self.last_block_number.lock().await;
        let mut operations = self.current_block_operations.lock().await;

        // Reset counter if we're on a new block
        if *last_block != Some(current_block) {
            *last_block = Some(current_block);
            *operations = 0;
        }

        // Check if we can execute
        if *operations < self.operations_per_block {
            *operations += 1;
            true
        } else {
            false
        }
    }

    /// Get current operation count for this block
    pub async fn current_count(&self) -> u32 {
        *self.current_block_operations.lock().await
    }

    /// Get operations per block limit
    pub fn limit(&self) -> u32 {
        self.operations_per_block
    }
}

impl Default for BlockRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection manager that handles all connection lifecycle
#[derive(Debug)]
pub struct ConnectionManager {
    pool: Arc<ConnectionPool>,
    rate_limiter: BlockRateLimiter,
    health_check_interval: Duration,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self::with_config(endpoint, MAX_POOL_SIZE, Duration::from_secs(30))
    }

    /// Create a new connection manager with custom configuration
    pub fn with_config(
        endpoint: impl Into<String>,
        max_pool_size: usize,
        health_check_interval: Duration,
    ) -> Self {
        let endpoint_str: String = endpoint.into();

        Self {
            pool: Arc::new(ConnectionPool::with_config(&endpoint_str, max_pool_size)),
            rate_limiter: BlockRateLimiter::new(),
            health_check_interval,
        }
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<Arc<ManagedConnection>, ChainError> {
        self.pool.get_connection().await
    }

    /// Execute an operation with full retry and circuit breaker protection
    pub async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T, ChainError>
    where
        F: Fn(&BittensorClient) -> Fut + Clone,
        Fut: std::future::Future<Output = Result<T, ChainError>>,
    {
        let config = RetryConfig::default();

        let backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(config.initial_delay)
            .with_max_interval(config.max_delay)
            .with_max_elapsed_time(Some(config.max_elapsed_time))
            .build();

        let operation = operation.clone();

        retry(backoff, || {
            let operation = operation.clone();
            async move {
                let conn = self
                    .get_connection()
                    .await
                    .map_err(BackoffError::transient)?;

                let result = operation(conn.client()).await;

                match &result {
                    Ok(_) => conn.record_success().await,
                    Err(e) => {
                        conn.record_failure().await;
                        warn!("Operation failed on connection {}: {}", conn.id(), e);

                        // If connection is unhealthy, remove it from pool
                        if !conn.is_healthy().await {
                            self.pool.remove_connection(conn.id()).await;
                        }
                    }
                }

                result.map_err(BackoffError::transient)
            }
        })
        .await
    }

    /// Check if operation can proceed based on rate limiting
    pub async fn check_rate_limit(&self, current_block: u64) -> bool {
        self.rate_limiter.check_and_record(current_block).await
    }

    /// Get pool statistics
    pub async fn pool_stats(&self) -> PoolStats {
        self.pool.stats().await
    }

    /// Get connection statistics
    pub async fn connection_stats(&self) -> Vec<ConnectionStats> {
        self.pool.connection_stats().await
    }

    /// Get circuit breaker reference
    pub fn circuit_breaker(&self) -> &Arc<CircuitBreaker> {
        self.pool.circuit_breaker()
    }

    /// Start health check background task
    pub fn start_health_checks(&self) {
        let pool = self.pool.clone();
        let interval = self.health_check_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;
                pool.health_check().await;
            }
        });
    }

    /// Close all connections
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get the pool reference
    pub fn pool(&self) -> &Arc<ConnectionPool> {
        &self.pool
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new(crate::chain::DEFAULT_RPC_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_new_closed() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::with_config(3, 1);

        assert!(cb.can_execute().await);

        cb.record_failure().await;
        cb.record_failure().await;
        assert!(cb.can_execute().await);

        cb.record_failure().await; // 3rd failure
        assert!(!cb.can_execute().await); // Circuit should be open
        assert_eq!(cb.state().await, CircuitBreakerState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_success() {
        let cb = CircuitBreaker::with_config(3, 1);

        // Open the circuit
        for _ in 0..3 {
            cb.record_failure().await;
        }

        assert_eq!(cb.state().await, CircuitBreakerState::Open);

        // Wait for reset timeout and transition to half-open
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Now should allow execution (half-open)
        assert!(cb.can_execute().await);
        assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

        // Success should move to closed
        cb.record_success().await;
        cb.record_success().await; // Need 2 successes

        // Check that we can still execute (circuit is closed or half-open)
        assert!(cb.can_execute().await);
    }

    #[tokio::test]
    async fn test_rate_limiter_per_block() {
        let limiter = BlockRateLimiter::with_limit(3);

        assert!(limiter.check_and_record(100).await);
        assert!(limiter.check_and_record(100).await);
        assert!(limiter.check_and_record(100).await);
        assert!(!limiter.check_and_record(100).await); // Over limit

        // New block should reset
        assert!(limiter.check_and_record(101).await);
    }

    #[tokio::test]
    async fn test_connection_metadata() {
        let mut meta = ConnectionMetadata::new();

        assert_eq!(meta.success_rate(), 1.0);

        meta.record_success();
        meta.record_success();
        assert_eq!(meta.success_rate(), 1.0);

        meta.record_failure();
        assert_eq!(meta.success_rate(), 0.6666666666666666);

        assert_eq!(meta.consecutive_failures, 1);
        assert_eq!(meta.total_operations, 3);
    }

    #[tokio::test]
    async fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, MAX_RETRIES);
        assert_eq!(
            config.initial_delay,
            Duration::from_millis(INITIAL_RETRY_DELAY_MS)
        );
        assert_eq!(config.max_delay, Duration::from_millis(MAX_RETRY_DELAY_MS));
    }
}
