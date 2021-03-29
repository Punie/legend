use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, Object, Schema,
};
use rocket::{
    fairing::AdHoc,
    response::{
        content::Html,
        status::{Created, NoContent},
    },
    trace, State,
};
use rocket_contrib::json::Json;

use crate::{
    db::{LegendDb, LegendDbSqlx},
    gql::{BatchRequest, Response},
    post::{NewPost, Post, PostUpdate},
};

type LegendSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, name: String) -> String {
        format!("Hello {}! o/", name)
    }

    async fn posts(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Post>> {
        let conn = ctx.data::<LegendDbSqlx>()?;
        let posts = Post::all_sqlx(conn).await.unwrap();

        Ok(posts)
    }

    async fn post(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Option<Post>> {
        let conn = ctx.data::<LegendDbSqlx>()?;
        let post = Post::find_by_id_sqlx(id, conn).await.unwrap();

        Ok(post)
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_post(&self, ctx: &Context<'_>, post: NewPost) -> async_graphql::Result<Post> {
        let conn = ctx.data::<LegendDbSqlx>()?;
        let post = Post::create_sqlx(post, conn).await.unwrap();

        Ok(post)
    }

    async fn update_post(&self, ctx: &Context<'_>, id: i32, post: PostUpdate) -> async_graphql::Result<Option<Post>> {
        let conn = ctx.data::<LegendDbSqlx>()?;
        let post = Post::update_sqlx(id, post, conn).await.unwrap();

        Ok(post)
    }

    async fn delete_post(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<bool> {
        let conn = ctx.data::<LegendDbSqlx>()?;
        let res = Post::delete_sqlx(id, conn).await.unwrap();

        Ok(res)
    }
}

#[get("/playground")]
fn graphql_playground() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

// #[get("/graphql?<query..>")]
// async fn graphql_query(schema: State<'_, LegendSchema>, query: Request) -> Response {
//     query.execute(&schema).await
// }

#[post("/", data = "<request>", format = "application/json")]
async fn graphql_request(schema: State<'_, LegendSchema>, request: BatchRequest) -> Response {
    trace::debug!(?request);
    request.execute(&schema).await
}

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
    let location = uri!("/api", get_post: id = result.id).to_string();

    Created::new(location).body(Json(result))
}

#[post("/posts", format = "json", data = "<new_post>")]
async fn create_post_sqlx(new_post: Json<NewPost>, conn: LegendDbSqlx) -> Created<Json<Post>> {
    let result = Post::create_sqlx(new_post.into_inner(), &conn)
        .await
        .unwrap();
    let location = uri!("/api", get_post: id = result.id).to_string();

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
        .attach(AdHoc::on_attach(
            "graphql schema",
            move |rocket| async move {
                let pool = match rocket.state::<LegendDbSqlx>() {
                    Some(pool) => pool.clone(),
                    None => {
                        trace::error!("LegendDbSqlx must be attached first");
                        return Err(rocket);
                    }
                };

                let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
                    .data(pool)
                    .finish();

                Ok(rocket.manage(schema))
            },
        ))
        .mount("/", routes![hello_world, health])
        .mount("/graphql", routes![graphql_playground, graphql_request])
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
