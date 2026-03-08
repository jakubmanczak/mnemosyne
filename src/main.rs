use std::error::Error;

use axum::Router;
use tokio::net::TcpListener;

mod api;
mod config;
mod database;
mod persons;
mod quotes;
mod tags;
mod users;
mod web;

/// Mnemosyne, the mother of the nine muses
const DEFAULT_PORT: u16 = 0x9999; // 39321

/// The string to be returned alongside HTTP 500
const ISE_MSG: &str = "Internal server error";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenvy::dotenv()
        && !e.not_found()
    {
        return Err(e.into());
    }
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format(config::envlogger_write_format)
        .init();

    database::migrations()?;
    users::auth::init_password_dummies();
    users::setup::initialise_reserved_users_if_needed()?;

    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>()?,
        Err(e) => match e {
            std::env::VarError::NotPresent => DEFAULT_PORT,
            _ => return Err(e)?,
        },
    };
    let r = Router::new()
        .merge(api::api_router())
        .merge(web::web_router());
    let l = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    log::info!("Listener bound to {}", l.local_addr()?);

    axum::serve(l, r).await?;
    Ok(())
}
