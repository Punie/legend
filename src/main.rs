#[macro_use]
extern crate rocket;

use color_eyre::eyre::WrapErr;

mod api;
mod database;
mod graphql_rocket;
mod models;

pub use api::rocket;

#[rocket::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    api::rocket()
        .launch()
        .await
        .wrap_err("Launching Rocket server")?;

    Ok(())
}
