mod graphql;
mod posts;

use graphql::{graphql_playground, graphql_query, graphql_request, GraphQlFairing};
use rocket::response::status::NoContent;

use crate::database::LegendDb;

use posts::{create_post, delete_post, get_post, list_posts, update_post};

#[get("/health")]
fn health() -> NoContent {
    NoContent
}

pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(LegendDb::fairing("legend_db"))
        .attach(GraphQlFairing)
        .mount("/", routes![health])
        .mount(
            "/graphql",
            routes![graphql_playground, graphql_query, graphql_request],
        )
        .mount(
            "/api",
            routes![list_posts, get_post, create_post, update_post, delete_post],
        )
}

#[cfg(test)]
mod tests {
    use rocket::{http::Status, local::blocking::Client};

    use super::rocket;

    #[test]
    fn health() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/health").dispatch();

        assert_eq!(response.status(), Status::NoContent);
    }
}
