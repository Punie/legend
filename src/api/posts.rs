use rocket::response::status::{Created, NoContent};
use rocket_contrib::json::Json;

use crate::{
    database::LegendDb,
    models::post::{NewPost, Post, PostUpdate},
};

#[get("/posts")]
pub async fn list_posts(conn: LegendDb) -> Json<Vec<Post>> {
    let posts = Post::all(&conn).await.unwrap();

    Json(posts)
}

#[get("/posts/<id>")]
pub async fn get_post(id: i32, conn: LegendDb) -> Option<Json<Post>> {
    let post = Post::find_by_id(id, &conn).await.unwrap();

    post.map(Json)
}

#[post("/posts", format = "json", data = "<new_post>")]
pub async fn create_post(new_post: Json<NewPost>, conn: LegendDb) -> Created<Json<Post>> {
    let result = Post::create(new_post.into_inner(), &conn).await.unwrap();
    let location = uri!("/api", get_post: id = result.id).to_string();

    Created::new(location).body(Json(result))
}

#[put("/posts/<id>", format = "json", data = "<post_update>")]
pub async fn update_post(
    id: i32,
    post_update: Json<PostUpdate>,
    conn: LegendDb,
) -> Option<Json<Post>> {
    let result = Post::update(id, post_update.into_inner(), &conn)
        .await
        .unwrap();

    result.map(Json)
}

#[delete("/posts/<id>")]
pub async fn delete_post(id: i32, conn: LegendDb) -> Option<NoContent> {
    let result = Post::delete(id, &conn).await.unwrap();

    result.then(|| NoContent)
}
