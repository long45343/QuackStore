//! `quackd`: The unified backend for QuackStore.
//! Exposes RKP, device-backed signing, keystore management, and in-process scheduler daemon.

mod builtin;
pub mod daemon;

use clap::{Arg, ArgAction, error::ErrorKind};
use quack_core::{
    Context, Feature, Sysroot,
    command::{CommandFailure, CommandOutput, CommandResult},
    envelope,
};

static FEATURES: &[&dyn Feature] = &[
    &quack_rkp::FEATURE,
    &quack_rkp_device::FEATURE,
    &quack_tricky_store::FEATURE,
];

fn cli() -> clap::Command {
    clap::Command::new("quackd")
        .version(env!("CARGO_PKG_VERSION"))
        .about("QuackStore backend: RKP hardware provisioning, Keystore manager and scheduler daemon")
        .arg(
            Arg::new("json")
                .long("json")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Accepted for compatibility; output is always JSON"),
        )
        .subcommand(
            clap::Command::new("daemon")
                .about("Run the in-process periodic scheduler daemon (foreground)")
        )
        .subcommand(
            clap::Command::new("sync-now")
                .about("Immediately trigger a full RKP hardware sign, 2100 RSA injection and sync to active keystore")
        )
        .subcommands(builtin::subcommands())
        .subcommands(FEATURES.iter().map(|f| f.command()))
}

#[tokio::main]
async fn main() {
    let matches = match cli().try_get_matches() {
        Ok(matches) => matches,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            error.exit()
        }
        Err(error) => {
            let failure = CommandFailure::new(
                "bootstrap",
                "usage_error",
                &anyhow::anyhow!(error.render().to_string()),
                None,
            );
            envelope::emit(&envelope::failure(&failure));
            std::process::exit(2);
        }
    };

    let paths = match quack_core::AppPaths::discover() {
        Ok(paths) => paths,
        Err(error) => {
            let failure = CommandFailure::new("bootstrap", "bootstrap_error", &error, None);
            envelope::emit(&envelope::failure(&failure));
            std::process::exit(1);
        }
    };
    let ctx = Context {
        paths,
        sysroot: Sysroot::from_env(),
    };

    let result: CommandResult = match matches.subcommand() {
        Some(("daemon", _)) => {
            if let Err(e) = daemon::run_daemon(ctx).await {
                eprintln!("[quackd daemon error] {e:#}");
                std::process::exit(1);
            }
            return;
        }
        Some(("sync-now", _)) => {
            match daemon::run_sync_once(&ctx).await {
                Ok(data) => CommandOutput::new("sync-now", data),
                Err(err) => Err(CommandFailure::new("sync-now", "sync_failed", &err, None)),
            }
        }
        Some((builtin::MANIFEST, _)) => builtin::manifest(FEATURES),
        Some((builtin::DESCRIBE, _)) => builtin::describe(FEATURES, &ctx),
        Some((id, sub)) => match FEATURES.iter().find(|feature| feature.info().id == id) {
            Some(feature) => feature.run(sub, &ctx).await,
            None => unreachable!("clap only accepts registered subcommands"),
        },
        None => {
            let failure = CommandFailure::new("bootstrap", "usage_error", &anyhow::anyhow!("subcommand required"), None);
            envelope::emit(&envelope::failure(&failure));
            std::process::exit(2);
        }
    };

    emit(&ctx, result);
}

fn emit(ctx: &Context, result: CommandResult) {
    let (value, failed) = match result {
        Ok(output) => (envelope::success(&output), false),
        Err(failure) => (envelope::failure(&failure), true),
    };

    quack_core::log::append(&ctx.paths, &value);
    envelope::emit(&value);

    if failed {
        std::process::exit(1);
    }
}
