use axum::{
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Request},
    http::{header::{HeaderValue, USER_AGENT}, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};

struct UserAgent(HeaderValue);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.headers.get(USER_AGENT).cloned().map(Self)
            .ok_or((StatusCode::BAD_REQUEST, "User-Agent is required"))
    }
}

struct NonemptyBody(Bytes);

impl<S> FromRequest<S> for NonemptyBody
where
    S: Send + Sync,
    Bytes: FromRequest<S>,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await
            .map_err(IntoResponse::into_response)?;
        if bytes.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Body must not be empty").into_response());
        }
        Ok(Self(bytes))
    }
}

async fn echo(UserAgent(agent): UserAgent, NonemptyBody(body): NonemptyBody) -> impl IntoResponse {
    ([("x-client-agent", agent)], body)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/echo", post(echo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}
