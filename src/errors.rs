use thiserror::Error as ThisError;
use url::ParseError as UrlParseError;
use reqwest::Error as ReqwestError;
use http::{StatusCode, HeaderMap};
use serde_json::Error as JsonError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Missing config key {0}")]
    MissingClientConfig(&'static str),
    #[error("Failed to parse consul endpoint url {0:?}")]
    InvalidConsulEndpoint(#[from] UrlParseError),
    #[error("Request error: {0:?}")]
    RequestError(#[from] ReqwestError),
    #[error("Response error: {0:?}")]
    ResponseError(#[from] ResponseError),
}

#[derive(Debug, ThisError)]
pub enum ResponseError {
    #[error("Unexpected status code {status:?}")]
    UnexpectedStatus {
        status: StatusCode,
        headers: HeaderMap,
        body: String,
    },
    #[error("Invalid payload: {0}")]
    InvalidPayload(#[from] JsonError),
    #[error("Missing or invalid response headers: {}", strs_to_str(.0))]
    InvalidHeaders(Vec<&'static str>),
}

fn strs_to_str(strs: &[&'static str]) -> String {
    strs.join(", ")
}