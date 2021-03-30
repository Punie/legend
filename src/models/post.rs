use async_graphql::{InputObject, SimpleObject};
use rocket::trace;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use crate::{database::LegendDb, Result};

#[derive(SimpleObject, FromRow, Serialize, Clone, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(InputObject, Deserialize, Clone, Debug)]
pub struct NewPost {
    pub title: String,
    pub body: String,
}

#[derive(InputObject, Deserialize, Clone, Debug)]
pub struct PostUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub published: Option<bool>,
}

impl Post {
    #[trace::instrument(name = "Post::all", level = "info", skip(conn))]
    pub async fn all(conn: &LegendDb) -> Result<Vec<Post>> {
        let res = sqlx::query_as!(Post, r#"SELECT * FROM posts WHERE published ORDER BY id"#)
            .fetch_all(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::find_by_id", level = "info", skip(conn))]
    pub async fn find_by_id(id: i32, conn: &LegendDb) -> Result<Option<Post>> {
        let res = sqlx::query_as!(Post, r#"SELECT * FROM posts WHERE id = $1"#, id)
            .fetch_optional(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::create", level = "info", skip(conn))]
    pub async fn create(new_post: NewPost, conn: &LegendDb) -> Result<Post> {
        let res = sqlx::query_as!(
            Post,
            r#"INSERT INTO posts (title, body) VALUES ($1, $2) RETURNING *"#,
            new_post.title,
            new_post.body,
        )
        .fetch_one(conn.pool())
        .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::update", level = "info", skip(conn))]
    pub async fn update(id: i32, post_update: PostUpdate, conn: &LegendDb) -> Result<Option<Post>> {
        let res = sqlx::query_as!(
                Post,
                r#"UPDATE posts SET (title, body, published) = (COALESCE($1, title), COALESCE($2, body), COALESCE($3, published)) WHERE id = $4 RETURNING *"#,
                post_update.title,
                post_update.body,
                post_update.published,
                id
            )
            .fetch_optional(conn.pool())
            .await?;

        Ok(res)
    }

    #[trace::instrument(name = "Post::update", level = "info", skip(conn))]
    pub async fn delete(id: i32, conn: &LegendDb) -> Result<bool> {
        let res = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
            .execute(conn.pool())
            .await?;

        Ok(res.rows_affected() > 0)
    }
}
