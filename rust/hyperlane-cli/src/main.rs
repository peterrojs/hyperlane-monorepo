mod api;
mod interfaces;
mod model;

use crate::model::matching_list::MatchingList;
use crate::model::send_args::SendArgs;
use clap::{arg, command, Command};
use colored::Colorize;
use ethers::core::rand::rngs::OsRng;
use ethers::prelude::{LocalWallet, Signer};
use serde_json::from_str;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    let matches = command!()
        .about("A CLI tool for interacting with Hyperlane")
        .subcommand(
            Command::new("send")
                .about("Dispatches a message")
                .arg(arg!(-w --wallet <WALLET> "Sets the private key to use"))
                .arg(arg!(-c --chain <CHAIN> "Chain ID for EIP-155").required(true))
                .arg(arg!(-m --mailbox <MAILBOX> "Sets the mailbox address").required(true))
                .arg(arg!(-u --url <URL> "RPC URL to send the message to").required(true))
                .arg(arg!(-d --domain <DOMAIN> "Sets the destination chain domain").required(true))
                .arg(arg!(-a --address <ADDRESS> "Sets the destination address").required(true))
                .arg(
                    arg!(-b --message <MESSAGE> "Message bytes in hexadecimal format")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("Queries for messages sent from a specified chain")
                .arg(arg!(-l --list <LIST> "MatchingList for the query. Should be in the following format: [{\"origindomain\": \"<VALUE>\", \"senderaddress\": \"<VALUE>\", \"destinationdomain\": \"<VALUE>\", \"recipientaddress\": \"<VALUE>\"}]").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("send", send_matches)) => {
            let wallet_arg = send_matches.get_one::<String>("wallet");
            let chain_id = send_matches
                .get_one::<String>("chain")
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse chain ID to number");
            let wallet = match wallet_arg {
                Some(wallet_key) => LocalWallet::from_str(
                    wallet_key
                        .strip_prefix("0x")
                        .expect("Wrongfully formatted private key"),
                )
                .expect("Failed to parse private key")
                .with_chain_id(chain_id),

                None => {
                    println!("{}", "No wallet provided, generating a new one".bold());
                    LocalWallet::new(&mut OsRng).with_chain_id(chain_id)
                }
            };

            let args = SendArgs::from_matches(send_matches);
            api::send_message(wallet, args).await;
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
