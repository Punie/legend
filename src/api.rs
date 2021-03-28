use rocket::response::status::{Created, NoContent};
use rocket_contrib::json::Json;

use crate::{
    db::{LegendDb, LegendDbSqlx},
    post::{NewPost, Post, PostUpdate},
};

#[get("/hello")]
fn hello_world() -> &'static str {
    "Hello, world! o/"
}

#[get("/health")]
fn health() -> NoContent {
    NoContent
}

#[get("/posts")]
async fn list_posts(conn: LegendDb) -> Json<Vec<Post>> {
    let posts = Post::all(&conn).await.unwrap();

    Json(posts)
}

#[get("/posts")]
async fn list_posts_sqlx(conn: LegendDbSqlx) -> Json<Vec<Post>> {
    let posts = Post::all_sqlx(&conn).await.unwrap();

    Json(posts)
}

#[get("/posts/<id>")]
async fn get_post(id: i32, conn: LegendDb) -> Option<Json<Post>> {
    let post = Post::find_by_id(id, &conn).await.unwrap();

    post.map(Json)
}

#[get("/posts/<id>")]
async fn get_post_sqlx(id: i32, conn: LegendDbSqlx) -> Option<Json<Post>> {
    let post = Post::find_by_id_sqlx(id, &conn).await.unwrap();

    post.map(Json)
}

#[post("/posts", format = "json", data = "<new_post>")]
async fn create_post(new_post: Json<NewPost>, conn: LegendDb) -> Created<Json<Post>> {
    let result = Post::create(new_post.into_inner(), &conn).await.unwrap();
    let location = uri!("/api", get_post: id = result.id()).to_string();

    Created::new(location).body(Json(result))
}

#[post("/posts", format = "json", data = "<new_post>")]
async fn create_post_sqlx(new_post: Json<NewPost>, conn: LegendDbSqlx) -> Created<Json<Post>> {
    let result = Post::create_sqlx(new_post.into_inner(), &conn)
        .await
        .unwrap();
    let location = uri!("/api", get_post: id = result.id()).to_string();

    Created::new(location).body(Json(result))
}

#[put("/posts/<id>", format = "json", data = "<post_update>")]
async fn update_post(id: i32, post_update: Json<PostUpdate>, conn: LegendDb) -> Option<Json<Post>> {
    let result = Post::update(id, post_update.into_inner(), &conn)
        .await
        .unwrap();

    result.map(Json)
}

#[put("/posts/<id>", format = "json", data = "<post_update>")]
async fn update_post_sqlx(
    id: i32,
    post_update: Json<PostUpdate>,
    conn: LegendDbSqlx,
) -> Option<Json<Post>> {
    let result = Post::update_sqlx(id, post_update.into_inner(), &conn)
        .await
        .unwrap();

    result.map(Json)
}

#[delete("/posts/<id>")]
async fn delete_post(id: i32, conn: LegendDb) -> Option<NoContent> {
    let result = Post::delete(id, &conn).await.unwrap();

    result.then(|| NoContent)
}

#[delete("/posts/<id>")]
async fn delete_post_sqlx(id: i32, conn: LegendDbSqlx) -> Option<NoContent> {
    let result = Post::delete_sqlx(id, &conn).await.unwrap();

    result.then(|| NoContent)
}

pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(LegendDb::fairing())
        .attach(LegendDbSqlx::fairing("legend_db"))
        .mount("/", routes![hello_world, health])
        .mount(
            "/api",
            routes![list_posts, get_post, create_post, update_post, delete_post],
        )
        .mount(
            "/api/sqlx",
            routes![
                list_posts_sqlx,
                get_post_sqlx,
                create_post_sqlx,
                update_post_sqlx,
                delete_post_sqlx
            ],
        )
}

#[cfg(test)]
mod tests {
    use rocket::{http::Status, local::blocking::Client};

    use super::rocket;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/hello").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, world! o/".into()));
    }

    #[test]
    fn health() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/health").dispatch();

        assert_eq!(response.status(), Status::NoContent);
    }
}
