# Rust Error Handling Library Comparison

This repository demonstrates error handling approaches across four popular Rust crates through practical examples.

## Libraries

- [`sextant-anyhow`](./crates/sextant-anyhow/) - `anyhow` for flexible error handling
- [`sextant-thiserror`](./crates/sextant-thiserror/) - `thiserror` for structured errors
- [`sextant-snafu`](./crates/sextant-snafu/) - `snafu` for context-rich errors
- [`sextant-color-eyre`](./crates/sextant-color-eyre/) - `color-eyre` for enhanced display

## Error Type Definition

### `anyhow` - Dynamic Errors

No custom error types required. Uses type alias and context extension:

```rust
pub type Result<T> = anyhow::Result<T>;

// Usage
let data = parse_file(path)
    .with_context(|| format!("Failed to process {}", path.display()))?;
```

### `thiserror` - Structured Enums

Define structured error types with automatic trait implementations:

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
}
```

### `snafu` - Context Selectors

Generate context selectors for ergonomic error creation:

```rust
#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("File read failed: {path}"))]
    FileRead { path: String, source: std::io::Error },
}

// Usage with generated context selector
std::fs::read_to_string(path).context(FileReadSnafu { path })?;
```

### `color-eyre` - Enhanced Display

Similar to `anyhow` with enhanced visual output:

```rust
pub type Result<T> = color_eyre::Result<T>;

// Initialization for enhanced reports
color_eyre::install()?;
```

## Cross-Module Error Handling

### `anyhow` - Transparent Propagation

All modules use the same `Result<T>` type. Context added at boundaries:

```rust
pub fn analyze_chart(path: &Path) -> Result<Analysis> {
    let metadata = load_metadata(path)
        .context("Chart analysis failed")?;
    // ...
}
```

### `thiserror` - Hierarchical Types

Each module defines its error type, converted at boundaries:

```rust
// Root error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Chart(#[from] chart::ChartError),

    #[error(transparent)]
    Template(#[from] template::TemplateError),
}
```

### `snafu` - Context Integration

Modules generate contexts that integrate with parent errors:

```rust
#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Chart error"), context(false))]
    Chart { source: chart::ChartError },
}
```

## Public Library APIs

### Pattern Matching Support

**`thiserror`** - Full pattern matching:
```rust
match library_function() {
    Err(LibError::FileNotFound { path }) => handle_missing_file(path),
    Err(LibError::InvalidFormat { .. }) => handle_format_error(),
    Ok(data) => process(data),
}
```

**`snafu`** - Structured matching with context:
```rust
match result {
    Err(Error::FileRead { path, .. }) => retry_with_backup(&path),
    Err(Error::ParseFailed { .. }) => use_defaults(),
    Ok(data) => data,
}
```

**`anyhow`/`color-eyre`** - Limited to downcasting:
```rust
match error.downcast_ref::<std::io::Error>() {
    Some(io_err) => handle_io_error(io_err),
    None => handle_generic_error(&error),
}
```

## Error Display and Backtraces

### Cascading Error Examples

Here are real error outputs from each library showing how errors cascade through multiple layers when analyzing a malformed Chart.yaml file:

**`anyhow`** - Clean error chains with full context:
```
Error: Failed to analyze chart at /tmp/test-chart

Caused by:
    0: Failed to load chart metadata
    1: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml
    2: Invalid YAML format in Chart.yaml
    3: mapping values are not allowed in this context at line 2 column 10

Stack backtrace:
   0: <E as anyhow::context::ext::StdError>::ext_context
   1: anyhow::context::<impl anyhow::Context<T,E> for core::result::Result<T,E>>::context
   2: sextant_anyhow::chart::ChartMetadata::from_yaml
             at ./src/chart.rs:69:44
   3: sextant_anyhow::chart::ChartMetadata::load_from_file
             at ./src/chart.rs:63:9
   4: sextant_anyhow::analyzer::analyze_chart
             at ./src/analyzer.rs:23:9
   5: sextant_anyhow::analyze_single_chart
             at ./src/main.rs:98:20
```

**`thiserror`** - Structured display with source chaining:
```
Error: Failed to load chart metadata: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
  Caused by: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
  Caused by: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
  Caused by: mapping values are not allowed in this context at line 2 column 10

Stack backtrace:
   0: sextant_thiserror::main
   1: tokio::runtime::park::CachedParkThread::block_on
   2: tokio::runtime::runtime::Runtime::block_on
```

**`snafu`** - Rich reporting with intelligent deduplication:
```
Error: Analysis error *

Caused by these errors (recent errors listed first):
  1: Failed to load chart metadata *
  2: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml *
  3: Invalid YAML format in Chart.yaml *
  4: mapping values are not allowed in this context at line 2 column 10

NOTE: Some redundant information has been removed from the lines marked with *.
Set SNAFU_RAW_ERROR_MESSAGES=1 to disable this behavior.
```

**`color-eyre`** - Concise chain with enhanced formatting:
```
Error: Failed to analyze chart at /tmp/test-chart: Failed to load chart metadata: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
```

### Key Differences

- **`anyhow`**: Shows numbered error chain with full context and detailed backtraces pointing to exact source locations
- **`thiserror`**: Displays complete error messages at each level with clear "Caused by" hierarchy
- **`snafu`**: Provides numbered error list with smart deduplication and optional raw message mode
- **`color-eyre`**: Presents single-line causal chain optimized for readability (with color support in terminals)

Each approach shows the same logical error flow (main → analyze → load → parse → yaml error) but with different presentation styles optimized for their intended use cases.

## Error Context and Conversion

### From External Libraries

**`anyhow`** - Context extension:
```rust
serde_json::from_str(content)
    .with_context(|| format!("Failed to parse JSON from {}", path))?
```

**`thiserror`** - Automatic conversion:
```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("JSON parsing failed")]
    Json(#[from] serde_json::Error),
}
```

**`snafu`** - Context selectors:
```rust
serde_json::from_str(content).context(ParseJsonSnafu { path })?
```

## Quick Reference

| Library | Use Case | Pattern Matching | Boilerplate | Display Quality |
|---------|----------|------------------|-------------|-----------------|
| `anyhow` | Applications | Limited | Minimal | Good |
| `thiserror` | Libraries | Full | Moderate | Good |
| `snafu` | Rich context | Full | High | Excellent |
| `color-eyre` | Applications | Limited | Minimal | Excellent |

## Recommendations

- **`anyhow`** - Rapid application development, prototyping
- **`thiserror`** - Library APIs requiring structured error handling
- **`snafu`** - Applications needing detailed error context and debugging
- **`color-eyre`** - End-user applications prioritizing error presentation

Each approach trades off between simplicity, structure, and functionality. Choose based on your specific requirements for error handling, API design, and debugging needs.
