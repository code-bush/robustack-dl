//! @project       RoBustack-DL
//! @organization  CodeBush Collective
//! @license       GPL-3.0-only
//! ---------------------------------------------------------------------------
//! AI PROVENANCE & HUMAN-IN-THE-LOOP (HITL) METADATA:
//! - Prompt Engineering: Gemini 3 Flash (Strategy, Scoping & Context Tuning)
//! - Code Generation:   Gemini 3 Pro (Core Systems Engineering & Async Logic)
//! - Technical Review:  Claude 4.6 Opus (Security Audit & Idiomatic Refinement)
//! - HITL Verification: Collisio-Adolebitque - AA0614550BDC21F1 (Manual Audit & Final Validation)
//! ---------------------------------------------------------------------------
//! Verified Date: 2026-02-15
//! Integrity: GPG-Signed | HITL-Certified
//!
//! Entry point for the RoBustack-DL CLI.
//!
//! # Errors
//! Returns `anyhow::Error` on:
//! - Tracing subscriber initialization failure
//! - Subcommand execution failure (I/O, network, integrity mismatch)

use tracing::info;

mod cli;
mod client;
mod config;
mod integrity;
mod processor;

use cli::{Cli, Commands};

/// Main async entry point.
///
/// Initializes structured logging via `tracing`, parses CLI arguments,
/// and dispatches to the appropriate subcommand handler.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Wire up EnvFilter so RUST_LOG is respected at runtime.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = <Cli as clap::Parser>::parse();

    match cli.command {
        Commands::Download(args) => {
            info!(url = %args.url, output = %args.output.display(), "Starting download");
            // TODO: Wire to client::download pipeline
        }
        Commands::Audit(args) => {
            info!(manifest = %args.manifest.display(), "Starting integrity audit");
            // TODO: Wire to integrity::verify pipeline
        }
        Commands::Completions(args) => {
            cli::print_completions(args.shell);
        }
    }

    Ok(())
}
