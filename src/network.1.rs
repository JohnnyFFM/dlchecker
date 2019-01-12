extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio;
use hyper::rt::{self, Future, Stream};

//use futures::{future, Future, Stream};

extern crate serde_derive;
extern crate serde_json;
use serde_json::Value;

#[derive(Deserialize, Debug)]
struct BlockInfo {
    pub generationSignature: String,
}

pub fn get_generation_signature(_height: u64) -> BlockInfo {
    rt::run(rt::lazy(|| {
        let https = hyper_tls::HttpsConnector::new(4).unwrap();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        client
            .get(
                "https://wallet.burst.cryptoguru.org/burst?requestType=getBlock&height=520000"
                    .parse()
                    .unwrap(),
            )
            .and_then(|res| res.into_body().concat2())
            .and_then(|body| {
                let json = serde_json::from_slice::<BlockInfo>(&body).unwrap();
                println!("{}", json.generationSignature);
                Ok(())

            })
            .map_err(|e| panic!("example expects stdout to work: {}", e))
    }));
    
}
