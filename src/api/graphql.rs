use async_graphql::{
    extensions::Tracing,
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, Object, Schema,
};
use rocket::{
    fairing::{Fairing, Info, Kind},
    response::content::Html,
    tokio::{fs::File, io::AsyncWriteExt},
    trace::{self, Instrument},
    Rocket, State,
};

use crate::{
    database::LegendDb,
    graphql_rocket::{BatchRequest, Query, Response},
    models::post::{NewPost, Post, PostUpdate},
};

pub type LegendSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct GraphQlFairing;

#[rocket::async_trait]
impl Fairing for GraphQlFairing {
    fn info(&self) -> Info {
        Info {
            name: "GraphQL API",
            kind: Kind::Attach,
        }
    }

    async fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let pool = match rocket.state::<LegendDb>() {
            Some(pool) => pool.clone(),
            None => {
                trace::error!("LegendDb fairing must be attached first");
                return Err(rocket);
            }
        };

        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
            .extension(Tracing::default())
            .data(pool)
            .finish();

        let mut file = match File::create("schema.graphql").await {
            Ok(file) => file,
            Err(error) => {
                trace::error!(%error, "Failed to open schema.graphql");
                return Err(rocket);
            }
        };
        match file.write_all(schema.sdl().as_bytes()).await {
            Ok(_) => {}
            Err(error) => {
                trace::error!(%error, "Failed to write schema to schema.graphql");
                return Err(rocket);
            }
        }

        Ok(rocket.manage(schema))
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, name: String) -> String {
        format!("Hello {}! o/", name)
    }

    async fn posts(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Post>> {
        let conn = ctx.data::<LegendDb>()?;
        let posts = Post::all(conn).await?;

        Ok(posts)
    }

    async fn post(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<Option<Post>> {
        let conn = ctx.data::<LegendDb>()?;
        let post = Post::find_by_id(id, conn).await?;

        Ok(post)
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_post(&self, ctx: &Context<'_>, post: NewPost) -> async_graphql::Result<Post> {
        let conn = ctx.data::<LegendDb>()?;
        let post = Post::create(post, conn).await?;

        Ok(post)
    }

    async fn update_post(
        &self,
        ctx: &Context<'_>,
        id: i32,
        post: PostUpdate,
    ) -> async_graphql::Result<Option<Post>> {
        let conn = ctx.data::<LegendDb>()?;
        let post = Post::update(id, post, conn).await?;

        Ok(post)
    }

    async fn delete_post(&self, ctx: &Context<'_>, id: i32) -> async_graphql::Result<bool> {
        let conn = ctx.data::<LegendDb>()?;
        let res = Post::delete(id, conn).await?;

        Ok(res)
    }
}

#[get("/")]
pub fn graphql_playground() -> Html<String> {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[get("/?<query..>")]
pub async fn graphql_query(schema: State<'_, LegendSchema>, query: Query) -> Response {
    trace::debug!(?query);
    let root_span = trace::info_span!("GraphQL request");
    query.execute(&schema).instrument(root_span).await
}

#[post("/", data = "<request>", format = "application/json")]
pub async fn graphql_request(schema: State<'_, LegendSchema>, request: BatchRequest) -> Response {
    trace::debug!(?request);
    let root_span = trace::info_span!("GraphQL request");
    request.execute(&schema).instrument(root_span).await
}
