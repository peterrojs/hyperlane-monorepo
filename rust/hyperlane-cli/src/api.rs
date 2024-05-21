use crate::interfaces::i_mailbox::IMailbox;
use crate::model::graph_ql::{GraphQLQuery, GraphQLResponse, Message, SearchQueryResponse};
use crate::model::matching_list::MatchingList;
use crate::model::send_args::SendArgs;
use anyhow::Result;
use colored::Colorize;
use ethers::abi::Address;
use ethers::core::types::Bytes;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{LocalWallet};
use ethers::providers::{Http, Provider};
use std::str::FromStr;
use std::sync::Arc;

/// Sends a message to a mailbox contract
pub async fn send_message(wallet: LocalWallet, args: SendArgs) {
    let provider =
        Provider::<Http>::try_from(args.rpc_url).expect("Failed to create provider from RPC URL");
    let client = SignerMiddleware::new(provider, wallet);
    let contract_address =
        Address::from_str(&*args.mailbox).expect("Failed to parse contract address");
    let contract = IMailbox::new(contract_address, Arc::new(client));

    let mut fixed_address_array = [0u8; 32];
    fixed_address_array.copy_from_slice(args.address.as_bytes());
    let message_body = Bytes::from_str(&*args.message).expect("Failed to parse message bytes");

    println!("{} {}", "Sending message to mailbox:".bold(), args.mailbox);

    match contract
        .dispatch_0(args.domain, fixed_address_array, message_body)
        .send()
        .await
    {
        Ok(pending_transaction) => {
            println!(
                "{} {:?}",
                "Transaction sent:".bold(),
                pending_transaction.to_string().green()
            );
        }
        Err(e) => {
            println!("{} {}", "Failed to send transaction:".bold().red(), e);
        }
    }
}

/// Performs a search for messages based on the provided matching list
pub async fn perform_search(matching_list: MatchingList) {
    let query = r#"
    query Message(
      $senderaddress: bytea,
      $recipientaddress: bytea,
      $origindomain: [Int!],
      $destinationdomain: [Int!]
    ) {
      message(
        where: {
          sender: {_eq: $senderAddress},
          recipient: {_eq: $recipientAddress},
          origin: {_in: $originDomain}
          destination: {_in: $destinationDomain}
        }
        order_by: {time_created: desc}
        limit: 10
      ) {
        destination
        id
        msg_body
        msg_id
        nonce
        origin
        origin_mailbox
        origin_tx_id
        recipient
        sender
        time_created
      }
    }
    "#;

    let variables = matching_list.0
        .map(|list_elements| serde_json::to_value(&list_elements).unwrap())
        .unwrap_or_else(|| {
            println!("MatchingList is empty");
            serde_json::Value::Null
        });

    println!("Variables: {}", variables.clone());

    match send_graphql_request::<Message<Vec<SearchQueryResponse>>>(
        "https://api.hyperlane.xyz/v1/graphql",
        query,
        variables,
    )
    .await
    {
        Ok(data) => {
            let pretty_data =
                serde_json::to_string_pretty(&data.message).expect("Failed to serialize data");
            println!("{}", pretty_data);
        }
        Err(e) => println!("Error sending request: {}", e),
    }
}

async fn send_graphql_request<T: for<'de> serde::Deserialize<'de>>(
    endpoint: &str,
    query: &str,
    variables: serde_json::Value,
) -> Result<T, reqwest::Error> {
    let client = reqwest::Client::new();
    let graphql_query = GraphQLQuery {
        query: query.to_string(),
        variables,
    };

    let res = client
        .post(endpoint)
        .json(&graphql_query)
        .send()
        .await?
        .json::<GraphQLResponse<T>>()
        .await?;

    Ok(res.data)
}
