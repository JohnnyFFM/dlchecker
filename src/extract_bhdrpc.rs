extern crate jsonrpc;
extern crate serde;
use pbr::ProgressBar;

#[serde(default)]
#[derive(Serialize, Deserialize, Default)]
pub struct Block {
    #[serde(skip_serializing)]
    hash: String,
    #[serde(skip_serializing)]
    confirmations: u64,
    #[serde(skip_serializing)]
    #[serde(rename = "strippedsize")]
    stripped_size: u64,
    #[serde(skip_serializing)]
    size: u64,
    #[serde(skip_serializing)]
    weight: u64,
    height: u64,
    #[serde(skip_serializing)]
    version: u64,
    #[serde(rename = "versionHex")]
    #[serde(skip_serializing)]
    version_hex: String,
    #[serde(skip_serializing)]
    #[serde(rename = "merkleroot")]
    merkle_root: String,
    #[serde(skip_serializing)]
    tx: Vec<String>,
    time: u64,
    #[serde(skip_serializing)]
    #[serde(rename = "mediantime")]
    median_time: u64,
    #[serde(skip_serializing)]
    difficulty: f64,
    #[serde(skip_serializing)]
    chainwork: String,
    #[serde(rename = "baseTarget")]
    base_target: u64,
    #[serde(rename = "plotterId")]
    plotter_id: u64,
    nonce: u64,
    #[serde(rename = "generationSignature")]
    generation_signature: String,
    deadline: u64,
    #[serde(skip_serializing)]
    generator: String,
    #[serde(skip_serializing)]
    previousblockhash: String,
    #[serde(skip_serializing)]
    nextblockhash: String,
}

pub fn extract_bhdrpc(matches: &clap::ArgMatches) {
    // read cookie
    let cookiepath = value_t!(matches, "cookie", String).unwrap_or_else(|e| e.exit());
    let cookiepath = std::path::Path::new(&cookiepath);
    let cookiepath = cookiepath.join(".cookie");
    println!("Cookie Path: {:?}", cookiepath);
    let cookie: String = std::fs::read_to_string(cookiepath)
        .expect("can't read cookie file")
        .parse()
        .expect("can't parse cookie file");
    let pass = &cookie[11..];
    // start rpc
    let wallet = value_t!(matches, "wallet", String).unwrap_or_else(|e| e.exit());
    let client =
        jsonrpc::client::Client::new(wallet, Some("__cookie__".to_owned()), Some(pass.to_owned()));
    let request = client.build_request("getblockcount", &[]);
    // query block count
    let blockcount = match client
        .send_request(&request)
        .and_then(|res| res.into_result::<u64>())
    {
        Ok(res) => res, // Ok!
        Err(e) => {
            println!("{:?}", e);
            0u64
        }
    };
    println!("Blockcount : {}", blockcount);

    // initialise write
    let filename = format!("bhd_{}.csv", blockcount);
    println!("Filename   : {}", &filename);
    let mut wtr = csv::Writer::from_path(&filename).unwrap();

    let mut pb = ProgressBar::new(blockcount);
    pb.format("│██░│");
    pb.set_width(Some(80));
    pb.message("Extracting : ");

    // extract blocks
    for i in 0..blockcount as u64 {
        let param = vec![serde_json::to_value(i).unwrap()];
        let request = client.build_request("getblockhash", &param);
        let blockhash = match client
            .send_request(&request)
            .and_then(|res| res.into_result::<String>())
        {
            Ok(res) => res, // Ok!
            Err(e) => {
                println!("{:?}", e);
                "".to_owned()
            }
        };
        let param = vec![serde_json::to_value(blockhash).unwrap()];
        let request = client.build_request("getblock", &param);
        let block = match client
            .send_request(&request)
            .and_then(|res| res.into_result::<Block>())
        {
            Ok(res) => res, // Ok!
            Err(e) => {
                println!("{:?}", e);
                Block::default()
            }
        };
        // export to csv
        wtr.serialize(block)
            .expect("error deserialising block info");
        pb.inc();
    }
    pb.finish_print("done.");
}
