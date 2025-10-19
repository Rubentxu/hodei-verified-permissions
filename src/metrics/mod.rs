//! Metrics collection for authorization service
//!
//! This module provides lightweight metrics collection for monitoring
//! cache performance, authorization latencies, and throughput.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metrics collector for the authorization service
#[derive(Debug, Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

#[derive(Debug)]
struct MetricsInner {
    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_size: AtomicU64,
    
    // Authorization metrics
    authorization_count: AtomicU64,
    authorization_allow: AtomicU64,
    authorization_deny: AtomicU64,
    
    // Latency metrics (in microseconds)
    total_latency_us: AtomicU64,
    min_latency_us: AtomicU64,
    max_latency_us: AtomicU64,
    
    // Throughput metrics
    requests_per_second: AtomicU64,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                cache_hits: AtomicU64::new(0),
                cache_misses: AtomicU64::new(0),
                cache_size: AtomicU64::new(0),
                authorization_count: AtomicU64::new(0),
                authorization_allow: AtomicU64::new(0),
                authorization_deny: AtomicU64::new(0),
                total_latency_us: AtomicU64::new(0),
                min_latency_us: AtomicU64::new(u64::MAX),
                max_latency_us: AtomicU64::new(0),
                requests_per_second: AtomicU64::new(0),
            }),
        }
    }

    // Cache metrics
    
    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.inner.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.inner.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Update cache size
    pub fn set_cache_size(&self, size: u64) {
        self.inner.cache_size.store(size, Ordering::Relaxed);
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.inner.cache_hits.load(Ordering::Relaxed);
        let misses = self.inner.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    // Authorization metrics
    
    /// Record an authorization decision
    pub fn record_authorization(&self, allow: bool, latency: Duration) {
        self.inner.authorization_count.fetch_add(1, Ordering::Relaxed);
        
        if allow {
            self.inner.authorization_allow.fetch_add(1, Ordering::Relaxed);
        } else {
            self.inner.authorization_deny.fetch_add(1, Ordering::Relaxed);
        }
        
        // Record latency
        let latency_us = latency.as_micros() as u64;
        self.inner.total_latency_us.fetch_add(latency_us, Ordering::Relaxed);
        
        // Update min latency
        let mut current_min = self.inner.min_latency_us.load(Ordering::Relaxed);
        while latency_us < current_min {
            match self.inner.min_latency_us.compare_exchange(
                current_min,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }
        
        // Update max latency
        let mut current_max = self.inner.max_latency_us.load(Ordering::Relaxed);
        while latency_us > current_max {
            match self.inner.max_latency_us.compare_exchange(
                current_max,
                latency_us,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    /// Get average authorization latency in microseconds
    pub fn avg_latency_us(&self) -> u64 {
        let total = self.inner.total_latency_us.load(Ordering::Relaxed);
        let count = self.inner.authorization_count.load(Ordering::Relaxed);
        
        if count == 0 {
            0
        } else {
            total / count
        }
    }

    /// Get minimum latency in microseconds
    pub fn min_latency_us(&self) -> u64 {
        let min = self.inner.min_latency_us.load(Ordering::Relaxed);
        if min == u64::MAX {
            0
        } else {
            min
        }
    }

    /// Get maximum latency in microseconds
    pub fn max_latency_us(&self) -> u64 {
        self.inner.max_latency_us.load(Ordering::Relaxed)
    }

    // Summary methods
    
    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            cache_hits: self.inner.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.inner.cache_misses.load(Ordering::Relaxed),
            cache_size: self.inner.cache_size.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            authorization_count: self.inner.authorization_count.load(Ordering::Relaxed),
            authorization_allow: self.inner.authorization_allow.load(Ordering::Relaxed),
            authorization_deny: self.inner.authorization_deny.load(Ordering::Relaxed),
            avg_latency_us: self.avg_latency_us(),
            min_latency_us: self.min_latency_us(),
            max_latency_us: self.max_latency_us(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.inner.cache_hits.store(0, Ordering::Relaxed);
        self.inner.cache_misses.store(0, Ordering::Relaxed);
        self.inner.authorization_count.store(0, Ordering::Relaxed);
        self.inner.authorization_allow.store(0, Ordering::Relaxed);
        self.inner.authorization_deny.store(0, Ordering::Relaxed);
        self.inner.total_latency_us.store(0, Ordering::Relaxed);
        self.inner.min_latency_us.store(u64::MAX, Ordering::Relaxed);
        self.inner.max_latency_us.store(0, Ordering::Relaxed);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: u64,
    pub cache_hit_rate: f64,
    pub authorization_count: u64,
    pub authorization_allow: u64,
    pub authorization_deny: u64,
    pub avg_latency_us: u64,
    pub min_latency_us: u64,
    pub max_latency_us: u64,
}

impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Authorization Metrics ===")?;
        writeln!(f, "Cache:")?;
        writeln!(f, "  Hits:      {}", self.cache_hits)?;
        writeln!(f, "  Misses:    {}", self.cache_misses)?;
        writeln!(f, "  Hit Rate:  {:.2}%", self.cache_hit_rate * 100.0)?;
        writeln!(f, "  Size:      {} stores", self.cache_size)?;
        writeln!(f)?;
        writeln!(f, "Authorization:")?;
        writeln!(f, "  Total:     {}", self.authorization_count)?;
        writeln!(f, "  Allow:     {}", self.authorization_allow)?;
        writeln!(f, "  Deny:      {}", self.authorization_deny)?;
        writeln!(f)?;
        writeln!(f, "Latency (μs):")?;
        writeln!(f, "  Average:   {}", self.avg_latency_us)?;
        writeln!(f, "  Min:       {}", self.min_latency_us)?;
        writeln!(f, "  Max:       {}", self.max_latency_us)?;
        Ok(())
    }
}

/// Helper to measure operation latency
pub struct LatencyTimer {
    start: Instant,
}

impl LatencyTimer {
    /// Start a new latency timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.cache_hits, 0);
        assert_eq!(snapshot.cache_misses, 0);
        assert_eq!(snapshot.authorization_count, 0);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = Metrics::new();
        
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert!((snapshot.cache_hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_authorization_metrics() {
        let metrics = Metrics::new();
        
        metrics.record_authorization(true, Duration::from_micros(100));
        metrics.record_authorization(false, Duration::from_micros(200));
        metrics.record_authorization(true, Duration::from_micros(150));
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.authorization_count, 3);
        assert_eq!(snapshot.authorization_allow, 2);
        assert_eq!(snapshot.authorization_deny, 1);
        assert_eq!(snapshot.avg_latency_us, 150);
        assert_eq!(snapshot.min_latency_us, 100);
        assert_eq!(snapshot.max_latency_us, 200);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();
        
        metrics.record_cache_hit();
        metrics.record_authorization(true, Duration::from_micros(100));
        
        metrics.reset();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 0);
        assert_eq!(snapshot.authorization_count, 0);
    }

    #[test]
    fn test_latency_timer() {
        let timer = LatencyTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_metrics_display() {
        let metrics = Metrics::new();
        metrics.record_cache_hit();
        metrics.record_authorization(true, Duration::from_micros(100));
        
        let snapshot = metrics.snapshot();
        let display = format!("{}", snapshot);
        
        assert!(display.contains("Authorization Metrics"));
        assert!(display.contains("Cache:"));
        assert!(display.contains("Latency"));
    }
}
