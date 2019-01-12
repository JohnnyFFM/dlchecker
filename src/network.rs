extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use hyper::rt::{Future, Stream};

extern crate serde_derive;
extern crate serde_json;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    pub generation_signature: String,
}

pub fn get_generation_signature(_height: u64) -> BlockInfo {
    let https = hyper_tls::HttpsConnector::new(4).unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let fut = client
        .get(
            "https://wallet.burst.cryptoguru.org/burst?requestType=getBlock&height=520000"
                .parse()
                .unwrap(),
        )
        .and_then(|res| res.into_body().concat2())
        .map_err(|e| panic!("example expects stdout to work: {}", e))
        .and_then(|body| Ok(serde_json::from_slice::<BlockInfo>(&body).unwrap()));
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let result = core.run(fut).unwrap();
    result
}
