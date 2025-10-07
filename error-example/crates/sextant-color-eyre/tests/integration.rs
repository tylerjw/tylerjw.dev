//! Integration tests for the sextant binary

use color_eyre::Result;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use test_log::test;

/// Get the path to the sextant binary
fn sextant_binary() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.push("sextant-color-eyre");
    path
}

/// Create a test chart directory with basic files
fn create_test_chart(temp_dir: &TempDir, name: &str) -> std::io::Result<PathBuf> {
    let chart_dir = temp_dir.path().join(name);
    std::fs::create_dir(&chart_dir)?;

    // Chart.yaml
    let chart_yaml = format!(
        r#"apiVersion: v2
name: {}
version: 1.0.0
description: A test chart
type: application
"#,
        name
    );
    std::fs::write(chart_dir.join("Chart.yaml"), chart_yaml)?;

    // values.yaml
    let values_yaml = r#"name: test-app
replicas: 2
image:
  repository: nginx
  tag: "1.21"
service:
  port: 80
"#;
    std::fs::write(chart_dir.join("values.yaml"), values_yaml)?;

    // values-prod.yaml
    let values_prod_yaml = r#"name: test-app
replicas: 5
image:
  repository: nginx
  tag: "1.21"
service:
  port: 80
"#;
    std::fs::write(chart_dir.join("values-prod.yaml"), values_prod_yaml)?;

    // templates directory
    let templates_dir = chart_dir.join("templates");
    std::fs::create_dir(&templates_dir)?;

    // deployment.yaml template
    let deployment_template = r#"apiVersion: apps/v1
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
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ .Values.name }}-config
data:
  app.conf: |
    port={{ .Values.service.port }}
"#;
    std::fs::write(templates_dir.join("deployment.yaml"), deployment_template)?;

    Ok(chart_dir)
}

#[test]
fn test_sextant_chart_command_json_output() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let chart_dir = create_test_chart(&temp_dir, "test-chart")?;

    let output = Command::new(sextant_binary())
        .arg("chart")
        .arg(&chart_dir)
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute sextant");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Check that JSON was output
    assert!(stdout.contains("test-chart"));
    assert!(stdout.contains("\"chart_name\""));
    assert!(stdout.contains("\"Deployment\""));
    assert!(stdout.contains("\"Service\""));
    assert!(stdout.contains("\"ConfigMap\""));

    // Check summary in stderr
    assert!(stderr.contains("Chart Summary"));
    assert!(stderr.contains("test-chart (1.0.0)"));

    Ok(())
}

#[test]
fn test_sextant_chart_command_yaml_output() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let chart_dir = create_test_chart(&temp_dir, "test-chart")?;

    let output = Command::new(sextant_binary())
        .arg("chart")
        .arg(&chart_dir)
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("Failed to execute sextant");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout)?;

    // Check that YAML was output
    assert!(stdout.contains("chart_name: test-chart"));
    assert!(stdout.contains("chart_version: 1.0.0"));

    Ok(())
}

#[test]
fn test_sextant_chart_command_with_output_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let chart_dir = create_test_chart(&temp_dir, "test-chart")?;
    let output_file = temp_dir.path().join("report.json");

    let output = Command::new(sextant_binary())
        .arg("chart")
        .arg(&chart_dir)
        .arg("--output")
        .arg(&output_file)
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute sextant");

    assert!(output.status.success(), "Command failed: {:?}", output);

    // Check that file was created
    assert!(output_file.exists());

    let content = std::fs::read_to_string(&output_file)?;
    assert!(content.contains("test-chart"));
    assert!(content.contains("Deployment"));

    Ok(())
}

#[test]
fn test_sextant_charts_command() -> Result<()> {
    let temp_dir = TempDir::new()?;

    // Create multiple charts
    create_test_chart(&temp_dir, "chart1")?;
    create_test_chart(&temp_dir, "chart2")?;

    // Create a non-chart directory (should be ignored)
    let not_chart = temp_dir.path().join("not-a-chart");
    std::fs::create_dir(&not_chart)?;
    std::fs::write(not_chart.join("readme.txt"), "Not a chart")?;

    let output = Command::new(sextant_binary())
        .arg("charts")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute sextant");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    // Should contain both charts
    assert!(stdout.contains("chart1"));
    assert!(stdout.contains("chart2"));

    // Should show summary
    assert!(stderr.contains("Charts analyzed: 2"));

    Ok(())
}

#[test]
fn test_sextant_charts_command_with_summary() -> Result<()> {
    let temp_dir = TempDir::new()?;
    create_test_chart(&temp_dir, "test-chart")?;

    let output = Command::new(sextant_binary())
        .arg("charts")
        .arg(temp_dir.path())
        .arg("--format")
        .arg("json")
        .arg("--summary")
        .output()
        .expect("Failed to execute sextant");

    assert!(output.status.success(), "Command failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout)?;

    // Should contain markdown summary
    assert!(stdout.contains("# Helm Chart Analysis Report"));
    assert!(stdout.contains("## Chart: test-chart"));
    assert!(stdout.contains("| Values File |"));

    Ok(())
}

#[test]
fn test_sextant_charts_command_with_output_dir() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let charts_dir = temp_dir.path().join("charts");
    std::fs::create_dir(&charts_dir)?;

    // Create a proper test chart in the charts directory
    let chart_temp_dir = TempDir::new()?;
    create_test_chart(&chart_temp_dir, "test-chart")?;

    // Copy the chart to the charts directory
    let src_chart = chart_temp_dir.path().join("test-chart");
    let dst_chart = charts_dir.join("test-chart");

    fn copy_dir_recursively(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_dir_recursively(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    copy_dir_recursively(&src_chart, &dst_chart)?;

    let output_dir = temp_dir.path().join("output");

    let _output = Command::new(sextant_binary())
        .arg("charts")
        .arg(&charts_dir)
        .arg("--output")
        .arg(&output_dir)
        .arg("--format")
        .arg("json")
        .arg("--summary")
        .output()
        .expect("Failed to execute sextant");

    // Output dir should be created even if no charts found
    assert!(output_dir.exists());

    Ok(())
}

#[test]
fn test_sextant_invalid_chart_directory() {
    let output = Command::new(sextant_binary())
        .arg("chart")
        .arg("/nonexistent/directory")
        .output()
        .expect("Failed to execute sextant");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Chart.yaml"));
}

#[test]
fn test_sextant_invalid_format() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let chart_dir = create_test_chart(&temp_dir, "test-chart")?;

    let output = Command::new(sextant_binary())
        .arg("chart")
        .arg(&chart_dir)
        .arg("--format")
        .arg("xml")
        .output()
        .expect("Failed to execute sextant");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported format"));

    Ok(())
}
