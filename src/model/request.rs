use serde::Serialize;

#[derive(Serialize)]
pub struct Request {
    pub action: String,
    pub query: String,
}