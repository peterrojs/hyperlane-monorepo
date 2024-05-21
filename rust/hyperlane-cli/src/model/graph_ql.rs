use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GraphQLQuery {
    pub query: String,
    pub variables: serde_json::Value,
}

#[derive(Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message<T> {
    pub message: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchQueryResponse {
    pub destination: i32,
    pub id: i64,
    #[serde(rename = "msg_body")]
    pub msg_body: String,
    #[serde(rename = "msg_id")]
    pub msg_id: String,
    pub nonce: i64,
    pub origin: i32,
    #[serde(rename = "origin_mailbox")]
    pub origin_mailbox: String,
    #[serde(rename = "origin_tx_id")]
    pub origin_tx_id: i64,
    pub recipient: String,
    pub sender: String,
    pub time_created: String,
}
