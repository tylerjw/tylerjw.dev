//! Error demonstration example
//!
//! This example shows how color-eyre provides enhanced error reporting
//! compared to anyhow, including colorized output, better formatting,
//! and additional debugging information.

use color_eyre::{eyre::Context, Result};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // Install color-eyre for enhanced error reporting
    color_eyre::install()?;

    println!("=== color-eyre Error Handling Demo ===\n");

    // Demonstrate error chain with context
    println!("1. Demonstrating error chains with context:");
    if let Err(e) = read_config_file("nonexistent.yaml") {
        eprintln!("Error occurred: {:#}", e);
    }

    println!("\n2. Demonstrating nested function calls with context:");
    if let Err(e) = process_user_data("invalid_user") {
        eprintln!("Error occurred: {:#}", e);
    }

    println!("\n3. Demonstrating validation errors:");
    if let Err(e) = validate_input("") {
        eprintln!("Error occurred: {:#}", e);
    }

    Ok(())
}

/// Read configuration file with error context
fn read_config_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();

    // This will fail and show the file system error with context
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    parse_config(&contents).context("Failed to parse configuration")
}

/// Parse configuration with validation
fn parse_config(contents: &str) -> Result<String> {
    if contents.is_empty() {
        color_eyre::eyre::bail!("Configuration file is empty");
    }

    if !contents.contains("version") {
        color_eyre::eyre::bail!("Configuration missing required 'version' field");
    }

    Ok(contents.to_string())
}

/// Process user data through multiple function calls
fn process_user_data(username: &str) -> Result<()> {
    let user = load_user(username)
        .with_context(|| format!("Failed to process user data for '{}'", username))?;

    validate_user(&user).context("User validation failed")?;

    save_user_session(&user).context("Failed to save user session")
}

/// Load user from database (simulated)
fn load_user(username: &str) -> Result<User> {
    if username == "invalid_user" {
        color_eyre::eyre::bail!("User '{}' not found in database", username);
    }

    if username.is_empty() {
        color_eyre::eyre::bail!("Username cannot be empty");
    }

    Ok(User {
        name: username.to_string(),
        email: format!("{}@example.com", username),
    })
}

/// Validate user data
fn validate_user(user: &User) -> Result<()> {
    if user.name.len() < 3 {
        color_eyre::eyre::bail!("Username must be at least 3 characters long");
    }

    if !user.email.contains('@') {
        color_eyre::eyre::bail!("Invalid email format: {}", user.email);
    }

    Ok(())
}

/// Save user session (simulated)
fn save_user_session(user: &User) -> Result<()> {
    // Simulate a file system error
    fs::write(
        "/invalid/path/session.json",
        format!("{{\"user\": \"{}\"}}", user.name),
    )
    .with_context(|| format!("Failed to save session for user '{}'", user.name))?;

    Ok(())
}

/// Simple validation function
fn validate_input(input: &str) -> Result<()> {
    if input.is_empty() {
        color_eyre::eyre::bail!("Input cannot be empty");
    }

    if input.len() < 5 {
        color_eyre::eyre::bail!(
            "Input must be at least 5 characters long, got {}",
            input.len()
        );
    }

    Ok(())
}

#[derive(Debug)]
struct User {
    name: String,
    email: String,
}
