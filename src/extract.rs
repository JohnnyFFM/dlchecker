use crate::extract_bhdrpc::extract_bhdrpc;
arg_enum! {
    #[derive(Debug)]
    pub enum WalletType {
        BurstHttp,
        BurstDB,
        BhdRpc,
    }
}

pub fn extract(matches: &clap::ArgMatches) {
    let wallet_type =
        value_t!(matches.value_of("wallet_type"), WalletType).unwrap_or_else(|e| e.exit());
    println!("Wallet Type: {:?}", wallet_type);
    match wallet_type {
        WalletType::BurstHttp => {}
        WalletType::BurstDB => {}
        WalletType::BhdRpc => extract_bhdrpc(matches),
    }
}
