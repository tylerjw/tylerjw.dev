# Sextant (thiserror) ⚓

A Helm Chart Resource Analyzer demonstrating error handling with [`thiserror`](https://docs.rs/thiserror/latest/thiserror/).

## Key Characteristics

- **Structured error types** - Each module defines domain-specific error enums
- **Pattern matching support** - Consumers can match on specific error variants
- **Zero-cost abstractions** - No runtime overhead over manual implementations
- **Source chaining** - Built-in error cause tracking with `#[source]`

## Usage

```bash
# Analyze single chart
cargo run -- chart path/to/chart

# Analyze multiple charts
cargo run -- charts path/to/charts-dir --output reports/
```

## Error Handling Approach

### Type Definition
```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
}
```

### Hierarchical Error Structure
```rust
// Root error aggregates module errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Chart(#[from] chart::ChartError),

    #[error(transparent)]
    Analysis(#[from] analyzer::AnalysisError),
}

// Module-specific errors
#[derive(thiserror::Error, Debug)]
pub enum ChartError {
    #[error("No Chart.yaml found in {path}")]
    ChartFileNotFound { path: PathBuf },

    #[error("Chart name cannot be empty")]
    EmptyName,
}
```

### Error Conversion
```rust
// Automatic conversion with #[from]
let data: Value = serde_yaml::from_str(&content)?;  // Auto-converts to ChartError::Yaml

// Manual conversion with context
let chart_file = find_chart_file(chart_dir)
    .map_err(|source| AnalysisError::ChartAnalysisFailed {
        path: chart_dir.to_path_buf(),
        source,
    })?;
```

### Pattern Matching
```rust
match analyze_chart(path) {
    Ok(analysis) => process(analysis),
    Err(Error::Chart(ChartError::ChartFileNotFound { path })) => {
        eprintln!("Missing Chart.yaml at: {}", path.display());
    }
    Err(Error::Chart(ChartError::EmptyName)) => {
        eprintln!("Chart must have a name");
    }
    Err(other) => eprintln!("Other error: {}", other),
}
```

### Error Output
```
Error: Failed to load chart metadata: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
  Caused by: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
  Caused by: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
  Caused by: mapping values are not allowed in this context at line 2 column 10
```

## Advantages

- **Type safety** - Compile-time error type checking
- **Pattern matching** - Handle specific error conditions differently
- **Structured data** - Errors carry relevant context (paths, names, etc.)
- **Library friendly** - Perfect for public APIs

## Trade-offs

- **More verbose** - Requires defining error types upfront
- **Manual conversions** - Some `map_err` calls needed for context
- **Maintenance overhead** - Error types become part of public API

## Best For

- Library APIs where consumers need to handle specific error types
- Applications requiring structured error handling
- When compile-time error type safety is important
- Code that benefits from pattern matching on errors
