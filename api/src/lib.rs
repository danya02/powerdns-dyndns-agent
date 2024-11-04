#[derive(serde::Serialize, serde::Deserialize)]
pub struct UpdateRequest {
    pub name: String,
    pub content: String,
}
