//! Chart analysis orchestration module
//!
//! This module ties together chart parsing, template rendering, and resource
//! counting to produce comprehensive analysis reports.

use anyhow::{Context, Result, ensure};
use serde_yaml::Value as YamlValue;
use std::path::Path;

use crate::chart::{ChartMetadata, find_chart_file};
use crate::report::{ChartAnalysis, ResourceInfo, ResourceReport};
use crate::template::{Template, Values, find_template_files, find_values_files};

/// Analyze a single Helm chart directory
pub fn analyze_chart<P: AsRef<Path>>(chart_dir: P) -> Result<ChartAnalysis> {
    let chart_dir = chart_dir.as_ref();

    // Find and parse Chart.yaml
    let chart_file = find_chart_file(chart_dir)
        .with_context(|| format!("Chart analysis failed for {}", chart_dir.display()))?;

    let chart_metadata =
        ChartMetadata::load_from_file(&chart_file).context("Failed to load chart metadata")?;

    chart_metadata
        .validate()
        .context("Chart metadata validation failed")?;

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
        find_template_files(&templates_dir)
            .with_context(|| format!("Failed to find templates in {}", templates_dir.display()))?
    } else {
        Vec::new()
    };

    // Load templates
    let mut templates = Vec::new();
    for template_path in template_files {
        let template = Template::load_from_file(&template_path)
            .with_context(|| format!("Failed to load template {}", template_path.display()))?;

        if !template.is_empty_template() {
            templates.push(template);
        }
    }

    // Find values files
    let values_files = find_values_files(chart_dir)
        .with_context(|| format!("Failed to find values files in {}", chart_dir.display()))?;

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

        let values = Values::load_from_file(&values_path)
            .with_context(|| format!("Failed to load values file {}", values_path.display()))?;

        let resource_report = analyze_with_values(&templates, &values)
            .with_context(|| format!("Analysis failed for values file {}", values_file_name))?;

        analysis.add_resource_report(values_file_name, resource_report);
    }

    // If no values files were found, analyze with empty values
    if analysis.values_file_count() == 0 {
        let empty_values = Values::empty();
        let resource_report = analyze_with_values(&templates, &empty_values)
            .context("Analysis failed with empty values")?;

        analysis.add_resource_report("default".to_string(), resource_report);
    }

    Ok(analysis)
}

/// Analyze templates with specific values to count resources
fn analyze_with_values(templates: &[Template], values: &Values) -> Result<ResourceReport> {
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
            .with_context(|| format!("Failed to render template {}", template.path.display()))?;

        let resources =
            extract_resources_from_yaml(&rendered.rendered_content).with_context(|| {
                format!(
                    "Failed to extract resources from template {}",
                    template.path.display()
                )
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
fn extract_resources_from_yaml(yaml_content: &str) -> Result<Vec<ExtractedResource>> {
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
fn extract_resource_info(yaml: &YamlValue) -> Result<Option<ExtractedResource>> {
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
pub async fn analyze_charts<P: AsRef<Path>>(charts_dir: P) -> Result<Vec<ChartAnalysis>> {
    let charts_dir = charts_dir.as_ref();

    ensure!(
        charts_dir.exists(),
        "Charts directory does not exist: {}",
        charts_dir.display()
    );

    let mut analyses = Vec::new();
    let mut handles = Vec::new();

    // Find all chart directories
    for entry in std::fs::read_dir(charts_dir)
        .with_context(|| format!("Failed to read charts directory {}", charts_dir.display()))?
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
