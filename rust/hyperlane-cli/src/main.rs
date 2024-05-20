mod api;
mod interfaces;
mod model;

use crate::model::matching_list::MatchingList;
use crate::model::send_args::SendArgs;
use clap::{arg, command, Command};
use ethers::core::rand::rngs::OsRng;
use ethers::prelude::LocalWallet;
use serde_json::from_str;

#[tokio::main]
async fn main() {
    /// Creates a new wallet
    let wallet = LocalWallet::new(&mut OsRng);

    let matches = command!()
        .about("A CLI tool for interacting with Hyperlane")
        .subcommand(
            Command::new("send")
                .about("Dispatches a message")
                .arg(arg!(-m --mailbox <MAILBOX> "Sets the mailbox address").required(true))
                .arg(arg!(-d --domain <DOMAIN> "Sets the destination chain domain").required(true))
                .arg(arg!(-a --address <ADDRESS> "Sets the destination address").required(true))
                .arg(arg!(-u --url <URL> "RPC URL to send the message to").required(true))
                .arg(
                    arg!(-b --message <MESSAGE> "Message bytes in hexadecimal format")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("Queries for messages sent from a specified chain")
                .arg(arg!(-u --url <URL> "RPC URL to send the message to").required(true))
                .arg(arg!(-l --list "MatchingList for the query").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("send", send_matches)) => {
            let args = SendArgs::from_matches(send_matches);
            api::send_message(args, wallet).await;
        }
        Some(("search", search_matches)) => {
            let json_str = search_matches.get_one::<String>("list").unwrap();
            let matching_list: MatchingList =
                from_str::<MatchingList>(json_str).expect("Failed to parse MatchingList");
            api::perform_search(matching_list).await;
        }
        _ => {
            println!("Pass a valid subcommand. Use --help for more information.");
        }
    }
}
