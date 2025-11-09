# Sextant (color-eyre) ⚓

A Helm Chart Resource Analyzer demonstrating error handling with [`color-eyre`](https://docs.rs/color-eyre/latest/color_eyre/).

## Key Characteristics

- **Enhanced display** - Colorized error output with beautiful formatting
- **Zero custom types** - Uses `color_eyre::Result<T>` for everything
- **Automatic backtraces** - Built-in span traces and location information
- **Rich debugging** - Enhanced panic hooks and error reporting

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
// No custom error types needed
pub type Result<T> = color_eyre::Result<T>;
```

### Enhanced Setup
```rust
// Install enhanced error reporting
color_eyre::config::HookBuilder::default()
    .capture_span_trace_by_default(true)
    .display_location_section(true)
    .display_env_section(false)
    .install()?;
```

### Context Addition
```rust
use color_eyre::{Result, eyre::Context};

// Same ergonomics as anyhow
let metadata = ChartMetadata::load_from_file(&chart_file)
    .with_context(|| format!("Failed to load {}", chart_file.display()))?;

// Custom errors with bail!
if !path.exists() {
    color_eyre::eyre::bail!("Chart directory not found: {}", path.display());
}
```

### Error Conversion
```rust
// Automatic conversion from any std::error::Error
let content = std::fs::read_to_string(path)?;  // io::Error -> color_eyre::Error
let data: Value = serde_yaml::from_str(&content)?;  // serde_yaml::Error -> color_eyre::Error
```

### Error Output
```
Error: Failed to analyze chart at /tmp/test-chart: Failed to load chart metadata: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml: Invalid YAML format in Chart.yaml: mapping values are not allowed in this context at line 2 column 10
```

*Note: In terminals with color support, this output includes syntax highlighting, colored headers, and enhanced visual formatting.*

## Advantages

- **Beautiful output** - Colorized, formatted error messages
- **Rich debugging** - Enhanced backtraces with span information
- **Zero boilerplate** - Like `anyhow` but with better visuals
- **Enhanced hooks** - Better panic handling and error reporting

## Trade-offs

- **No pattern matching** - Consumers can't handle specific error types
- **Setup required** - Must install color-eyre hooks for full benefits
- **Application focused** - Not suitable for library APIs
- **Dependencies** - Forces consumers into color-eyre ecosystem

## Best For

- Command-line applications with end-user error display
- Development and debugging environments
- Applications prioritizing visual error presentation
- Tools where error aesthetics matter to user experience
