pub mod agent;
pub mod catalog;
pub mod health;
pub mod kv;
mod request;
mod common;
mod errors;

use std::time::Duration;

use reqwest::{ClientBuilder};
use url::Url;

use self::errors::*;

#[derive(Debug)]
pub struct Client {
    config: Config,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(config: Config) -> Result<Self, Error> {
        Ok(Self {
            config,
            http_client: ClientBuilder::new().build()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub address: Url,
    pub datacenter: Option<String>,
    pub wait_time: Option<Duration>,
}

#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    address: Option<String>,
    datacenter: Option<String>,
    wait_time: Option<Duration>,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl ConfigBuilder {
    pub fn address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }

    pub fn datacenter(mut self, datacenter: String) -> Self {
        self.datacenter = Some(datacenter);
        self
    }

    pub fn build(self) -> Result<Config, Error> {
        Ok(Config {
            address: self.address
                .ok_or(Error::MissingClientConfig("address"))
                .and_then(|address| Url::parse(&address).map_err(|e| e.into()))?,
            datacenter: self.datacenter,
            wait_time: self.wait_time,
        })
    }
}

