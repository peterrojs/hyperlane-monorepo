use crate::interfaces::i_mailbox::IMailbox;
use crate::model::graph_ql::{GraphQLQuery, GraphQLResponse, Message, SearchQueryResponse};
use crate::model::matching_list::MatchingList;
use crate::model::send_args::SendArgs;
use color_eyre::owo_colors::OwoColorize;
use colored::Colorize;
use ethers::abi::Address;
use ethers::core::types::Bytes;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::LocalWallet;
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
    let query_template = r#"
    query Message(
      $destinationdomain: [Int!]
      $origindomain: [Int!],
      $recipientaddress: [bytea!],
      $senderaddress: [bytea!],
    ) {
      message(
        where: {
          sender: {_in: $senderaddress},
          recipient: {_in: $recipientaddress},
          origin: {_in: $origindomain}
          destination: {_in: $destinationdomain}
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

    let mut queries = Vec::new();

    if let Some(mut list_element) = matching_list.0 {
        for element in list_element.iter_mut() {
            let variables = serde_json::to_value(&element).unwrap();
            queries.push(GraphQLQuery {
                query: query_template.to_string(),
                variables,
            });
        }
    }

    println!("{}", "Querying...".bold());

    match send_graphql_request::<Vec<GraphQLResponse<Message<Vec<SearchQueryResponse>>>>>(
        "https://api.hyperlane.xyz/v1/graphql",
        &queries,
    )
    .await
    {
        Ok(data) => {
            for (i, response) in data.iter().enumerate() {
                if response.data.message.is_empty() {
                    println!("No results found for query");
                    return;
                }
                println!("{} {}", "\nResult for query:".bold(), (i + 1).bold());

                for message in response.data.message.iter() {
                    println!("{}", message);
                }
            }
        }
        Err(e) => println!("Error sending request: {}", e),
    }
}

async fn send_graphql_request<T: for<'de> serde::Deserialize<'de>>(
    endpoint: &str,
    queries: &[GraphQLQuery],
) -> Result<T, reqwest::Error> {
    let client = reqwest::Client::new();

    let res = client
        .post(endpoint)
        .json(queries)
        .send()
        .await?
        .json::<T>()
        .await?;

    Ok(res)
}
