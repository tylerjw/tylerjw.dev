//! Report generation module
//!
//! Handles generating reports about Kubernetes resources found in Helm charts.

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Analysis report for a single Helm chart
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartAnalysis {
    /// Chart name
    pub chart_name: String,
    /// Chart version
    pub chart_version: String,
    /// Chart directory path
    pub chart_path: PathBuf,
    /// Analysis results for each values file
    pub values_analyses: HashMap<String, ResourceReport>,
    /// Chart metadata
    pub metadata: ChartMetadata,
}

/// Resource count report for a specific values file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceReport {
    /// Values file name
    pub values_file: String,
    /// Count of each Kubernetes resource type
    pub resource_counts: HashMap<String, u32>,
    /// List of resource names by type
    pub resources: HashMap<String, Vec<ResourceInfo>>,
    /// Total number of resources
    pub total_resources: u32,
}

/// Information about a specific Kubernetes resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceInfo {
    /// Resource name
    pub name: String,
    /// Resource namespace (if applicable)
    pub namespace: Option<String>,
    /// Template file that generated this resource
    pub source_template: PathBuf,
}

/// Chart metadata for reporting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartMetadata {
    /// Chart description
    pub description: Option<String>,
    /// Chart API version
    pub api_version: String,
    /// Chart type (application or library)
    pub chart_type: Option<String>,
    /// Chart keywords
    pub keywords: Option<Vec<String>>,
    /// Number of dependencies
    pub dependency_count: u32,
}

impl ChartAnalysis {
    /// Create a new chart analysis
    pub fn new(
        chart_name: String,
        chart_version: String,
        chart_path: PathBuf,
        metadata: crate::chart::ChartMetadata,
    ) -> Self {
        Self {
            chart_name,
            chart_version,
            chart_path,
            values_analyses: HashMap::new(),
            metadata: ChartMetadata {
                description: metadata.description,
                api_version: metadata.api_version,
                chart_type: metadata.chart_type,
                keywords: metadata.keywords,
                dependency_count: metadata
                    .dependencies
                    .as_ref()
                    .map_or(0, |deps| deps.len() as u32),
            },
        }
    }

    /// Add a resource report for a specific values file
    pub fn add_resource_report(&mut self, values_file: String, report: ResourceReport) {
        self.values_analyses.insert(values_file, report);
    }

    /// Get the total number of values files analyzed
    pub fn values_file_count(&self) -> usize {
        self.values_analyses.len()
    }

    /// Get a summary of all resources across all values files
    pub fn get_resource_summary(&self) -> HashMap<String, u32> {
        let mut summary = HashMap::new();

        for report in self.values_analyses.values() {
            for (resource_type, count) in &report.resource_counts {
                *summary.entry(resource_type.clone()).or_insert(0) += count;
            }
        }

        summary
    }

    /// Export to JSON format
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize chart analysis to JSON")
    }

    /// Export to YAML format
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize chart analysis to YAML")
    }

    /// Save to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P, format: ReportFormat) -> Result<()> {
        let path = path.as_ref();
        let content = match format {
            ReportFormat::Json => self.to_json()?,
            ReportFormat::Yaml => self.to_yaml()?,
        };

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write report to {}", path.display()))?;

        Ok(())
    }
}

impl ResourceReport {
    /// Create a new empty resource report
    pub fn new(values_file: String) -> Self {
        Self {
            values_file,
            resource_counts: HashMap::new(),
            resources: HashMap::new(),
            total_resources: 0,
        }
    }

    /// Add a resource to the report
    pub fn add_resource(&mut self, resource_type: String, resource: ResourceInfo) {
        *self
            .resource_counts
            .entry(resource_type.clone())
            .or_insert(0) += 1;
        self.resources
            .entry(resource_type)
            .or_default()
            .push(resource);
        self.total_resources += 1;
    }

    /// Get the count of a specific resource type
    pub fn get_count(&self, resource_type: &str) -> u32 {
        self.resource_counts
            .get(resource_type)
            .copied()
            .unwrap_or(0)
    }

    /// Get all resource types found
    pub fn get_resource_types(&self) -> Vec<String> {
        let mut types: Vec<_> = self.resource_counts.keys().cloned().collect();
        types.sort();
        types
    }

    /// Check if any resources were found
    pub fn has_resources(&self) -> bool {
        self.total_resources > 0
    }
}

impl ResourceInfo {
    /// Create a new resource info
    pub fn new(name: String, namespace: Option<String>, source_template: PathBuf) -> Self {
        Self {
            name,
            namespace,
            source_template,
        }
    }

    /// Get the full resource identifier (namespace/name or just name)
    pub fn full_name(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{}/{}", ns, self.name),
            None => self.name.clone(),
        }
    }
}

/// Output format for reports
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl ReportFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::Yaml => "yaml",
        }
    }

    /// Parse format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ReportFormat::Json),
            "yaml" | "yml" => Some(ReportFormat::Yaml),
            _ => None,
        }
    }
}

/// Generate a summary table in markdown format
pub fn generate_markdown_summary(analyses: &[ChartAnalysis]) -> String {
    let mut output = String::new();

    output.push_str("# Helm Chart Analysis Report\n\n");

    for analysis in analyses {
        output.push_str(&format!(
            "## Chart: {} ({})\n\n",
            analysis.chart_name, analysis.chart_version
        ));

        if let Some(description) = &analysis.metadata.description {
            output.push_str(&format!("**Description:** {}\n\n", description));
        }

        output.push_str(&format!("**Path:** {}\n\n", analysis.chart_path.display()));
        output.push_str(&format!(
            "**Dependencies:** {}\n\n",
            analysis.metadata.dependency_count
        ));

        // Resource summary table
        output.push_str("### Resource Summary\n\n");
        output.push_str("| Values File | ");

        // Get all resource types across all values files
        let mut all_types = std::collections::HashSet::new();
        for report in analysis.values_analyses.values() {
            all_types.extend(report.resource_counts.keys().cloned());
        }
        let mut all_types: Vec<_> = all_types.into_iter().collect();
        all_types.sort();

        for resource_type in &all_types {
            output.push_str(&format!("{} | ", resource_type));
        }
        output.push_str("Total |\n");

        // Table separator
        output.push('|');
        for _ in 0..=all_types.len() {
            output.push_str("---|");
        }
        output.push('\n');

        // Data rows
        for (values_file, report) in &analysis.values_analyses {
            output.push_str(&format!("| {} | ", values_file));

            for resource_type in &all_types {
                let count = report.get_count(resource_type);
                output.push_str(&format!("{} | ", count));
            }
            output.push_str(&format!("{} |\n", report.total_resources));
        }

        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use test_log::test;

    fn create_test_chart_metadata() -> crate::chart::ChartMetadata {
        crate::chart::ChartMetadata {
            name: "test-chart".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test chart".to_string()),
            api_version: "v2".to_string(),
            chart_type: Some("application".to_string()),
            keywords: Some(vec!["test".to_string(), "example".to_string()]),
            maintainers: None,
            dependencies: Some(vec![crate::chart::Dependency {
                name: "postgresql".to_string(),
                version: "11.6.21".to_string(),
                repository: Some("https://charts.bitnami.com/bitnami".to_string()),
                condition: None,
            }]),
        }
    }

    #[test]
    fn test_create_chart_analysis() -> Result<()> {
        let metadata = create_test_chart_metadata();
        let analysis = ChartAnalysis::new(
            "test-chart".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/charts/test-chart"),
            metadata,
        );

        assert_eq!(analysis.chart_name, "test-chart");
        assert_eq!(analysis.chart_version, "1.0.0");
        assert_eq!(analysis.metadata.dependency_count, 1);
        assert_eq!(analysis.values_file_count(), 0);

        Ok(())
    }

    #[test]
    fn test_resource_report() -> Result<()> {
        let mut report = ResourceReport::new("values.yaml".to_string());
        assert!(!report.has_resources());
        assert_eq!(report.total_resources, 0);

        let resource = ResourceInfo::new(
            "test-deployment".to_string(),
            Some("default".to_string()),
            PathBuf::from("templates/deployment.yaml"),
        );

        report.add_resource("Deployment".to_string(), resource);

        assert!(report.has_resources());
        assert_eq!(report.total_resources, 1);
        assert_eq!(report.get_count("Deployment"), 1);
        assert_eq!(report.get_count("Service"), 0);

        let types = report.get_resource_types();
        assert_eq!(types, vec!["Deployment"]);

        Ok(())
    }

    #[test]
    fn test_resource_info() -> Result<()> {
        let resource = ResourceInfo::new(
            "test-app".to_string(),
            Some("production".to_string()),
            PathBuf::from("templates/deployment.yaml"),
        );

        assert_eq!(resource.full_name(), "production/test-app");

        let resource_no_ns = ResourceInfo::new(
            "cluster-role".to_string(),
            None,
            PathBuf::from("templates/rbac.yaml"),
        );

        assert_eq!(resource_no_ns.full_name(), "cluster-role");

        Ok(())
    }

    #[test]
    fn test_chart_analysis_with_reports() -> Result<()> {
        let metadata = create_test_chart_metadata();
        let mut analysis = ChartAnalysis::new(
            "test-chart".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/charts/test-chart"),
            metadata,
        );

        let mut report = ResourceReport::new("values.yaml".to_string());
        report.add_resource(
            "Deployment".to_string(),
            ResourceInfo::new(
                "app".to_string(),
                Some("default".to_string()),
                PathBuf::from("templates/deployment.yaml"),
            ),
        );
        report.add_resource(
            "Service".to_string(),
            ResourceInfo::new(
                "app".to_string(),
                Some("default".to_string()),
                PathBuf::from("templates/service.yaml"),
            ),
        );

        analysis.add_resource_report("values.yaml".to_string(), report);

        assert_eq!(analysis.values_file_count(), 1);

        let summary = analysis.get_resource_summary();
        assert_eq!(summary.get("Deployment"), Some(&1));
        assert_eq!(summary.get("Service"), Some(&1));

        Ok(())
    }

    #[test]
    fn test_report_format() -> Result<()> {
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Yaml.extension(), "yaml");

        assert_eq!(
            ReportFormat::from_extension("json"),
            Some(ReportFormat::Json)
        );
        assert_eq!(
            ReportFormat::from_extension("yaml"),
            Some(ReportFormat::Yaml)
        );
        assert_eq!(
            ReportFormat::from_extension("yml"),
            Some(ReportFormat::Yaml)
        );
        assert_eq!(ReportFormat::from_extension("txt"), None);

        Ok(())
    }

    #[test]
    fn test_json_serialization() -> Result<()> {
        let metadata = create_test_chart_metadata();
        let mut analysis = ChartAnalysis::new(
            "test-chart".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/charts/test-chart"),
            metadata,
        );

        let mut report = ResourceReport::new("values.yaml".to_string());
        report.add_resource(
            "Deployment".to_string(),
            ResourceInfo::new(
                "app".to_string(),
                None,
                PathBuf::from("templates/deployment.yaml"),
            ),
        );

        analysis.add_resource_report("values.yaml".to_string(), report);

        let json = analysis.to_json()?;
        assert!(json.contains("test-chart"));
        assert!(json.contains("Deployment"));

        // Test deserialization
        let deserialized: ChartAnalysis = serde_json::from_str(&json)?;
        assert_eq!(deserialized.chart_name, "test-chart");

        Ok(())
    }

    #[test]
    fn test_save_to_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let metadata = create_test_chart_metadata();
        let analysis = ChartAnalysis::new(
            "test-chart".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/charts/test-chart"),
            metadata,
        );

        let json_path = temp_dir.path().join("report.json");
        analysis.save_to_file(&json_path, ReportFormat::Json)?;

        assert!(json_path.exists());

        let content = std::fs::read_to_string(&json_path)?;
        assert!(content.contains("test-chart"));

        Ok(())
    }

    #[test]
    fn test_generate_markdown_summary() -> Result<()> {
        let metadata = create_test_chart_metadata();
        let mut analysis = ChartAnalysis::new(
            "test-chart".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/charts/test-chart"),
            metadata,
        );

        let mut report = ResourceReport::new("values.yaml".to_string());
        report.add_resource(
            "Deployment".to_string(),
            ResourceInfo::new(
                "app".to_string(),
                None,
                PathBuf::from("templates/deployment.yaml"),
            ),
        );

        analysis.add_resource_report("values.yaml".to_string(), report);

        let markdown = generate_markdown_summary(&[analysis]);

        assert!(markdown.contains("# Helm Chart Analysis Report"));
        assert!(markdown.contains("## Chart: test-chart (1.0.0)"));
        assert!(markdown.contains("| values.yaml |"));
        assert!(markdown.contains("Deployment"));

        Ok(())
    }
}
