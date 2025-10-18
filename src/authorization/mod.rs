//! Authorization Service - Evaluación de políticas con Cedar
//! 
//! Este módulo implementa el servicio de autorización que:
//! - Evalúa políticas 100% en memoria usando el cache
//! - Proporciona latencia ultra-baja (~100μs)
//! - Registra decisiones de forma asíncrona

pub mod service;

pub use service::AuthorizationService;
