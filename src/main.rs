use std::io::{self, Read};

use anyhow::{Context, Result, bail};
use codex_gearbox::{
    Config,
    app_server::{ControlClient, ManagedServer, route_live},
    hook, metrics, proxy, update,
};

#[tokio::main]
async fn main() -> Result<()> {
    if update::is_update_worker() {
        update::download_latest().await;
        return Ok(());
    }
    if let Some(code) = update::delegate_to_cached().await? {
        std::process::exit(code);
    }
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let config = Config::load()?;
    let shift_mode = is_shift_binary();
    match args.first().map(String::as_str) {
        Some("route") => {
            args.remove(0);
            if args.is_empty() {
                bail!("usage: shift route <prompt>");
            }
            let route = route_live(&config, &args.join(" ")).await;
            println!("{}", serde_json::to_string_pretty(&route)?);
        }
        Some("hook") => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let parsed = hook::parse(&input)?;
            let route = route_live(&config, &parsed.prompt).await;
            println!("{}", hook::output(&parsed, &route));
        }
        Some("account") => {
            let mut server = ManagedServer::start().await?;
            let result = async {
                let mut client = ControlClient::connect_with_retry(&server.url).await?;
                client.snapshot(&config).await
            }
            .await;
            server.stop().await;
            println!("{}", serde_json::to_string_pretty(&result?)?);
        }
        Some("report") => println!("{}", serde_json::to_string_pretty(&metrics::report()?)?),
        Some("--help" | "-h" | "help") if shift_mode => print_shift_help(),
        Some("--version" | "-V") if shift_mode => println!("shift {}", env!("CARGO_PKG_VERSION")),
        Some("--help" | "-h" | "help") => print_help(),
        Some("--version" | "-V") => println!("codex-gearbox {}", env!("CARGO_PKG_VERSION")),
        Some("run") if !shift_mode => {
            args.remove(0);
            std::process::exit(proxy::run(config, args).await?);
        }
        _ if shift_mode => {
            print_shift_help();
            bail!("usage: shift <route|account|report|hook>");
        }
        _ => std::process::exit(
            proxy::run(config, args)
                .await
                .context("Codex Gearbox failed")?,
        ),
    }
    Ok(())
}

fn is_shift_binary() -> bool {
    std::env::args_os()
        .next()
        .and_then(|arg| {
            std::path::PathBuf::from(arg)
                .file_stem()
                .map(ToOwned::to_owned)
        })
        .and_then(|name| name.into_string().ok())
        .is_some_and(|name| name == "shift")
}

fn print_help() {
    println!(
        "Codex Gearbox — plan-aware model and effort routing\n\n\
         Usage:\n  codex-gearbox [codex arguments]\n  codex-gearbox --version\n\n\
         Utilities:\n  shift route <prompt>\n  shift hook\n  shift account\n  shift report"
    );
}

fn print_shift_help() {
    println!(
        "Codex Gearbox utilities\n\n\
         Usage:\n  shift route <prompt>\n  shift hook\n  shift account\n  shift report"
    );
}
