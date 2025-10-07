//! Sextant - Helm Chart Resource Analyzer
//!
//! A tool for analyzing Helm charts and generating reports about the Kubernetes
//! resources they would create. This version uses `anyhow` for error handling.

pub mod analyzer;
pub mod chart;
pub mod report;
pub mod template;

pub use analyzer::{analyze_chart, analyze_charts};
pub use report::{ChartAnalysis, ResourceReport};

/// Main result type using anyhow for error handling
pub type Result<T> = anyhow::Result<T>;
