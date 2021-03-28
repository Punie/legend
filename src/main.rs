#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

use color_eyre::eyre::WrapErr;

mod api;
mod db;
mod post;
mod schema;
mod utils;

pub use api::rocket;
pub use utils::*;

#[rocket::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    api::rocket()
        .launch()
        .await
        .wrap_err("Launching Rocket server")?;

    Ok(())
}
