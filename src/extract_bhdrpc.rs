extern crate jsonrpc;
extern crate serde;

pub fn extract_bhdrpc(matches: &clap::ArgMatches){
    // read cookie
    let cookiepath = value_t!(matches, "cookie", String).unwrap_or_else(|e| e.exit());
    let cookiepath = std::path::Path::new(&cookiepath);
    let cookiepath = cookiepath.join(".cookie");
    println!("Cookie Path: {:?}", cookiepath);
    let cookie: String = std::fs::read_to_string(cookiepath).expect("can't read cookie file").parse().expect("can't parse cookie file");
    let pass = &cookie[11..];
    // start rpc
    let wallet = value_t!(matches, "wallet", String).unwrap_or_else(|e| e.exit());
    let client = jsonrpc::client::Client::new(wallet, Some("__cookie__".to_owned()), Some(pass.to_owned()));
    let request = client.build_request("getblockcount", &[]);
    let blockcount = match client.send_request(&request).and_then(|res| res.into_result::<u64>()) {
        Ok(res) => res, // Ok!
        Err(e) => {
            println!("{:?}",e);
            0u64
        }
    };
    // query block count
    println!("Blockcount : {}", blockcount);
    // extract blocks
    // export to csv
}