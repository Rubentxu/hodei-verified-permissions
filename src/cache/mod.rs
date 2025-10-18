//! Cache layer for in-memory policy evaluation
//! 
//! Este módulo implementa el sistema de cache que mantiene los PolicySets
//! de Cedar en memoria para evaluación ultra-rápida (~100μs).

pub mod policy_store_cache;
pub mod cache_manager;

pub use policy_store_cache::PolicyStoreCache;
pub use cache_manager::CacheManager;
