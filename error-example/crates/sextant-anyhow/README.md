# Sextant (anyhow) ⚓

A Helm Chart Resource Analyzer demonstrating error handling with [`anyhow`](https://docs.rs/anyhow/latest/anyhow/).

## Key Characteristics

- **No custom error types** - Uses `anyhow::Error` for everything
- **Maximum ergonomics** - `bail!`, `ensure!`, and `anyhow!` macros
- **Automatic conversion** - Any `std::error::Error` converts seamlessly
- **Rich context chains** - `.with_context()` for meaningful error messages

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
pub type Result<T> = anyhow::Result<T>;
```

### Context Addition
```rust
use anyhow::{Context, Result, bail, ensure};

// Dynamic context
let metadata = ChartMetadata::load_from_file(&chart_file)
    .with_context(|| format!("Failed to load {}", chart_file.display()))?;

// Validation with ensure!
ensure!(!self.name.is_empty(), "Chart name cannot be empty");

// Early returns with bail!
if !path.exists() {
    bail!("Chart directory not found: {}", path.display());
}
```

### Error Conversion
```rust
// Automatic conversion from any std::error::Error
let content = std::fs::read_to_string(path)?;  // io::Error -> anyhow::Error
let data: Value = serde_yaml::from_str(&content)?;  // serde_yaml::Error -> anyhow::Error
```

### Error Output
```
Error: Failed to analyze chart at /tmp/test-chart

Caused by:
    0: Failed to load chart metadata
    1: Failed to parse Chart.yaml from /tmp/test-chart/Chart.yaml
    2: Invalid YAML format in Chart.yaml
    3: mapping values are not allowed in this context at line 2 column 10
```

## Advantages

- **Minimal boilerplate** - Just one type alias
- **Rapid development** - No upfront error type design
- **Clear error chains** - Automatic context preservation
- **Thread safe** - `Send + Sync` by default

## Trade-offs

- **No pattern matching** - Consumers can't handle specific error types
- **Dynamic errors** - All type information erased at runtime
- **Library dependency** - Consumers must use `anyhow` transitively

## Best For

- Applications where error structure isn't critical
- Rapid prototyping and development
- Internal tools and utilities
- When maximum development speed is priority