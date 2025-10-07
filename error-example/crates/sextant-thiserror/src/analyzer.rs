//! Chart analysis orchestration module
//!
//! This module ties together chart parsing, template rendering, and resource
//! counting to produce comprehensive analysis reports.

use serde_yaml::Value as YamlValue;
use std::path::{Path, PathBuf};

use crate::chart::{ChartMetadata, find_chart_file};
use crate::report::{ChartAnalysis, ResourceInfo, ResourceReport};
use crate::template::{Template, Values, find_template_files, find_values_files};

/// Analysis-specific errors
#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error("Chart analysis failed for {path}: {source}")]
    ChartAnalysisFailed {
        path: PathBuf,
        #[source]
        source: crate::chart::ChartError,
    },

    #[error("Failed to load chart metadata: {0}")]
    MetadataLoad(#[from] crate::chart::ChartError),

    #[error("Chart metadata validation failed: {0}")]
    MetadataValidation(String),

    #[error("Failed to find templates in {path}: {source}")]
    TemplatesNotFound {
        path: PathBuf,
        #[source]
        source: crate::template::TemplateError,
    },

    #[error("Failed to load template {path}: {source}")]
    TemplateLoad {
        path: PathBuf,
        #[source]
        source: crate::template::TemplateError,
    },

    #[error("Failed to find values files in {path}: {source}")]
    ValuesFilesNotFound {
        path: PathBuf,
        #[source]
        source: crate::template::TemplateError,
    },

    #[error("Failed to load values file {path}: {source}")]
    ValuesFileLoad {
        path: PathBuf,
        #[source]
        source: crate::template::TemplateError,
    },

    #[error("Analysis failed for values file {name}: {source}")]
    ValuesAnalysisFailed {
        name: String,
        #[source]
        source: Box<AnalysisError>,
    },

    #[error("Analysis failed with empty values: {0}")]
    EmptyValuesAnalysis(#[source] Box<AnalysisError>),

    #[error("Failed to render template {path}: {source}")]
    TemplateRender {
        path: PathBuf,
        #[source]
        source: crate::template::TemplateError,
    },

    #[error("Failed to extract resources from template {path}: {source}")]
    ResourceExtraction {
        path: PathBuf,
        #[source]
        source: Box<AnalysisError>,
    },

    #[error("Charts directory does not exist: {0}")]
    ChartsDirectoryNotFound(PathBuf),

    #[error("Failed to read charts directory {path}: {source}")]
    ChartsDirectoryRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}

/// Analyze a single Helm chart directory
pub fn analyze_chart<P: AsRef<Path>>(chart_dir: P) -> Result<ChartAnalysis, AnalysisError> {
    let chart_dir = chart_dir.as_ref();

    // Find and parse Chart.yaml
    let chart_file =
        find_chart_file(chart_dir).map_err(|source| AnalysisError::ChartAnalysisFailed {
            path: chart_dir.to_path_buf(),
            source,
        })?;

    let chart_metadata = ChartMetadata::load_from_file(&chart_file)?;

    chart_metadata
        .validate()
        .map_err(|e| AnalysisError::MetadataValidation(format!("{}", e)))?;

    // Skip library charts as they don't produce resources
    if chart_metadata.is_library() {
        return Ok(ChartAnalysis::new(
            chart_metadata.name.clone(),
            chart_metadata.version.clone(),
            chart_dir.to_path_buf(),
            chart_metadata,
        ));
    }

    let mut analysis = ChartAnalysis::new(
        chart_metadata.name.clone(),
        chart_metadata.version.clone(),
        chart_dir.to_path_buf(),
        chart_metadata,
    );

    // Find template files
    let templates_dir = chart_dir.join("templates");
    let template_files = if templates_dir.exists() {
        find_template_files(&templates_dir).map_err(|source| AnalysisError::TemplatesNotFound {
            path: templates_dir.clone(),
            source,
        })?
    } else {
        Vec::new()
    };

    // Load templates
    let mut templates = Vec::new();
    for template_path in template_files {
        let template = Template::load_from_file(&template_path).map_err(|source| {
            AnalysisError::TemplateLoad {
                path: template_path.clone(),
                source,
            }
        })?;

        if !template.is_empty_template() {
            templates.push(template);
        }
    }

    // Find values files
    let values_files =
        find_values_files(chart_dir).map_err(|source| AnalysisError::ValuesFilesNotFound {
            path: chart_dir.to_path_buf(),
            source,
        })?;

    // If no values files found, create a default empty one
    let values_files = if values_files.is_empty() {
        vec![]
    } else {
        values_files
    };

    // Analyze each values file
    for values_path in values_files {
        let values_file_name = values_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let values = Values::load_from_file(&values_path).map_err(|source| {
            AnalysisError::ValuesFileLoad {
                path: values_path.clone(),
                source,
            }
        })?;

        let resource_report = analyze_with_values(&templates, &values).map_err(|source| {
            AnalysisError::ValuesAnalysisFailed {
                name: values_file_name.clone(),
                source: Box::new(source),
            }
        })?;

        analysis.add_resource_report(values_file_name, resource_report);
    }

    // If no values files were found, analyze with empty values
    if analysis.values_file_count() == 0 {
        let empty_values = Values::empty();
        let resource_report = analyze_with_values(&templates, &empty_values)
            .map_err(|source| AnalysisError::EmptyValuesAnalysis(Box::new(source)))?;

        analysis.add_resource_report("default".to_string(), resource_report);
    }

    Ok(analysis)
}

/// Analyze templates with specific values to count resources
fn analyze_with_values(
    templates: &[Template],
    values: &Values,
) -> Result<ResourceReport, AnalysisError> {
    let mut report = ResourceReport::new(
        values
            .source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );

    for template in templates {
        let rendered = template
            .render(values)
            .map_err(|source| AnalysisError::TemplateRender {
                path: template.path.clone(),
                source,
            })?;

        let resources =
            extract_resources_from_yaml(&rendered.rendered_content).map_err(|source| {
                AnalysisError::ResourceExtraction {
                    path: template.path.clone(),
                    source: Box::new(source),
                }
            })?;

        for resource in resources {
            let resource_info =
                ResourceInfo::new(resource.name, resource.namespace, template.path.clone());

            report.add_resource(resource.kind, resource_info);
        }
    }

    Ok(report)
}

/// Extracted Kubernetes resource information
#[derive(Debug, Clone)]
struct ExtractedResource {
    kind: String,
    name: String,
    namespace: Option<String>,
}

/// Extract Kubernetes resources from rendered YAML content
fn extract_resources_from_yaml(
    yaml_content: &str,
) -> Result<Vec<ExtractedResource>, AnalysisError> {
    let mut resources = Vec::new();

    // Split on document separators
    let documents = yaml_content
        .split("---")
        .map(|doc| doc.trim())
        .filter(|doc| !doc.is_empty() && !doc.starts_with('#'));

    for doc in documents {
        if let Ok(parsed) = serde_yaml::from_str::<YamlValue>(doc) {
            if let Some(resource) = extract_resource_info(&parsed)? {
                resources.push(resource);
            }
        }
    }

    Ok(resources)
}

/// Extract resource information from a parsed YAML document
fn extract_resource_info(yaml: &YamlValue) -> Result<Option<ExtractedResource>, AnalysisError> {
    let obj = match yaml.as_mapping() {
        Some(mapping) => mapping,
        None => return Ok(None),
    };

    // Get kind
    let kind = obj
        .get(YamlValue::String("kind".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    // Get metadata
    let metadata = obj
        .get(YamlValue::String("metadata".to_string()))
        .and_then(|v| v.as_mapping());

    let name = metadata
        .and_then(|m| m.get(YamlValue::String("name".to_string())))
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed");

    let namespace = metadata
        .and_then(|m| m.get(YamlValue::String("namespace".to_string())))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Skip empty or invalid resources
    if kind == "Unknown" || name == "unnamed" {
        return Ok(None);
    }

    Ok(Some(ExtractedResource {
        kind: kind.to_string(),
        name: name.to_string(),
        namespace,
    }))
}

/// Analyze multiple chart directories
#[async_backtrace::framed]
pub async fn analyze_charts<P: AsRef<Path>>(
    charts_dir: P,
) -> Result<Vec<ChartAnalysis>, AnalysisError> {
    let charts_dir = charts_dir.as_ref();

    if !charts_dir.exists() {
        return Err(AnalysisError::ChartsDirectoryNotFound(
            charts_dir.to_path_buf(),
        ));
    }

    let mut analyses = Vec::new();
    let mut handles = Vec::new();

    // Find all chart directories
    for entry in
        std::fs::read_dir(charts_dir).map_err(|source| AnalysisError::ChartsDirectoryRead {
            path: charts_dir.to_path_buf(),
            source,
        })?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if this looks like a chart directory
            let chart_yaml = path.join("Chart.yaml");
            let chart_yml = path.join("Chart.yml");

            if chart_yaml.exists() || chart_yml.exists() {
                let chart_path = path.clone();
                let handle = tokio::task::spawn_blocking(move || analyze_chart(&chart_path));
                handles.push(handle);
            }
        }
    }

    // Wait for all analyses to complete
    for handle in handles {
        match handle.await {
            Ok(Ok(analysis)) => analyses.push(analysis),
            Ok(Err(e)) => {
                // Log error but continue with other charts
                eprintln!("Chart analysis failed: {}", e);
            }
            Err(e) => {
                eprintln!("Task failed: {}", e);
            }
        }
    }

    // Sort by chart name for consistent output
    analyses.sort_by(|a, b| a.chart_name.cmp(&b.chart_name));

    Ok(analyses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use test_log::test;

    type Result<T> = std::result::Result<T, AnalysisError>;

    fn create_test_chart_yaml() -> &'static str {
        r#"
apiVersion: v2
name: test-app
version: 1.0.0
description: A test application
type: application
"#
    }

    fn create_test_values_yaml() -> &'static str {
        r#"
name: my-app
replicas: 2
image:
  repository: nginx
  tag: "1.21"
service:
  port: 80
"#
    }

    fn create_test_deployment_template() -> &'static str {
        r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .Values.name }}
  namespace: default
spec:
  replicas: {{ .Values.replicas }}
  selector:
    matchLabels:
      app: {{ .Values.name }}
  template:
    metadata:
      labels:
        app: {{ .Values.name }}
    spec:
      containers:
      - name: {{ .Values.name }}
        image: {{ .Values.image.repository }}:{{ .Values.image.tag }}
---
apiVersion: v1
kind: Service
metadata:
  name: {{ .Values.name }}-service
spec:
  ports:
  - port: {{ .Values.service.port }}
  selector:
    app: {{ .Values.name }}
"#
    }

    #[test]
    fn test_analyze_chart() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_dir = temp_dir.path();

        // Create chart structure
        std::fs::write(chart_dir.join("Chart.yaml"), create_test_chart_yaml())?;
        std::fs::write(chart_dir.join("values.yaml"), create_test_values_yaml())?;

        let templates_dir = chart_dir.join("templates");
        std::fs::create_dir(&templates_dir)?;
        std::fs::write(
            templates_dir.join("deployment.yaml"),
            create_test_deployment_template(),
        )?;

        let analysis = analyze_chart(chart_dir)?;

        assert_eq!(analysis.chart_name, "test-app");
        assert_eq!(analysis.chart_version, "1.0.0");
        assert_eq!(analysis.values_file_count(), 1);

        let report = analysis.values_analyses.get("values.yaml").unwrap();
        assert_eq!(report.get_count("Deployment"), 1);
        assert_eq!(report.get_count("Service"), 1);
        assert_eq!(report.total_resources, 2);

        Ok(())
    }

    #[test]
    fn test_analyze_chart_with_multiple_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_dir = temp_dir.path();

        // Create chart structure
        std::fs::write(chart_dir.join("Chart.yaml"), create_test_chart_yaml())?;
        std::fs::write(chart_dir.join("values.yaml"), create_test_values_yaml())?;
        std::fs::write(
            chart_dir.join("values-prod.yaml"),
            "name: my-app\nreplicas: 5\nimage:\n  repository: nginx\n  tag: \"1.21\"\nservice:\n  port: 80",
        )?;

        let templates_dir = chart_dir.join("templates");
        std::fs::create_dir(&templates_dir)?;
        std::fs::write(
            templates_dir.join("deployment.yaml"),
            create_test_deployment_template(),
        )?;

        let analysis = analyze_chart(chart_dir)?;

        assert_eq!(analysis.values_file_count(), 2);

        // Both should have the same resource counts since we're counting types, not replicas
        for report in analysis.values_analyses.values() {
            assert_eq!(report.get_count("Deployment"), 1);
            assert_eq!(report.get_count("Service"), 1);
        }

        Ok(())
    }

    #[test]
    fn test_analyze_library_chart() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_dir = temp_dir.path();

        let library_chart = r#"
apiVersion: v2
name: common-lib
version: 1.0.0
description: A library chart
type: library
"#;

        std::fs::write(chart_dir.join("Chart.yaml"), library_chart)?;

        let analysis = analyze_chart(chart_dir)?;

        assert_eq!(analysis.chart_name, "common-lib");
        assert_eq!(analysis.values_file_count(), 0); // Library charts don't get analyzed for resources

        Ok(())
    }

    #[test]
    fn test_extract_resources_from_yaml() -> Result<()> {
        let yaml_content = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
  namespace: production
---
apiVersion: v1
kind: Service
metadata:
  name: test-service
"#;

        let resources = extract_resources_from_yaml(yaml_content)?;

        assert_eq!(resources.len(), 2);

        let deployment = &resources[0];
        assert_eq!(deployment.kind, "Deployment");
        assert_eq!(deployment.name, "test-deployment");
        assert_eq!(deployment.namespace, Some("production".to_string()));

        let service = &resources[1];
        assert_eq!(service.kind, "Service");
        assert_eq!(service.name, "test-service");
        assert_eq!(service.namespace, None);

        Ok(())
    }

    #[test]
    fn test_analyze_chart_no_templates() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_dir = temp_dir.path();

        // Create chart with no templates directory
        std::fs::write(chart_dir.join("Chart.yaml"), create_test_chart_yaml())?;
        std::fs::write(chart_dir.join("values.yaml"), create_test_values_yaml())?;

        let analysis = analyze_chart(chart_dir)?;

        assert_eq!(analysis.chart_name, "test-app");
        assert_eq!(analysis.values_file_count(), 1);

        let report = analysis.values_analyses.get("values.yaml").unwrap();
        assert_eq!(report.total_resources, 0);

        Ok(())
    }

    #[test]
    fn test_analyze_chart_no_values() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let chart_dir = temp_dir.path();

        // Create chart with no values file
        std::fs::write(chart_dir.join("Chart.yaml"), create_test_chart_yaml())?;

        let templates_dir = chart_dir.join("templates");
        std::fs::create_dir(&templates_dir)?;
        std::fs::write(
            templates_dir.join("deployment.yaml"),
            "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: static-name",
        )?;

        let analysis = analyze_chart(chart_dir)?;

        assert_eq!(analysis.values_file_count(), 1);
        assert!(analysis.values_analyses.contains_key("default"));

        Ok(())
    }

    #[tokio::test]
    #[async_backtrace::framed]
    async fn test_analyze_charts() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let charts_dir = temp_dir.path();

        // Create first chart
        let chart1_dir = charts_dir.join("chart1");
        std::fs::create_dir(&chart1_dir)?;
        std::fs::write(chart1_dir.join("Chart.yaml"), create_test_chart_yaml())?;

        // Create second chart
        let chart2_dir = charts_dir.join("chart2");
        std::fs::create_dir(&chart2_dir)?;
        let chart2_yaml = r#"
apiVersion: v2
name: another-app
version: 2.0.0
description: Another test application
type: application
"#;
        std::fs::write(chart2_dir.join("Chart.yaml"), chart2_yaml)?;

        // Create non-chart directory (should be ignored)
        let not_chart_dir = charts_dir.join("not-a-chart");
        std::fs::create_dir(&not_chart_dir)?;
        std::fs::write(not_chart_dir.join("readme.txt"), "Not a chart")?;

        let analyses = analyze_charts(charts_dir).await?;

        assert_eq!(analyses.len(), 2);

        // Should be sorted by name
        assert_eq!(analyses[0].chart_name, "another-app");
        assert_eq!(analyses[1].chart_name, "test-app");

        Ok(())
    }
}
