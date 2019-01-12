#[macro_use]
extern crate serde_derive;
mod network;
use crate::network::get_generation_signature;

fn main() {
    println!(
        "Block Height        : {}",
        520000
    );
    println!(
        "Generation Signature: {}",
        get_generation_signature(520000).generation_signature
    );
}
