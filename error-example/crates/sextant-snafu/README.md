# Sextant (snafu) ⚓

A Helm Chart Resource Analyzer demonstrating error handling with [`snafu`](https://docs.rs/snafu/latest/snafu/).

## Key Characteristics

- **Context-rich errors** - Built-in backtrace support and detailed context
- **Generated context selectors** - Ergonomic error creation with `.context()`
- **Selective backtraces** - Choose which errors need debugging information
- **Enhanced reporting** - Beautiful error output with `snafu::Report`

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
#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("File read failed: {}", path.display()))]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
        backtrace: snafu::Backtrace,
    },
    
    #[snafu(display("Invalid format: {message}"))]
    InvalidFormat { 
        message: String,
        backtrace: snafu::Backtrace, 
    },
    
    #[snafu(display("YAML error"), context(false))]
    Yaml { 
        source: serde_yaml::Error,
        backtrace: snafu::Backtrace,
    },
}
```

### Context Selectors
```rust
use snafu::prelude::*;

// Generated context selectors make error creation ergonomic
let content = std::fs::read_to_string(path)
    .context(FileReadSnafu { path })?;

// context(false) enables automatic conversion
let data: Value = serde_yaml::from_str(&content)?;  // Auto-converts to Error::Yaml
```

### Selective Backtraces
```rust
#[derive(Snafu, Debug)]
pub enum ChartError {
    // Include backtrace for complex debugging scenarios
    #[snafu(display("Template render failed"))]
    RenderFailed {
        source: TemplateError,
        backtrace: snafu::Backtrace,  // Captures call stack
    },
    
    // Skip backtrace for simple validation errors
    #[snafu(display("Chart name cannot be empty"))]
    EmptyName,  // No backtrace field
}
```

### Enhanced Reporting
```rust
use snafu::{Report, ErrorCompat};

if let Err(error) = result {
    // Beautiful, structured error reporting
    let report = Report::from_error(&error);
    eprintln!("Error: {}", report);
    
    // Access backtrace when available
    if let Some(backtrace) = ErrorCompat::backtrace(&error) {
        eprintln!("Backtrace:\n{}", backtrace);
    }
}
```

### Error Output
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

## Advantages

- **Rich debugging** - Built-in backtraces where you need them
- **Ergonomic creation** - Context selectors reduce boilerplate
- **Intelligent reporting** - `snafu::Report` with deduplication
- **Flexible patterns** - Mix automatic and manual error conversion

## Trade-offs

- **Learning curve** - Context selectors and `context(false)` patterns
- **More complex** - Additional concepts beyond basic error enums
- **Selective overhead** - Choose wisely where to include backtraces

## Best For

- Applications needing detailed error context and debugging
- Complex error flows where backtraces add value
- When you want structured errors with enhanced ergonomics
- Debugging production issues with rich error information