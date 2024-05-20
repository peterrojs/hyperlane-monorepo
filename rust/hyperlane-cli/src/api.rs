use crate::interfaces::i_mailbox::IMailbox;
use crate::model::matching_list::MatchingList;
use crate::model::send_args::SendArgs;
use anyhow::Result;
use ethers::abi::Address;
use ethers::contract::ContractError;
use ethers::core::types::Bytes;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{LocalWallet, PendingTransaction};
use ethers::providers::{Http, Provider};
use std::str::FromStr;
use std::sync::Arc;

pub async fn send_message(args: SendArgs, wallet: LocalWallet) {
    let provider =
        Provider::<Http>::try_from(args.rpc_url).expect("Failed to create provider from RPC URL");
    let client = SignerMiddleware::new(provider, wallet);
    let contract_address =
        Address::from_str(&args.mailbox).expect("Failed to parse contract address");
    let contract = IMailbox::new(contract_address, Arc::new(client));

    let mut fixed_address_array = [0u8; 32];
    fixed_address_array.copy_from_slice(args.address.as_bytes());
    let message_body = Bytes::from_str(&*args.message).expect("Failed to parse message bytes");

    match contract
        .dispatch_0(args.domain, fixed_address_array, message_body)
        .send()
        .await
    {
        Ok(pending_transaction) => {
            println!("Transaction sent: {:?}", pending_transaction);
        }
        Err(e) => {
            println!("Failed to send transaction: {:?}", e);
        }
    }
}

pub async fn perform_search(matching_list: MatchingList) {}
