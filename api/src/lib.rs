#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateRequest {
    pub name: String,
    pub content: String,
}
