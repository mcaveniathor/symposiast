use axum::body::{to_bytes, Body};
use http_body_util::LengthLimitError;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), axum::Error> {
    let bytes = to_bytes(Body::from(vec![1_u8, 2, 3]), 3).await?;
    assert_eq!(&bytes[..], &[1, 2, 3]);

    let error = to_bytes(Body::from(vec![1_u8, 2, 3]), 1)
        .await
        .expect_err("three bytes exceed a one-byte limit");
    assert!(error.source().is_some_and(|source| source.is::<LengthLimitError>()));
    Ok(())
}
