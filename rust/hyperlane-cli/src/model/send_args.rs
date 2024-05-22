use clap::ArgMatches;

pub struct SendArgs {
    pub mailbox: String,
    pub rpc_url: String,
    pub domain: u32,
    pub address: String,
    pub message: String,
}

impl SendArgs {
    pub(crate) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            mailbox: matches
                .get_one::<String>("mailbox")
                .expect("Error getting mailbox address")
                .parse()
                .unwrap(),
            rpc_url: matches
                .get_one::<String>("url")
                .expect("Error getting RPC URL")
                .parse()
                .unwrap(),
            domain: matches
                .get_one::<String>("domain")
                .expect("Error getting destination domain/chain")
                .parse::<u32>()
                .expect("Domain is not a number"),
            address: matches
                .get_one::<String>("address")
                .expect("Error getting destination address")
                .parse()
                .unwrap(),
            message: matches
                .get_one::<String>("message")
                .expect("Error getting message")
                .parse()
                .unwrap(),
        }
    }
}
