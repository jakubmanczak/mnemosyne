use std::error::Error;

use tokio::net::TcpListener;

mod api;
mod database;
mod persons;
mod quotes;
mod tags;
mod users;

/// Mnemosyne, the mother of the nine muses
const DEFAULT_PORT: u16 = 0x9999; // 39321

/// The string to be returned alongside HTTP 500
const ISE_MSG: &str = "Internal server error";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = dotenvy::dotenv() {
        if !e.not_found() {
            return Err(e.into());
        }
    }

    database::migrations()?;
    users::setup::initialise_reserved_users_if_needed()?;

    let port = match std::env::var("PORT") {
        Ok(p) => p.parse::<u16>()?,
        Err(e) => match e {
            std::env::VarError::NotPresent => DEFAULT_PORT,
            _ => return Err(e)?,
        },
    };
    let r = api::api_router();
    let l = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("Listener bound to {}", l.local_addr()?);

    axum::serve(l, r).await?;
    Ok(())
}
