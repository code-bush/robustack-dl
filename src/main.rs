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
//! # Architecture
//! `main()` is a **thin composition root**.  It:
//! 1. Parses CLI arguments.
//! 2. Initializes tracing.
//! 3. Builds `AppConfig` from CLI args.
//! 4. Constructs infrastructure (`ReqwestClient`).
//! 5. Dispatches to the appropriate handler.
//!
//! No business logic lives here — all domain work happens in `handlers/`.

// Provenance headers use proper nouns (e.g. CodeBush) that are not code identifiers.
#![allow(clippy::doc_markdown)]
// Stub modules are scaffolded but not yet wired to subcommands.
#![allow(dead_code)]

mod cli;
mod client;
mod config;
mod handlers;
mod integrity;
mod processor;

use cli::{Cli, Commands};
use client::ReqwestClient;
use config::AppConfig;

/// Thin composition root — no business logic, only wiring.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = <Cli as clap::Parser>::parse();

    // Step 1: Initialize tracing (respect --verbose).
    let default_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level)),
        )
        .init();

    // Step 2: Dispatch to handler.
    match cli.command {
        Commands::Download(ref args) => {
            let config = AppConfig::from_cli(&cli, args.limit, Some(args));
            let http_client = ReqwestClient::from_config(&config);
            handlers::download::run(&args.url, &config, &http_client).await?;
        }
        Commands::List(ref args) => {
            let config = AppConfig::from_cli(&cli, args.limit, None);
            let http_client = ReqwestClient::from_config(&config);
            handlers::list::run(&args.url, &config, &http_client).await?;
        }
        Commands::Audit(ref args) => {
            handlers::audit::run(&args.manifest)?;
        }
        Commands::Completions(ref args) => {
            cli::print_completions(args.shell);
        }
        Commands::Version => {
            use std::io::Write;
            let version = <Cli as clap::CommandFactory>::command().render_version();
            // Write to stdout (not println! — project uses tracing for logging).
            std::io::stdout().write_all(version.as_bytes())?;
        }
    }

    Ok(())
}
