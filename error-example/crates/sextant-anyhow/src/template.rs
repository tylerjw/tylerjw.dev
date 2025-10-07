//! Template rendering module
//!
//! Handles rendering Helm templates with values to determine what Kubernetes
//! resources would be created.

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use serde_yaml;

use std::path::{Path, PathBuf};

/// Represents a Helm template file
#[derive(Debug, Clone)]
pub struct Template {
    /// Template file path
    pub path: PathBuf,
    /// Template content
    pub content: String,
}

/// Values loaded from values.yaml files
#[derive(Debug, Clone)]
pub struct Values {
    /// The values data
    pub data: Value,
    /// Source file path
    pub source: PathBuf,
}

/// Rendered template output
#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    /// Original template path
    pub template_path: PathBuf,
    /// Rendered YAML content
    pub rendered_content: String,
    /// Values file used for rendering
    pub values_source: PathBuf,
}

impl Template {
    /// Load a template from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read template file {}", path.display()))?;

        Ok(Template {
            path: path.to_path_buf(),
            content,
        })
    }

    /// Check if this template would produce any output
    pub fn is_empty_template(&self) -> bool {
        self.content.trim().is_empty() || self.content.trim().starts_with("{{- if false")
    }

    /// Simple template rendering (basic variable substitution)
    /// This is a simplified version - real Helm uses Go templates
    pub fn render(&self, values: &Values) -> Result<RenderedTemplate> {
        let mut rendered = self.content.clone();

        // Simple variable substitution for common patterns
        rendered = self
            .substitute_variables(&rendered, &values.data)
            .with_context(|| format!("Failed to render template {}", self.path.display()))?;

        // Remove Helm template comments and empty lines
        rendered = self.clean_rendered_output(&rendered);

        Ok(RenderedTemplate {
            template_path: self.path.clone(),
            rendered_content: rendered,
            values_source: values.source.clone(),
        })
    }

    /// Substitute template variables with values
    fn substitute_variables(&self, content: &str, values: &Value) -> Result<String> {
        let mut result = content.to_string();

        // Handle nested values recursively
        self.substitute_nested_values(&mut result, values, "Values")?;

        // Handle conditional blocks (simplified)
        result = self.handle_conditionals(&result, values)?;

        Ok(result)
    }

    /// Recursively substitute nested values
    fn substitute_nested_values(
        &self,
        content: &mut String,
        values: &Value,
        prefix: &str,
    ) -> Result<()> {
        match values {
            Value::Object(obj) => {
                for (key, value) in obj {
                    let current_path = format!("{}.{}", prefix, key);

                    // Handle direct substitution for this key
                    let patterns = vec![
                        format!("{{{{ .{} }}}}", current_path),
                        format!("{{{{.{}}}}}", current_path),
                        format!("{{{{ .{} | quote }}}}", current_path),
                    ];

                    for pattern in patterns {
                        if let Some(replacement) = self.value_to_string(value) {
                            *content = content.replace(&pattern, &replacement);
                        }
                    }

                    // Recursively handle nested objects
                    if value.is_object() || value.is_array() {
                        self.substitute_nested_values(content, value, &current_path)?;
                    }
                }
            }
            Value::Array(arr) => {
                for (index, value) in arr.iter().enumerate() {
                    let current_path = format!("{}[{}]", prefix, index);
                    if value.is_object() || value.is_array() {
                        self.substitute_nested_values(content, value, &current_path)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Convert a JSON value to string for template substitution
    fn value_to_string(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Array(_) | Value::Object(_) => {
                // For complex types, serialize as YAML
                serde_yaml::to_string(value).ok()
            }
            Value::Null => Some("".to_string()),
        }
    }

    /// Handle simple conditional blocks
    fn handle_conditionals(&self, content: &str, _values: &Value) -> Result<String> {
        // This is a very simplified conditional handler
        // Real Helm uses Go's text/template engine
        let mut result = content.to_string();

        // Remove {{- if false }} blocks
        while let Some(start) = result.find("{{- if false }}") {
            if let Some(end) = result[start..].find("{{- end }}") {
                let end_pos = start + end + "{{- end }}".len();
                result.replace_range(start..end_pos, "");
            } else {
                break;
            }
        }

        Ok(result)
    }

    /// Clean up rendered output by removing comments and empty lines
    fn clean_rendered_output(&self, content: &str) -> String {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with('#')
                    && !trimmed.starts_with("{{")
                    && !trimmed.starts_with("---")
                    || (trimmed == "---" && !line.trim().is_empty())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Values {
    /// Load values from a YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read values file {}", path.display()))?;

        let data: Value = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse values file {}", path.display()))?;

        Ok(Values {
            data,
            source: path.to_path_buf(),
        })
    }

    /// Create empty values
    pub fn empty() -> Self {
        Values {
            data: Value::Object(serde_json::Map::new()),
            source: PathBuf::from("empty"),
        }
    }

    /// Merge with another values file (other takes precedence)
    pub fn merge(&self, other: &Values) -> Result<Values> {
        let merged_data = Self::merge_json_values(&self.data, &other.data);

        Ok(Values {
            data: merged_data,
            source: other.source.clone(), // Use the source of the overriding values
        })
    }

    /// Merge two JSON values recursively
    fn merge_json_values(base: &Value, override_val: &Value) -> Value {
        match (base, override_val) {
            (Value::Object(base_map), Value::Object(override_map)) => {
                let mut merged = base_map.clone();
                for (key, value) in override_map {
                    merged.insert(
                        key.clone(),
                        if let Some(base_value) = base_map.get(key) {
                            Self::merge_json_values(base_value, value)
                        } else {
                            value.clone()
                        },
                    );
                }
                Value::Object(merged)
            }
            _ => override_val.clone(),
        }
    }
}

/// Find all template files in a templates directory
pub fn find_template_files<P: AsRef<Path>>(templates_dir: P) -> Result<Vec<PathBuf>> {
    let templates_dir = templates_dir.as_ref();

    ensure!(
        templates_dir.exists(),
        "Templates directory does not exist: {}",
        templates_dir.display()
    );

    let mut template_files = Vec::new();

    for entry in std::fs::read_dir(templates_dir).with_context(|| {
        format!(
            "Failed to read templates directory {}",
            templates_dir.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    // Skip test files and notes
                    if let Some(file_name) = path.file_name() {
                        let file_name_str = file_name.to_string_lossy();
                        if !file_name_str.contains("test") && !file_name_str.contains("NOTES") {
                            template_files.push(path);
                        }
                    }
                }
            }
        }
    }

    template_files.sort();
    Ok(template_files)
}

/// Find all values files in a chart directory
pub fn find_values_files<P: AsRef<Path>>(chart_dir: P) -> Result<Vec<PathBuf>> {
    let chart_dir = chart_dir.as_ref();
    let mut values_files = Vec::new();

    // Look for values.yaml and values.yml
    for filename in &["values.yaml", "values.yml"] {
        let path = chart_dir.join(filename);
        if path.exists() {
            values_files.push(path);
        }
    }

    // Look for additional values files (values-*.yaml)
    for entry in std::fs::read_dir(chart_dir)
        .with_context(|| format!("Failed to read chart directory {}", chart_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("values-")
                    && (file_name_str.ends_with(".yaml") || file_name_str.ends_with(".yml"))
                {
                    values_files.push(path);
                }
            }
        }
    }

    values_files.sort();
    Ok(values_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use test_log::test;

    fn create_test_template() -> String {
        r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ .Values.name }}
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
"#
        .trim()
        .to_string()
    }

    fn create_test_values() -> String {
        r#"
name: test-app
replicas: 3
image:
  repository: nginx
  tag: 1.21
"#
        .trim()
        .to_string()
    }

    #[test]
    fn test_load_template_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_path = temp_dir.path().join("deployment.yaml");

        std::fs::write(&template_path, create_test_template())?;

        let template = Template::load_from_file(&template_path)?;
        assert_eq!(template.path, template_path);
        assert!(template.content.contains("{{ .Values.name }}"));

        Ok(())
    }

    #[test]
    fn test_load_values_from_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let values_path = temp_dir.path().join("values.yaml");

        std::fs::write(&values_path, create_test_values())?;

        let values = Values::load_from_file(&values_path)?;
        assert_eq!(values.source, values_path);

        // Check that values were parsed correctly
        let name = values.data.get("name").unwrap().as_str().unwrap();
        assert_eq!(name, "test-app");

        Ok(())
    }

    #[test]
    fn test_render_template() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let template_path = temp_dir.path().join("deployment.yaml");
        let values_path = temp_dir.path().join("values.yaml");

        std::fs::write(&template_path, create_test_template())?;
        std::fs::write(&values_path, create_test_values())?;

        let template = Template::load_from_file(&template_path)?;
        let values = Values::load_from_file(&values_path)?;

        let rendered = template.render(&values)?;

        assert!(rendered.rendered_content.contains("name: test-app"));
        assert!(rendered.rendered_content.contains("replicas: 3"));
        assert!(rendered.rendered_content.contains("image: nginx:1.21"));

        Ok(())
    }

    #[test]
    fn test_find_template_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let templates_dir = temp_dir.path().join("templates");
        std::fs::create_dir(&templates_dir)?;

        // Create template files
        std::fs::write(
            templates_dir.join("deployment.yaml"),
            create_test_template(),
        )?;
        std::fs::write(templates_dir.join("service.yaml"), "kind: Service")?;
        std::fs::write(templates_dir.join("NOTES.txt"), "Notes file")?; // Should be ignored
        std::fs::write(templates_dir.join("test-deployment.yaml"), "test")?; // Should be ignored

        let template_files = find_template_files(&templates_dir)?;

        assert_eq!(template_files.len(), 2);
        assert!(
            template_files
                .iter()
                .any(|p| p.file_name().unwrap() == "deployment.yaml")
        );
        assert!(
            template_files
                .iter()
                .any(|p| p.file_name().unwrap() == "service.yaml")
        );

        Ok(())
    }

    #[test]
    fn test_find_values_files() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create values files
        std::fs::write(temp_dir.path().join("values.yaml"), create_test_values())?;
        std::fs::write(temp_dir.path().join("values-prod.yaml"), "env: prod")?;
        std::fs::write(temp_dir.path().join("values-dev.yml"), "env: dev")?;

        let values_files = find_values_files(temp_dir.path())?;

        assert_eq!(values_files.len(), 3);
        assert!(
            values_files
                .iter()
                .any(|p| p.file_name().unwrap() == "values.yaml")
        );
        assert!(
            values_files
                .iter()
                .any(|p| p.file_name().unwrap() == "values-prod.yaml")
        );
        assert!(
            values_files
                .iter()
                .any(|p| p.file_name().unwrap() == "values-dev.yml")
        );

        Ok(())
    }

    #[test]
    fn test_merge_values() -> Result<()> {
        let temp_dir = TempDir::new()?;

        let base_values_content = r#"
name: test-app
replicas: 1
image:
  repository: nginx
  tag: latest
"#;

        let override_values_content = r#"
replicas: 3
image:
  tag: "1.21"
env: production
"#;

        let base_path = temp_dir.path().join("values.yaml");
        let override_path = temp_dir.path().join("values-prod.yaml");

        std::fs::write(&base_path, base_values_content)?;
        std::fs::write(&override_path, override_values_content)?;

        let base_values = Values::load_from_file(&base_path)?;
        let override_values = Values::load_from_file(&override_path)?;

        let merged = base_values.merge(&override_values)?;

        // Check merged values
        assert_eq!(
            merged.data.get("name").unwrap().as_str().unwrap(),
            "test-app"
        );
        assert_eq!(merged.data.get("replicas").unwrap().as_i64().unwrap(), 3);
        assert_eq!(
            merged.data.get("env").unwrap().as_str().unwrap(),
            "production"
        );

        // Check nested merge
        let image = merged.data.get("image").unwrap().as_object().unwrap();
        assert_eq!(image.get("repository").unwrap().as_str().unwrap(), "nginx");
        assert_eq!(image.get("tag").unwrap().as_str().unwrap(), "1.21");

        Ok(())
    }

    #[test]
    fn test_is_empty_template() -> Result<()> {
        let template = Template {
            path: PathBuf::from("test.yaml"),
            content: "".to_string(),
        };
        assert!(template.is_empty_template());

        let template = Template {
            path: PathBuf::from("test.yaml"),
            content: "{{- if false }}\nsome content\n{{- end }}".to_string(),
        };
        assert!(template.is_empty_template());

        let template = Template {
            path: PathBuf::from("test.yaml"),
            content: "kind: Deployment".to_string(),
        };
        assert!(!template.is_empty_template());

        Ok(())
    }
}
