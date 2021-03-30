use rocket::{
    fairing::{AdHoc, Fairing},
    figment::{self, providers::Serialized, Figment},
    http::Status,
    request::{FromRequest, Outcome, Request},
    trace,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Config {
    pub url: String,
    pub pool_size: u32,
    pub timeout: u8,
}

impl Config {
    pub fn from(db_name: &str, rocket: &rocket::Rocket) -> Result<Config, figment::Error> {
        let db_key = format!("databases.{}", db_name);
        let key = |name: &str| format!("{}.{}", db_key, name);

        Figment::from(rocket.figment())
            .merge(Serialized::default(
                &key("pool_size"),
                rocket.config().workers * 2,
            ))
            .merge(Serialized::default(&key("timeout"), 5))
            .extract_inner::<Self>(&db_key)
    }
}

#[derive(Clone, Debug)]
pub struct LegendDb(PgPool);

impl LegendDb {
    pub fn pool(&self) -> &PgPool {
        &self.0
    }

    pub fn fairing(db: &'static str) -> impl Fairing {
        AdHoc::on_attach("'legend_db' Database pool sqlx", move |rocket| async move {
            let config = match Config::from(db, &rocket) {
                Ok(config) => config,
                Err(e) => {
                    trace::error!(error = %e, "Database config error for sqlx pool named `{}`", db);
                    return Err(rocket);
                }
            };

            let pool_size = config.pool_size;

            let pool = PgPoolOptions::new()
                .max_connections(pool_size)
                .connect(&config.url)
                .await;

            match pool {
                Ok(pool) => Ok(rocket.manage(LegendDb(pool))),
                Err(e) => {
                    trace::error!(error = %e, "Error connecting to `{}` pool", db);
                    Err(rocket)
                }
            }
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LegendDb {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        match request.managed_state::<Self>() {
            Some(pool) => Outcome::Success(pool.clone()),
            None => {
                trace::error!("Missing database fairing for `LegendDbSqlx`",);
                Outcome::Failure((Status::InternalServerError, ()))
            }
        }
    }
}
