//! Backtrace demonstration example
//!
//! This example shows how color-eyre provides enhanced backtrace support
//! for debugging errors in complex call stacks.

use color_eyre::{Result, eyre::Context};
use std::env;

fn main() -> Result<()> {
    // Enable backtraces by default
    if env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }
    }

    // Install color-eyre with enhanced configuration for backtraces
    color_eyre::config::HookBuilder::default()
        .capture_span_trace_by_default(true)
        .display_location_section(true)
        .display_env_section(true)
        .install()?;

    println!("=== color-eyre Backtrace Demo ===\n");

    // Demonstrate deep call stack with backtraces
    println!("Triggering an error deep in the call stack...");
    deep_call_stack()?;

    Ok(())
}

/// Start a deep call stack to demonstrate backtrace capability
fn deep_call_stack() -> Result<()> {
    level_one().context("Error in deep call stack")
}

/// First level of the call stack
fn level_one() -> Result<()> {
    level_two().context("Failed at level one")
}

/// Second level of the call stack
fn level_two() -> Result<()> {
    level_three().context("Failed at level two")
}

/// Third level of the call stack
fn level_three() -> Result<()> {
    level_four().context("Failed at level three")
}

/// Fourth level of the call stack
fn level_four() -> Result<()> {
    level_five().context("Failed at level four")
}

/// Fifth level of the call stack - where the error occurs
fn level_five() -> Result<()> {
    // Simulate a file system operation that fails
    std::fs::read_to_string("nonexistent_file.txt").context("Failed to read configuration file")?;

    Ok(())
}
