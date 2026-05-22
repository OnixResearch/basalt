use clap::Parser;
use clap::Subcommand;
use snafu::ResultExt;
use snafu::Snafu;
use std::fs;
use ucan_nickel_contracts::CapabilityGrant;
use ucan_nickel_contracts::EnforcementRequest;
use ucan_nickel_contracts::enforce;
use ucan_nickel_contracts::parse_policy_json;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Enforce(EnforceArgs),
}

#[derive(Debug, Parser)]
struct EnforceArgs {
    #[arg(long)]
    policy: String,
    #[arg(long)]
    contract: String,
    #[arg(long)]
    resource: String,
    #[arg(long)]
    ability: String,
    #[arg(long = "capability", value_parser = parse_capability)]
    capabilities: Vec<CapabilityGrant>,
}

#[derive(Debug, Snafu)]
enum CliError {
    #[snafu(display("failed to read policy file {path:?}: {source}"))]
    ReadPolicy {
        path: String,
        source: std::io::Error,
    },
    #[snafu(display("enforcement failed: {source}"))]
    Enforce {
        source: ucan_nickel_contracts::EnforcementError,
    },
    #[snafu(display("failed to serialize receipt: {source}"))]
    Serialize { source: serde_json::Error },
}

type CliResult<T> = Result<T, CliError>;

fn parse_capability(input: &str) -> Result<CapabilityGrant, String> {
    let Some((resource, ability)) = input.split_once('=') else {
        return Err("expected RESOURCE=ABILITY".to_owned());
    };
    if resource.is_empty() || ability.is_empty() {
        return Err("capability resource and ability must be non-empty".to_owned());
    }
    Ok(CapabilityGrant::new(resource, ability))
}

fn main() -> CliResult<()> {
    let args = Args::parse();
    match args.command {
        Command::Enforce(enforce_args) => run_enforce(enforce_args),
    }
}

fn run_enforce(args: EnforceArgs) -> CliResult<()> {
    let policy_json = fs::read_to_string(args.policy.as_str()).context(ReadPolicySnafu {
        path: args.policy.clone(),
    })?;
    let policy = parse_policy_json(policy_json.as_str()).context(EnforceSnafu)?;
    let request = EnforcementRequest {
        contract: args.contract,
        resource: args.resource,
        ability: args.ability,
        capabilities: args.capabilities,
    };
    let receipt = enforce(&policy, &request).context(EnforceSnafu)?;
    let output = serde_json::to_string(&receipt).context(SerializeSnafu)?;
    println!("{output}");
    Ok(())
}
