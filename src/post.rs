use async_graphql::{InputObject, SimpleObject};
use diesel::{prelude::*, result::Error};
use rocket::trace;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use crate::schema::posts;
use crate::{
    db::{LegendDb, LegendDbSqlx},
    debug_query,
};

#[derive(SimpleObject, Queryable, Identifiable, FromRow, Serialize, Clone, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(InputObject, Insertable, Deserialize, Clone, Debug)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub body: String,
}

#[derive(InputObject, AsChangeset, Deserialize, Clone, Debug)]
#[table_name = "posts"]
pub struct PostUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub published: Option<bool>,
}

impl Post {
    #[trace::instrument(name = "Post::all", level = "info", skip(conn))]
    pub async fn all(conn: &LegendDb) -> QueryResult<Vec<Post>> {
        let query = posts::table.filter(posts::published).order(posts::id);

        debug_query!(query);

        conn.run(move |c| query.load(c)).await
    }

    pub async fn all_sqlx(conn: &LegendDbSqlx) -> color_eyre::Result<Vec<Post>> {
        let res = sqlx::query_as("SELECT * FROM posts WHERE published ORDER BY id")
            .fetch_all(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::find_by_id", level = "info", skip(conn))]
    pub async fn find_by_id(id: i32, conn: &LegendDb) -> QueryResult<Option<Post>> {
        let query = posts::table.find(id);

        debug_query!(query);

        let result = conn.run(move |c| query.get_result(c)).await;

        match result {
            Ok(post) => Ok(Some(post)),
            Err(Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn find_by_id_sqlx(id: i32, conn: &LegendDbSqlx) -> color_eyre::Result<Option<Post>> {
        let res = sqlx::query_as("SELECT * FROM posts WHERE id = $1")
            .bind(id)
            .fetch_optional(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::create", level = "info", skip(conn))]
    pub async fn create(new_post: NewPost, conn: &LegendDb) -> QueryResult<Post> {
        let query = diesel::insert_into(posts::table).values(new_post);

        debug_query!(query);

        conn.run(|c| query.get_result(c)).await
    }

    pub async fn create_sqlx(new_post: NewPost, conn: &LegendDbSqlx) -> color_eyre::Result<Post> {
        let res = sqlx::query_as("INSERT INTO posts (title, body) VALUES ($1, $2) RETURNING *")
            .bind(new_post.title)
            .bind(new_post.body)
            .fetch_one(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::update", level = "info", skip(conn))]
    pub async fn update(
        id: i32,
        post_update: PostUpdate,
        conn: &LegendDb,
    ) -> QueryResult<Option<Post>> {
        let query = diesel::update(posts::table.find(id)).set(post_update);

        debug_query!(query);

        let result = conn.run(move |c| query.get_result(c)).await;

        match result {
            Ok(post) => Ok(Some(post)),
            Err(Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn update_sqlx(
        id: i32,
        post_update: PostUpdate,
        conn: &LegendDbSqlx,
    ) -> color_eyre::Result<Option<Post>> {
        let res = sqlx::query_as("UPDATE posts SET (title, body, published) = (COALESCE($1, title), COALESCE($2, body), COALESCE($3, published)) WHERE id = $4 RETURNING *")
            .bind(post_update.title)
            .bind(post_update.body)
            .bind(post_update.published)
            .bind(id)
            .fetch_optional(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::update", level = "info", skip(conn))]
    pub async fn delete(id: i32, conn: &LegendDb) -> QueryResult<bool> {
        let query = diesel::delete(posts::table.find(id));

        debug_query!(query);

        conn.run(move |c| query.execute(c).map(|n| n > 0)).await
    }

    pub async fn delete_sqlx(id: i32, conn: &LegendDbSqlx) -> color_eyre::Result<bool> {
        let res = sqlx::query("DELETE FROM posts WHERE id = $1")
            .bind(id)
            .execute(conn.pool())
            .await?;

        Ok(res.rows_affected() > 0)
    }
}
