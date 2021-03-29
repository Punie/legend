use std::io::Cursor;

use async_graphql::http::MultipartOptions;
use async_graphql::{ObjectType, ParseRequestError, Schema, SubscriptionType};
use rocket::{
    data::{self, Data, FromData, ToByteUnit},
    http::{ContentType, Header, Status},
    response::{self, Responder},
};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Debug)]
pub struct BatchRequest(pub async_graphql::BatchRequest);

impl BatchRequest {
    pub async fn execute<Query, Mutation, Subscription>(
        self,
        schema: &Schema<Query, Mutation, Subscription>,
    ) -> Response
    where
        Query: ObjectType + 'static,
        Mutation: ObjectType + 'static,
        Subscription: SubscriptionType + 'static,
    {
        Response(schema.execute_batch(self.0).await)
    }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for BatchRequest {
    type Error = ParseRequestError;

    async fn from_data(req: &'r rocket::Request<'_>, data: Data) -> data::Outcome<Self, Self::Error> {
        let opts: MultipartOptions = req.managed_state().copied().unwrap_or_default();

        let request = async_graphql::http::receive_batch_body(
            req.headers().get_one("Content-Type"),
            data.open(
                req.limits()
                    .get("graphql")
                    .unwrap_or_else(|| 128.kibibytes()),
            )
            .compat(),
            opts,
        )
        .await;

        match request {
            Ok(request) => data::Outcome::Success(Self(request)),
            Err(e) => data::Outcome::Failure((
                match e {
                    ParseRequestError::PayloadTooLarge => Status::PayloadTooLarge,
                    _ => Status::BadRequest,
                },
                e,
            )),
        }
    }
}

#[derive(Debug)]
pub struct Response(pub async_graphql::BatchResponse);

impl From<async_graphql::BatchResponse> for Response {
    fn from(batch: async_graphql::BatchResponse) -> Self {
        Self(batch)
    }
}
impl From<async_graphql::Response> for Response {
    fn from(res: async_graphql::Response) -> Self {
        Self(res.into())
    }
}

impl<'r> Responder<'r, 'static> for Response {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        let body = serde_json::to_string(&self.0).unwrap();

        let mut response = rocket::Response::new();
        response.set_header(ContentType::new("application", "json"));

        if self.0.is_ok() {
            if let Some(cache_control) = self.0.cache_control().value() {
                response.set_header(Header::new("cache-control", cache_control));
            }
            for (name, value) in self.0.http_headers() {
                response.adjoin_header(Header::new(name.to_string(), value.to_string()));
            }
        }

        response.set_sized_body(body.len(), Cursor::new(body));

        Ok(response)
    }
}
