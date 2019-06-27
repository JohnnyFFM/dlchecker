extern crate jsonrpc;
extern crate serde;

pub fn extract_bhdrpc(matches: &clap::ArgMatches){
    // read cookie
    let cookiepath = value_t!(matches, "cookie", String).unwrap_or_else(|e| e.exit());
    // start rpc
    let wallet = value_t!(matches, "wallet", String).unwrap_or_else(|e| e.exit());
    let client = jsonrpc::client::Client::new(wallet, Some("__cookie__".to_owned()), Some("67d4c4f38d1911cac58df0d1b714db0af9912c4536336f4ecd193c652a674844".to_owned()));
    let request = client.build_request("getblockcount", &[]);
    let blockcount = match client.send_request(&request).and_then(|res| res.into_result::<u64>()) {
        Ok(res) => res, // Ok!
        Err(e) => {
            println!("{:?}",e);
            0u64
        }
    };
    println!("Blockcount : {}", blockcount);
    // query block height
    // extract blocks
    // export to csv
}