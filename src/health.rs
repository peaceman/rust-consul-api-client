use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::agent::AgentService;
use crate::request::get;
use crate::{Client, common::{QueryMeta, QueryOptions}};
use crate::errors::Error;

#[allow(non_snake_case)]
#[serde(default)]
#[derive(Eq, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct HealthCheck {
    pub Node: String,
    pub CheckID: String,
    pub Name: String,
    pub Status: String,
    pub Notes: String,
    pub Output: String,
    pub ServiceID: String,
    pub ServiceName: String,
    pub ServiceTags: Option<Vec<String>>,
}

#[allow(non_snake_case)]
#[serde(default)]
#[derive(Eq, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct Node {
    pub ID: String,
    pub Node: String,
    pub Address: String,
    pub Datacenter: Option<String>,
    pub TaggedAddresses: Option<HashMap<String, String>>,
    pub Meta: Option<HashMap<String, String>>,
    pub CreateIndex: u64,
    pub ModifyIndex: u64,
}

#[allow(non_snake_case)]
#[serde(default)]
#[derive(Eq, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct ServiceEntry {
    pub Node: Node,
    pub Service: AgentService,
    pub Checks: Vec<HealthCheck>,
}

#[async_trait]
pub trait Health {
    async fn service(
        &self,
        service: &str,
        tag: Option<&str>,
        passing_only: bool,
        params: Option<HashMap<String, String>>,
        options: Option<&QueryOptions>,
    ) -> Result<(Vec<ServiceEntry>, QueryMeta), Error>;
}

#[async_trait]
impl Health for Client {
    async fn service(
        &self,
        service: &str,
        tag: Option<&str>,
        passing_only: bool,
        params: Option<HashMap<String, String>>,
        options: Option<&QueryOptions>,
    ) -> Result<(Vec<ServiceEntry>, QueryMeta), Error> {
        let mut params = params.unwrap_or_default();

        if passing_only {
            params.insert("passing".into(), "1".into());
        }

        if let Some(tag) = tag {
            params.insert("tag".into(), tag.into());
        }

        let path = format!("/v1/health/service/{}", service);
        get(&self.http_client, &self.config, &path, params, options).await
    }
}
