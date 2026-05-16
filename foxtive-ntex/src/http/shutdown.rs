//! Graceful shutdown coordination for foxtive-ntex services
//!
//! This module provides a registry-based system for managing service shutdown
//! with timeout handling and priority-based cleanup ordering.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tracing::{info, warn, error};

/// A service that needs to perform cleanup during shutdown
pub struct ShutdownService {
    /// Name of the service (for logging)
    pub name: String,
    
    /// Priority for shutdown ordering (lower = shutdown first)
    /// Typical priorities:
    /// - 0-10: Critical infrastructure (databases, message queues)
    /// - 11-50: Application services (caches, connection pools)
    /// - 51-100: Auxiliary services (loggers, metrics)
    pub priority: u8,
    
    /// The cleanup function that will be called during shutdown
    cleanup: Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>,
}

impl ShutdownService {
    /// Create a new shutdown service
    pub fn new<F, Fut>(name: &str, priority: u8, cleanup: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            name: name.to_string(),
            priority,
            cleanup: Box::new(move || Box::pin(cleanup())),
        }
    }
}

/// Registry for managing shutdown services
///
/// Services are shut down in priority order (lowest priority number first).
/// Each service has a timeout to prevent one slow service from blocking others.
pub struct ShutdownRegistry {
    services: Vec<ShutdownService>,
    default_timeout: Duration,
}

impl ShutdownRegistry {
    /// Create a new shutdown registry
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            default_timeout: Duration::from_secs(10), // 10 seconds per service
        }
    }
    
    /// Create a new shutdown registry with custom default timeout
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            services: Vec::new(),
            default_timeout: Duration::from_secs(timeout_secs),
        }
    }
    
    /// Register a service for shutdown cleanup
    ///
    /// # Arguments
    /// * `name` - Name of the service (used for logging)
    /// * `priority` - Shutdown priority (lower = shutdown first)
    /// * `cleanup` - Async cleanup function to execute
    ///
    /// # Example
    /// ```rust,no_run
    /// use foxtive_ntex::http::shutdown::ShutdownRegistry;
    ///
    /// let mut registry = ShutdownRegistry::new();
    /// registry.register("database", 1, || async {
    ///     // Close database connections
    /// });
    /// ```
    pub fn register<F, Fut>(&mut self, name: &str, priority: u8, cleanup: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.services.push(ShutdownService::new(name, priority, cleanup));
    }
    
    /// Get the number of registered services
    pub fn len(&self) -> usize {
        self.services.len()
    }
    
    /// Check if there are no registered services
    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }
    
    /// Execute shutdown for all registered services
    ///
    /// Services are shut down in priority order (lowest number first).
    /// Each service has a timeout to prevent blocking.
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for each service
    ///
    /// # Example
    /// ```rust,ignore
    /// use std::time::Duration;
    /// use foxtive_ntex::http::shutdown::ShutdownRegistry;
    ///
    /// # async fn example() {
    /// let mut registry = ShutdownRegistry::new();
    /// registry.register("service", 1, || async {
    ///     // cleanup logic
    /// });
    ///
    /// registry.shutdown_all(Duration::from_secs(30)).await;
    /// # }
    /// ```
    pub async fn shutdown_all(&mut self, timeout: Option<Duration>) {
        if self.services.is_empty() {
            info!("No services to shut down");
            return;
        }
        
        // Sort by priority (lowest first)
        self.services.sort_by_key(|s| s.priority);
        
        let service_timeout = timeout.unwrap_or(self.default_timeout);
        let total_services = self.services.len();
        
        info!(
            "Starting graceful shutdown of {} services (timeout: {:?} per service)",
            total_services, service_timeout
        );
        
        // Take ownership of all services
        let services = std::mem::take(&mut self.services);
        
        for (index, service) in services.into_iter().enumerate() {
            info!(
                "[{}/{}] Shutting down '{}' (priority: {})...",
                index + 1,
                total_services,
                service.name,
                service.priority
            );
            
            let start = tokio::time::Instant::now();
            
            // Execute cleanup with timeout
            let cleanup_future = (service.cleanup)();
            
            match tokio::time::timeout(service_timeout, cleanup_future).await {
                Ok(_) => {
                    let elapsed = start.elapsed();
                    info!(
                        "[{}/{}] Service '{}' shut down successfully in {:?}",
                        index + 1, total_services, service.name, elapsed
                    );
                }
                Err(_) => {
                    let elapsed = start.elapsed();
                    error!(
                        "[{}/{}] Service '{}' shutdown timed out after {:?} (limit: {:?})",
                        index + 1, total_services, service.name, elapsed, service_timeout
                    );
                    warn!(
                        "Service '{}' may not have cleaned up properly. Continuing with shutdown...",
                        service.name
                    );
                }
            }
        }
        
        info!("All services shut down completed");
    }
}

impl Default for ShutdownRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for shutdown behavior
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Total shutdown timeout
    pub timeout: Duration,
    
    /// Per-service timeout
    pub service_timeout: Duration,
    
    /// Whether to force kill after timeout
    pub force_kill: bool,
}

impl ShutdownConfig {
    /// Create a new shutdown configuration
    ///
    /// # Arguments
    /// * `timeout_secs` - Total timeout in seconds
    ///
    /// Service timeout is automatically set to half of the total timeout.
    pub fn new(timeout_secs: u64) -> Self {
        let timeout = Duration::from_secs(timeout_secs);
        Self {
            timeout,
            service_timeout: Duration::from_secs(timeout_secs / 2),
            force_kill: true,
        }
    }
    
    /// Create a new shutdown configuration with explicit timeouts
    pub fn with_timeouts(total_timeout_secs: u64, service_timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(total_timeout_secs),
            service_timeout: Duration::from_secs(service_timeout_secs),
            force_kill: true,
        }
    }
    
    /// Set whether to force kill after timeout
    pub fn force_kill(mut self, force: bool) -> Self {
        self.force_kill = force;
        self
    }
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self::new(30) // 30 seconds total timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_shutdown_registry_empty() {
        let mut registry = ShutdownRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        
        // Should complete immediately with no services
        registry.shutdown_all(None).await;
    }

    #[tokio::test]
    async fn test_shutdown_registry_single_service() {
        let executed = Arc::new(Mutex::new(false));
        let executed_clone = executed.clone();
        
        let mut registry = ShutdownRegistry::new();
        registry.register("test_service", 1, move || {
            let exec = executed_clone.clone();
            async move {
                let mut flag = exec.lock().await;
                *flag = true;
            }
        });
        
        assert_eq!(registry.len(), 1);
        
        registry.shutdown_all(None).await;
        
        let flag = executed.lock().await;
        assert!(*flag);
    }

    #[tokio::test]
    async fn test_shutdown_priority_ordering() {
        let order = Arc::new(Mutex::new(Vec::new()));
        
        let mut registry = ShutdownRegistry::new();
        
        // Register services with different priorities
        let order1 = order.clone();
        registry.register("low_priority", 10, move || {
            let ord = order1.clone();
            async move {
                ord.lock().await.push("low_priority");
            }
        });
        
        let order2 = order.clone();
        registry.register("high_priority", 1, move || {
            let ord = order2.clone();
            async move {
                ord.lock().await.push("high_priority");
            }
        });
        
        let order3 = order.clone();
        registry.register("medium_priority", 5, move || {
            let ord = order3.clone();
            async move {
                ord.lock().await.push("medium_priority");
            }
        });
        
        registry.shutdown_all(None).await;
        
        let final_order = order.lock().await;
        // Should shutdown in priority order: 1, 5, 10
        assert_eq!(*final_order, vec!["high_priority", "medium_priority", "low_priority"]);
    }

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let mut registry = ShutdownRegistry::new();
        
        // Register a service that takes longer than the timeout
        registry.register("slow_service", 1, || async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
        
        // Use a short timeout
        let start = tokio::time::Instant::now();
        registry.shutdown_all(Some(Duration::from_secs(1))).await;
        let elapsed = start.elapsed();
        
        // Should timeout after approximately 1 second, not 10
        assert!(elapsed < Duration::from_secs(3));
    }

    #[tokio::test]
    async fn test_shutdown_config() {
        let config = ShutdownConfig::new(30);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.service_timeout, Duration::from_secs(15));
        assert!(config.force_kill);
        
        let config2 = ShutdownConfig::with_timeouts(60, 10);
        assert_eq!(config2.timeout, Duration::from_secs(60));
        assert_eq!(config2.service_timeout, Duration::from_secs(10));
        
        let config3 = ShutdownConfig::new(20).force_kill(false);
        assert!(!config3.force_kill);
    }

    #[tokio::test]
    async fn test_multiple_services_with_timeout() {
        let executed = Arc::new(Mutex::new(Vec::new()));
        
        let mut registry = ShutdownRegistry::with_timeout(2); // 2 second timeout
        
        // Fast service
        let exec1 = executed.clone();
        registry.register("fast", 1, move || {
            let exec = exec1.clone();
            async move {
                exec.lock().await.push("fast");
            }
        });
        
        // Slow service (will timeout)
        let exec2 = executed.clone();
        registry.register("slow", 2, move || {
            let exec = exec2.clone();
            async move {
                tokio::time::sleep(Duration::from_secs(10)).await;
                exec.lock().await.push("slow");
            }
        });
        
        // Another fast service
        let exec3 = executed.clone();
        registry.register("another_fast", 3, move || {
            let exec = exec3.clone();
            async move {
                exec.lock().await.push("another_fast");
            }
        });
        
        registry.shutdown_all(None).await;
        
        let final_executed = executed.lock().await;
        // Fast services should execute, slow one should timeout
        assert_eq!(final_executed.len(), 2);
        assert_eq!(final_executed[0], "fast");
        assert_eq!(final_executed[1], "another_fast");
    }
}
