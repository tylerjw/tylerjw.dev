//! Sextant CLI - Helm Chart Resource Analyzer
//!
//! A command-line tool for analyzing Helm charts and generating reports about
//! the Kubernetes resources they would create.

use clap::{Parser, Subcommand};
use color_eyre::{Result, eyre::Context};
use sextant_color_eyre::{analyze_chart, analyzer::analyze_charts, report::ReportFormat};
use std::{env, path::PathBuf};

#[derive(Parser)]
#[command(
    name = "sextant",
    about = "Helm Chart Resource Analyzer",
    long_about = "Analyze Helm charts and generate reports showing what Kubernetes resources would be created"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a single Helm chart
    Chart {
        /// Path to the Helm chart directory
        path: PathBuf,
        /// Output file for the report
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format (json, yaml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Analyze multiple Helm charts in a directory
    Charts {
        /// Path to directory containing multiple Helm charts
        path: PathBuf,
        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format (json, yaml)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Generate a summary markdown report
        #[arg(short, long)]
        summary: bool,
    },
}

#[tokio::main]
#[async_backtrace::framed]
async fn main() {
    // Enable backtraces by default
    if env::var("RUST_BACKTRACE").is_err() {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }
    }

    // Install color-eyre with enhanced configuration
    color_eyre::config::HookBuilder::default()
        .capture_span_trace_by_default(true)
        .display_location_section(true)
        .display_env_section(false)
        .install()
        .expect("Failed to install color-eyre");

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Chart {
            path,
            output,
            format,
        } => analyze_single_chart(path, output, format).await,
        Commands::Charts {
            path,
            output,
            format,
            summary,
        } => analyze_multiple_charts(path, output, format, summary).await,
    };

    if let Err(error) = result {
        eprintln!("Error: {:#}", error);
        std::process::exit(1);
    }
}

#[async_backtrace::framed]
async fn analyze_single_chart(
    chart_path: PathBuf,
    output_path: Option<PathBuf>,
    format: String,
) -> Result<()> {
    println!("Analyzing chart: {}", chart_path.display());

    let analysis = analyze_chart(&chart_path)
        .with_context(|| format!("Failed to analyze chart at {}", chart_path.display()))?;

    let report_format = parse_format(&format)?;

    if let Some(output_path) = output_path {
        analysis.save_to_file(&output_path, report_format)?;
        println!("Report saved to: {}", output_path.display());
    } else {
        // Print to stdout
        let content = match report_format {
            ReportFormat::Json => analysis.to_json()?,
            ReportFormat::Yaml => analysis.to_yaml()?,
        };
        println!("{}", content);
    }

    // Print summary to stderr so it doesn't interfere with stdout output
    print_chart_summary(&analysis);

    Ok(())
}

#[async_backtrace::framed]
async fn analyze_multiple_charts(
    charts_dir: PathBuf,
    output_dir: Option<PathBuf>,
    format: String,
    generate_summary: bool,
) -> Result<()> {
    println!("Analyzing charts in: {}", charts_dir.display());

    let analyses = analyze_charts(&charts_dir)
        .await
        .with_context(|| format!("Failed to analyze charts in {}", charts_dir.display()))?;

    if analyses.is_empty() {
        println!("No Helm charts found in {}", charts_dir.display());
        return Ok(());
    }

    let report_format = parse_format(&format)?;

    if let Some(output_dir) = output_dir {
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&output_dir).with_context(|| {
            format!("Failed to create output directory {}", output_dir.display())
        })?;

        // Save individual reports
        for analysis in &analyses {
            let filename = format!("{}.{}", analysis.chart_name, report_format.extension());
            let output_path = output_dir.join(filename);

            analysis.save_to_file(&output_path, report_format)?;
            println!("Report saved: {}", output_path.display());
        }

        // Generate summary report if requested
        if generate_summary {
            let summary_markdown = sextant_color_eyre::report::generate_markdown_summary(&analyses);
            let summary_path = output_dir.join("summary.md");

            std::fs::write(&summary_path, summary_markdown).with_context(|| {
                format!("Failed to write summary to {}", summary_path.display())
            })?;

            println!("Summary saved: {}", summary_path.display());
        }
    } else {
        // Print all analyses to stdout
        for (i, analysis) in analyses.iter().enumerate() {
            if i > 0 {
                println!("---"); // Document separator
            }
            let content = match report_format {
                ReportFormat::Json => analysis.to_json()?,
                ReportFormat::Yaml => analysis.to_yaml()?,
            };
            println!("{}", content);
        }

        // Generate summary if requested
        if generate_summary {
            println!("\n---\n");
            let summary_markdown = sextant_color_eyre::report::generate_markdown_summary(&analyses);
            println!("{}", summary_markdown);
        }
    }

    // Print summary to stderr
    print_charts_summary(&analyses);

    Ok(())
}

fn parse_format(format: &str) -> Result<ReportFormat> {
    ReportFormat::from_extension(format).ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "Unsupported format '{}'. Supported formats: json, yaml",
            format
        )
    })
}

fn print_chart_summary(analysis: &sextant_color_eyre::ChartAnalysis) {
    eprintln!();
    eprintln!("=== Chart Summary ===");
    eprintln!(
        "Chart: {} ({})",
        analysis.chart_name, analysis.chart_version
    );
    eprintln!("Path: {}", analysis.chart_path.display());
    eprintln!("Values files: {}", analysis.values_file_count());

    if let Some(description) = &analysis.metadata.description {
        eprintln!("Description: {}", description);
    }

    eprintln!("Dependencies: {}", analysis.metadata.dependency_count);

    let resource_summary = analysis.get_resource_summary();
    if !resource_summary.is_empty() {
        eprintln!();
        eprintln!("Resource summary across all values files:");
        for (resource_type, count) in resource_summary {
            eprintln!("  {}: {}", resource_type, count);
        }
    } else {
        eprintln!("No resources found");
    }
    eprintln!();
}

fn print_charts_summary(analyses: &[sextant_color_eyre::ChartAnalysis]) {
    eprintln!();
    eprintln!("=== Analysis Summary ===");
    eprintln!("Charts analyzed: {}", analyses.len());

    if !analyses.is_empty() {
        let mut total_resources = std::collections::HashMap::new();
        let mut total_values_files = 0;

        for analysis in analyses {
            total_values_files += analysis.values_file_count();
            let summary = analysis.get_resource_summary();
            for (resource_type, count) in summary {
                *total_resources.entry(resource_type).or_insert(0) += count;
            }
        }

        eprintln!("Total values files: {}", total_values_files);

        if !total_resources.is_empty() {
            eprintln!("Total resources across all charts:");
            let mut sorted_resources: Vec<_> = total_resources.into_iter().collect();
            sorted_resources.sort_by(|a, b| a.0.cmp(&b.0));

            for (resource_type, count) in sorted_resources {
                eprintln!("  {}: {}", resource_type, count);
            }
        } else {
            eprintln!("No resources found in any charts");
        }
    }
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_parse_format() -> Result<()> {
        assert_eq!(parse_format("json")?, ReportFormat::Json);
        assert_eq!(parse_format("yaml")?, ReportFormat::Yaml);
        assert_eq!(parse_format("yml")?, ReportFormat::Yaml);

        assert!(parse_format("xml").is_err());
        assert!(parse_format("txt").is_err());

        Ok(())
    }
}
