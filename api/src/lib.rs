use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateRequest {
    pub name: String,
    pub content: String,

    #[serde_as(as = "Base64")]
    pub verifying_key: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HostsConfig {
    pub hosts: Vec<SingleHostConfig>,
}

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SingleHostConfig {
    #[serde_as(as = "Base64")]
    pub verifying_key: Vec<u8>,
    pub allowed_hosts: Vec<String>,
}
