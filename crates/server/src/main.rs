//! Server entry point: the headless program around the server library.

mod cli;
mod user_admin;

use std::{error::Error, sync::Arc};
use veltrunox_server::{
    Seed, config::Config, listen, open_store, prepare_tls, start_core, with_cors, with_web_app,
};
use veltrunox_server_adapters::{Argon2Hasher, RandomTokens, SqliteStore, SystemWallClock, router};
use veltrunox_server_app::AccountService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let invocation = cli::parse(std::env::args().skip(1)).unwrap_or_else(|usage| {
        eprintln!("{usage}");
        std::process::exit(2);
    });
    let config = Config::load(invocation.config.as_deref()).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    if let cli::Command::Users(command) = invocation.command {
        return administer_users(open_store(&config)?, &command);
    }
    let app = start_core(&config, Seed::Samples)?;
    let tls = prepare_tls(&config)?;
    let scheme = if tls.is_some() { "https" } else { "http" };
    let routes = with_web_app(
        with_cors(router(app), config.allowed_origin.as_deref()),
        config.web_dir.as_deref(),
    );
    let (_, serving) = listen(&config.address, routes, tls)?;
    println!("listening on {scheme}://{}", config.address);
    serving.await?;
    Ok(())
}

/// Runs one user command against the database and ends the process on failure.
fn administer_users(
    store: Arc<SqliteStore>,
    command: &cli::UserCommand,
) -> Result<(), Box<dyn Error>> {
    let accounts = AccountService::new(
        store.clone(),
        store,
        Arc::new(Argon2Hasher::new()),
        Arc::new(RandomTokens),
        Arc::new(SystemWallClock),
    )?;
    let result = user_admin::run(
        &accounts,
        command,
        &mut user_admin::TerminalPasswords,
        &mut std::io::stdout(),
    );
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
    Ok(())
}
