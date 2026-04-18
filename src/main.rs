use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use agent_receipts::{
    Batch, ChainFixture, Receipt, default_keys_dir, load_public_keys_dir, read_json_file,
    schema_check_file, verify_batch, verify_chain, verify_receipt,
};

#[derive(Parser)]
#[command(name = "receipts")]
#[command(about = "Verify post-quantum receipts for AI agent actions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SchemaCheck {
        path: PathBuf,
    },
    Verify {
        path: PathBuf,
        #[arg(long, default_value_os_t = default_keys_dir())]
        keys_dir: PathBuf,
    },
    VerifyBatch {
        path: PathBuf,
        #[arg(long, default_value_os_t = default_keys_dir())]
        keys_dir: PathBuf,
    },
    VerifyChain {
        path: PathBuf,
        #[arg(long, default_value_os_t = default_keys_dir())]
        keys_dir: PathBuf,
    },
    Inspect {
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::from(error_code(&error))
        }
    }
}

fn run(command: Commands) -> Result<()> {
    match command {
        Commands::SchemaCheck { path } => {
            let schema_version = schema_check_file(&path)?;
            println!("schema-check ok: {schema_version}");
        }
        Commands::Verify { path, keys_dir } => {
            let receipt: Receipt = read_json_file(&path)?;
            let keys = load_public_keys_dir(&keys_dir)?;
            verify_receipt(&receipt, &keys)?;
            println!("verify ok: {}", receipt.receipt_id);
        }
        Commands::VerifyBatch { path, keys_dir } => {
            let batch: Batch = read_json_file(&path)?;
            let keys = load_public_keys_dir(&keys_dir)?;
            verify_batch(&batch, &keys)?;
            println!("verify-batch ok: {}", batch.stratum_id);
        }
        Commands::VerifyChain { path, keys_dir } => {
            let chain: ChainFixture = read_json_file(&path)?;
            let keys = load_public_keys_dir(&keys_dir)?;
            verify_chain(&chain, &keys)?;
            println!("verify-chain ok: {}", chain.chain_id);
        }
        Commands::Inspect { path } => {
            let raw = std::fs::read_to_string(&path)?;
            let parsed: serde_json::Value = serde_json::from_str(&raw)?;
            println!("{}", serde_json::to_string_pretty(&parsed)?);
        }
    }

    Ok(())
}

fn error_code(error: &anyhow::Error) -> u8 {
    let message = format!("{error:#}");
    if message.contains("failed to read") || message.contains("failed to parse JSON") {
        3
    } else if message.contains("schema_version")
        || message.contains("invalid ")
        || message.contains("not a valid")
        || message.contains("missing")
        || message.contains("unsupported")
    {
        2
    } else {
        1
    }
}
