//! Sextant - Helm Chart Resource Analyzer
//!
//! A tool for analyzing Helm charts and generating reports about the Kubernetes
//! resources they would create. This version uses `thiserror` for error handling.

pub mod analyzer;
pub mod chart;
pub mod report;
pub mod template;

pub use analyzer::{analyze_chart, analyze_charts};
pub use report::{ChartAnalysis, ResourceReport};

/// Main error type using thiserror for error handling
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Chart(#[from] chart::ChartError),

    #[error(transparent)]
    Template(#[from] template::TemplateError),

    #[error(transparent)]
    Analysis(#[from] analyzer::AnalysisError),

    #[error(transparent)]
    Report(#[from] report::ReportError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error("Custom error: {message}")]
    Custom { message: String },
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self::Custom { message: msg }
    }
}

/// Main result type using thiserror for error handling
pub type Result<T> = std::result::Result<T, Error>;
