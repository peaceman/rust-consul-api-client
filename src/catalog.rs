use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::agent::{AgentCheck, AgentService};
use crate::common::{WriteMeta, WriteOptions};
use crate::errors::Error;
use crate::request::put;
use crate::Client;

#[allow(non_snake_case)]
#[serde(default)]
#[derive(Eq, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct CatalogRegistration {
    pub ID: String,
    pub Node: String,
    pub Address: String,
    pub TaggedAddresses: HashMap<String, String>,
    pub NodeMeta: HashMap<String, String>,
    pub Datacenter: String,
    pub Service: Option<AgentService>,
    pub Check: Option<AgentCheck>,
    pub SkipNodeUpdate: bool,
}

#[async_trait]
pub trait Catalog {
    async fn register(
        &self,
        reg: &CatalogRegistration,
        q: Option<&WriteOptions>,
    ) -> Result<(serde::de::IgnoredAny, WriteMeta), Error>;
}

#[async_trait]
impl Catalog for Client {
    async fn register(
        &self,
        reg: &CatalogRegistration,
        q: Option<&WriteOptions>,
    ) -> Result<(serde::de::IgnoredAny, WriteMeta), Error> {
        put(
            &self.http_client,
            &self.config,
            "/v1/catalog/register",
            Some(reg),
            HashMap::new(),
            q,
        )
        .await
    }
}
