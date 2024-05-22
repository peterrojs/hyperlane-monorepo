use color_eyre::owo_colors::OwoColorize;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize)]
pub struct GraphQLQuery {
    pub query: String,
    pub variables: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
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

impl Display for SearchQueryResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
                f,
                "{}{}: destination: {}, id: {}, nonce: {}, origin: {}, origin_mailbox: {}, origin_tx_id: {}, recipient: {}, sender: {}, time_created: {}, msg_body: {})",
                "Message ID: ".green().bold(),
                self.msg_id.green().bold(),
                self.destination,
                self.id,
                self.nonce,
                self.origin,
                self.origin_mailbox,
                self.origin_tx_id,
                self.recipient,
                self.sender,
                self.time_created,
                self.msg_body,
            )
    }
}
