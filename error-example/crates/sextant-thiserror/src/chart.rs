//! Chart metadata parsing module
//!
//! Handles parsing and validation of Helm Chart.yaml files.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Chart-specific errors
#[derive(thiserror::Error, Debug)]
pub enum ChartError {
    #[error("Failed to read Chart.yaml from {}: {source}", .path.display())]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse Chart.yaml from {}: {source}", .path.display())]
    ParseFailed {
        path: PathBuf,
        #[source]
        source: Box<ChartError>,
    },

    #[error("Invalid YAML format in Chart.yaml: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("Chart name cannot be empty")]
    EmptyName,

    #[error("Chart version cannot be empty")]
    EmptyVersion,

    #[error("Chart apiVersion must be 'v1' or 'v2', got '{}'", .version)]
    InvalidApiVersion { version: String },

    #[error("Chart type must be 'application' or 'library', got '{}'", .chart_type)]
    InvalidChartType { chart_type: String },

    #[error("No Chart.yaml or Chart.yml found in {}", .path.display())]
    ChartFileNotFound { path: PathBuf },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Helm chart metadata from Chart.yaml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartMetadata {
    /// Chart name
    pub name: String,
    /// Chart version
    pub version: String,
    /// Chart description
    pub description: Option<String>,
    /// Chart API version (v1 or v2)
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    /// Chart type (application or library)
    #[serde(rename = "type")]
    pub chart_type: Option<String>,
    /// Chart keywords
    pub keywords: Option<Vec<String>>,
    /// Chart maintainers
    pub maintainers: Option<Vec<Maintainer>>,
    /// Chart dependencies
    pub dependencies: Option<Vec<Dependency>>,
}

/// Chart maintainer information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Maintainer {
    /// Maintainer name
    pub name: String,
    /// Maintainer email
    pub email: Option<String>,
    /// Maintainer URL
    pub url: Option<String>,
}

/// Chart dependency information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Dependency version
    pub version: String,
    /// Dependency repository
    pub repository: Option<String>,
    /// Dependency condition
    pub condition: Option<String>,
}

impl ChartMetadata {
    /// Load chart metadata from a Chart.yaml file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ChartError> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|source| ChartError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;

        Self::from_yaml(&contents).map_err(|source| ChartError::ParseFailed {
            path: path.to_path_buf(),
            source: Box::new(source),
        })
    }

    /// Parse chart metadata from YAML string
    pub fn from_yaml(yaml_content: &str) -> Result<Self, ChartError> {
        serde_yaml::from_str(yaml_content).map_err(ChartError::InvalidYaml)
    }

    /// Validate the chart metadata
    pub fn validate(&self) -> Result<(), ChartError> {
        if self.name.is_empty() {
            return Err(ChartError::EmptyName);
        }

        if self.version.is_empty() {
            return Err(ChartError::EmptyVersion);
        }

        if !matches!(self.api_version.as_str(), "v1" | "v2") {
            return Err(ChartError::InvalidApiVersion {
                version: self.api_version.clone(),
            });
        }

        if let Some(chart_type) = &self.chart_type {
            if !matches!(chart_type.as_str(), "application" | "library") {
                return Err(ChartError::InvalidChartType {
                    chart_type: chart_type.clone(),
                });
            }
        }

        Ok(())
    }

    /// Check if this is a library chart
    pub fn is_library(&self) -> bool {
        self.chart_type.as_deref() == Some("library")
    }

    /// Check if this chart has dependencies
    pub fn has_dependencies(&self) -> bool {
        self.dependencies
            .as_ref()
            .is_some_and(|deps| !deps.is_empty())
    }
}

/// Find Chart.yaml file in a directory
pub fn find_chart_file<P: AsRef<Path>>(chart_dir: P) -> Result<std::path::PathBuf, ChartError> {
    let chart_dir = chart_dir.as_ref();
    let chart_yaml = chart_dir.join("Chart.yaml");

    if chart_yaml.exists() {
        return Ok(chart_yaml);
    }

    // Try Chart.yml as fallback
    let chart_yml = chart_dir.join("Chart.yml");
    if chart_yml.exists() {
        return Ok(chart_yml);
    }

    Err(ChartError::ChartFileNotFound {
        path: chart_dir.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use test_log::test;

    type Result<T> = std::result::Result<T, ChartError>;

    fn create_test_chart_yaml() -> String {
        r#"
apiVersion: v2
name: test-app
version: 1.0.0
description: A test Helm chart
type: application
keywords:
  - web
  - app
maintainers:
  - name: Test Maintainer
    email: test@example.com
dependencies:
  - name: postgresql
    version: 11.6.21
    repository: https://charts.bitnami.com/bitnami
"#
        .trim()
        .to_string()
    }

    #[test]
    fn test_parse_chart_metadata() -> Result<()> {
        let yaml_content = create_test_chart_yaml();
        let metadata = ChartMetadata::from_yaml(&yaml_content)?;

        assert_eq!(metadata.name, "test-app");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.api_version, "v2");
        assert_eq!(metadata.chart_type, Some("application".to_string()));
        assert_eq!(metadata.description, Some("A test Helm chart".to_string()));

        assert!(metadata.keywords.is_some());
        assert_eq!(metadata.keywords.as_ref().unwrap().len(), 2);

        assert!(metadata.maintainers.is_some());
        assert_eq!(
            metadata.maintainers.as_ref().unwrap()[0].name,
            "Test Maintainer"
        );

        assert!(metadata.dependencies.is_some());
        assert_eq!(
            metadata.dependencies.as_ref().unwrap()[0].name,
            "postgresql"
        );

        Ok(())
    }

    #[test]
    fn test_validate_chart_metadata() -> Result<()> {
        let yaml_content = create_test_chart_yaml();
        let metadata = ChartMetadata::from_yaml(&yaml_content)?;

        // Valid metadata should pass validation
        metadata.validate()?;

        Ok(())
    }

    #[test]
    fn test_validate_empty_name_fails() -> Result<()> {
        let mut metadata = ChartMetadata::from_yaml(&create_test_chart_yaml())?;
        metadata.name = String::new();

        let result = metadata.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Chart name cannot be empty")
        );

        Ok(())
    }

    #[test]
    fn test_validate_invalid_api_version_fails() -> Result<()> {
        let mut metadata = ChartMetadata::from_yaml(&create_test_chart_yaml())?;
        metadata.api_version = "v3".to_string();

        let result = metadata.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("apiVersion must be 'v1' or 'v2'")
        );

        Ok(())
    }

    #[test]
    fn test_is_library_chart() -> Result<()> {
        let mut metadata = ChartMetadata::from_yaml(&create_test_chart_yaml())?;
        assert!(!metadata.is_library());

        metadata.chart_type = Some("library".to_string());
        assert!(metadata.is_library());

        Ok(())
    }

    #[test]
    fn test_has_dependencies() -> Result<()> {
        let metadata = ChartMetadata::from_yaml(&create_test_chart_yaml())?;
        assert!(metadata.has_dependencies());

        let mut metadata_no_deps = metadata.clone();
        metadata_no_deps.dependencies = None;
        assert!(!metadata_no_deps.has_dependencies());

        Ok(())
    }

    #[test]
    fn test_load_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_path = temp_dir.path().join("Chart.yaml");

        std::fs::write(&chart_path, create_test_chart_yaml())?;

        let metadata = ChartMetadata::load_from_file(&chart_path)?;
        assert_eq!(metadata.name, "test-app");

        Ok(())
    }

    #[test]
    fn test_find_chart_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_path = temp_dir.path().join("Chart.yaml");

        std::fs::write(&chart_path, create_test_chart_yaml())?;

        let found_path = find_chart_file(temp_dir.path())?;
        assert_eq!(found_path, chart_path);

        Ok(())
    }

    #[test]
    fn test_find_chart_file_yml_fallback() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_path = temp_dir.path().join("Chart.yml");

        std::fs::write(&chart_path, create_test_chart_yaml())?;

        let found_path = find_chart_file(temp_dir.path())?;
        assert_eq!(found_path, chart_path);

        Ok(())
    }

    #[test]
    fn test_find_chart_file_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let result = find_chart_file(temp_dir.path());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No Chart.yaml or Chart.yml found")
        );
    }

    #[test]
    fn test_invalid_yaml_format() {
        let invalid_yaml = "invalid: yaml: content: [";
        let result = ChartMetadata::from_yaml(invalid_yaml);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid YAML format")
        );
    }
}
