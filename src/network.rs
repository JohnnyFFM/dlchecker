extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;

use hyper::rt::{Future, Stream};
use serde::de;
use std::fmt;

extern crate serde_derive;
extern crate serde_json;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    pub generation_signature: String,
    #[serde(deserialize_with = "from_str_or_int")]
    pub base_target: u64,
}

pub fn get_blockinfo(height: u64, walleturl: String) -> BlockInfo {
    let https = hyper_tls::HttpsConnector::new(4).unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let url = walleturl + &"/burst?requestType=getBlock&height=".to_owned()
        + &height.to_string();
    let fut = client
        .get(url.parse().unwrap())
        .and_then(|res| res.into_body().concat2())
        .map_err(|e| panic!("example expects stdout to work: {}", e))
        .and_then(|body| Ok(serde_json::from_slice::<BlockInfo>(&body).unwrap()));
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let result = core.run(fut).unwrap();
    result
}

// MOTHERFUCKING pool
fn from_str_or_int<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> de::Visitor<'de> for StringOrIntVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or int")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<u64>().map_err(de::Error::custom)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v)
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}
