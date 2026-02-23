use std::error::Error;

mod database;
mod persons;
mod quotes;
mod tags;
mod users;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    database::migrations()?;
    users::setup::initialise_reserved_users_if_needed()?;

    Ok(())
}
