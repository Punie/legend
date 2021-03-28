use rocket::{
    fairing::{AdHoc, Fairing},
    http::Status,
    request::{FromRequest, Outcome, Request},
    trace,
};
use rocket_contrib::databases::{diesel::PgConnection, Config};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[database("legend_db")]
pub struct LegendDb(PgConnection);

#[derive(Clone, Debug)]
pub struct LegendDbSqlx(PgPool);

impl LegendDbSqlx {
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
                Ok(pool) => Ok(rocket.manage(LegendDbSqlx(pool))),
                Err(e) => {
                    trace::error!(error = %e, "Error connecting to `{}` pool", db);
                    Err(rocket)
                }
            }
        })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LegendDbSqlx {
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
