//! Shared, serializable wire models for the promptogether API.
//!
//! This crate intentionally holds no Axum, SQLx, Tokio, or backend-domain
//! dependencies. It defines the JSON request/response contract that the Rust
//! backend serializes and the TypeScript webapp imports via generated bindings.

pub mod error;
pub mod gallery;
pub mod health;

pub use error::ErrorResponse;
pub use gallery::{
    GalleryProjectDetail, GalleryProjectFile, GalleryProjectFilePhase, GalleryProjectSummary,
};
pub use health::{HealthResponse, HealthStatus};
