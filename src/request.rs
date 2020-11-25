use std::collections::HashMap;
use std::time::Instant;

use http::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;

use crate::{Config};
use crate::common::{QueryMeta, QueryOptions, WriteMeta, WriteOptions};
use crate::errors::{Error, ResponseError};

pub async fn get<R: DeserializeOwned>(
    http_client: &HttpClient,
    config: &Config,
    path: &str,
    mut params: HashMap<String, String>,
    options: Option<&QueryOptions>,
) -> Result<(R, QueryMeta), Error> {
    params.fill(config, options);

    let mut url = config.address.join(path)?;
    url.query_pairs_mut().extend_pairs(params);

    let request_builder = http_client.get(url);
    let start = Instant::now();
    let response = request_builder.send().await?;

    let consul_index = parse_consul_index(response.headers())?;

    let json = response
        .json()
        .await?;

    Ok((
        json,
        QueryMeta {
            last_index: consul_index,
            request_time: Instant::now() - start,
        },
    ))
}

pub async fn get_vec<R: DeserializeOwned>(
    http_client: &HttpClient,
    config: &Config,
    path: &str,
    mut params: HashMap<String, String>,
    options: Option<&QueryOptions>,
) -> Result<(Vec<R>, QueryMeta), Error> {
    params.fill(config, options);

    let mut url = config.address.join(path)?;
    url.query_pairs_mut().extend_pairs(params);

    let request_builder = http_client.get(url);
    let start = Instant::now();
    let response = request_builder.send().await?;

    let consul_index = parse_consul_index(response.headers())?;

    let json = if response.status() != StatusCode::NOT_FOUND {
        response
            .json()
            .await?
    } else {
        vec![]
    };

    Ok((
        json,
        QueryMeta {
            last_index: consul_index,
            request_time: Instant::now() - start,
        },
    ))
}

pub async fn put<T: Serialize, R: DeserializeOwned>(
    http_client: &HttpClient,
    config: &Config,
    path: &str,
    body: Option<&T>,
    params: HashMap<String, String>,
    options: Option<&WriteOptions>,
) -> Result<(R, WriteMeta), Error> {
    let req = |http_client: &HttpClient, url: Url| -> RequestBuilder { http_client.put(url) };

    write_with_body(http_client, config, path, body, params, options, req).await
}

async fn write_with_body<T: Serialize, R: DeserializeOwned, F>(
    http_client: &HttpClient,
    config: &Config,
    path: &str,
    body: Option<&T>,
    mut params: HashMap<String, String>,
    options: Option<&WriteOptions>,
    req: F,
) -> Result<(R, WriteMeta), Error>
where
    F: Fn(&HttpClient, Url) -> RequestBuilder,
{
    params.fill(config, options);

    let mut url = config.address.join(path)?;
    url.query_pairs_mut().extend_pairs(params);

    let builder = req(http_client, url);
    let builder = if let Some(b) = body {
        builder.json(b)
    } else {
        builder
    };

    let start = Instant::now();
    let response = builder.send().await?;

    let json = response
        .json()
        .await?;

    Ok((
        json,
        WriteMeta {
            request_time: Instant::now() - start,
        },
    ))
}

fn parse_consul_index(headers: &HeaderMap<HeaderValue>) -> Result<Option<u64>, Error> {
    let r = headers
        .get("X-Consul-Index")
        .map(|bytes: &HeaderValue| -> Result<u64, Error> {
            std::str::from_utf8(bytes.as_bytes())
                .map_err(|_| ResponseError::InvalidHeaders(vec!["Invalid UTF-8 in X-Consul-Index header"]))
                .and_then(|s: &str| -> Result<u64, _> {
                    s.parse()
                        .map_err(|_| ResponseError::InvalidHeaders(vec!["Failed to parse X-Consul-Index from header"]))
                })
                .map_err(|e| e.into())
        });

    Ok(match r {
        Some(v) => Some(v?),
        None => None,
    })
}

trait Fillable<T> {
    fn fill(&mut self, config: &Config, options: Option<&T>);
}

impl Fillable<QueryOptions> for HashMap<String, String> {
    fn fill(&mut self, config: &Config, options: Option<&QueryOptions>) {
        let datacenter = options
            .and_then(|o| o.datacenter.as_ref())
            .or_else(|| config.datacenter.as_ref());

        if let Some(dc) = datacenter {
            self.insert("dc".into(), dc.into());
        }

        if let Some(options) = options {
            if let Some(index) = options.wait_index {
                self.insert("index".into(), index.to_string());
            }

            if let Some(wait_time) = options.wait_time {
                self.insert("wait".into(), format!("{}s", wait_time.as_secs()));
            }
        }
    }
}

impl Fillable<WriteOptions> for HashMap<String, String> {
    fn fill(&mut self, config: &Config, options: Option<&WriteOptions>) {
        let datacenter = options
            .and_then(|o| o.datacenter.as_ref())
            .or_else(|| config.datacenter.as_ref());

        if let Some(dc) = datacenter {
            self.insert("dc".into(), dc.into());
        }
    }
}
