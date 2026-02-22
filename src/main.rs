use std::error::Error;

mod database;
mod persons;
mod quotes;
mod tags;
mod users;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    Ok(())
}
