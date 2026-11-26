use anyhow::{Context, Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

#[derive(Debug)]
pub(super) struct Rejected(pub u16);
impl std::fmt::Display for Rejected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "controller returned {}", self.0)
    }
}
impl std::error::Error for Rejected {}
#[derive(Clone)]
pub(super) struct Control {
    pub origin: String,
    client: reqwest::Client,
}
impl Control {
    pub fn new(origin: &str) -> Result<Self> {
        let url = reqwest::Url::parse(origin)?;
        ensure!(
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
                && matches!(url.path(), "" | "/")
                && url.query().is_none()
                && url.fragment().is_none(),
            "controller must be an HTTP(S) origin"
        );
        Ok(Self {
            origin: origin.trim_end_matches('/').into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()?,
        })
    }
    pub async fn routes(&self) -> Result<crate::proto::wafl::Routing> {
        let response = self
            .client
            .get(format!("{}/proxy", self.origin))
            .send()
            .await?
            .error_for_status()?;
        let bytes = response.bytes().await?;
        ensure!(
            bytes.len() <= 16 * 1024 * 1024,
            "routing snapshot too large"
        );
        let routes: crate::proto::wafl::Routing = serde_json::from_slice(&bytes)?;
        ensure!(routes.version == 1, "unsupported routing version");
        Ok(routes)
    }
    pub async fn rpc<T: DeserializeOwned>(
        &self,
        method: &str,
        tenant: &str,
        request: &impl Serialize,
    ) -> Result<T> {
        let response = self
            .client
            .post(format!("{}/wafl.{method}", self.origin))
            .header("content-type", "application/json")
            .header("connect-protocol-version", "1")
            .header("wafl-tenant", tenant)
            .body(serde_json::to_vec(request)?)
            .send()
            .await?;
        let status = response.status();
        // Keep remote objects, manifests and environment values out of proxy logs.
        if !status.is_success() {
            return Err(Rejected(status.as_u16()).into());
        }
        serde_json::from_slice(&response.bytes().await?).context("invalid controller response")
    }
}
