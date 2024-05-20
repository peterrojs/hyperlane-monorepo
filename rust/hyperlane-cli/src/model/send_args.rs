use clap::ArgMatches;

pub struct SendArgs {
    pub domain: u32,
    pub address: String,
    pub message: String,
}

impl SendArgs {
    pub(crate) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            domain: matches
                .get_one::<String>("domain")
                .expect("Error getting destination domain/chain")
                .parse::<u32>()
                .unwrap(),
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
