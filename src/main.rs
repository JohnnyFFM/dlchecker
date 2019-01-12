#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

mod burstmath;
mod network;
mod shabals;

use crate::network::get_blockinfo;
use clap::AppSettings::ArgRequiredElseHelp;
use clap::{App, Arg};
use hex;
use libc::{c_void, size_t, uint64_t};
use std::u64;

extern "C" {
    pub fn noncegen(
        cache: *mut c_void,
        cache_size: size_t,
        chunk_offset: size_t,
        numeric_ID: uint64_t,
        local_startnonce: uint64_t,
        local_nonces: uint64_t,
    );
    pub fn find_best_deadline_sph(
        scoops: *mut c_void,
        nonce_count: uint64_t,
        gensig: *const c_void,
        best_deadline: *mut uint64_t,
        best_offset: *mut uint64_t,
    ) -> ();
}

fn main() {
    let matches = App::new("Deadliner Checker")
        .version("0.1")
        .author("JohnnyFFM")
        .about("a deadline checker written in rust")
        .setting(ArgRequiredElseHelp)
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .value_name("height")
                .help("block height")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("id")
                .short("i")
                .long("numeric_id")
                .value_name("id")
                .help("numeric id")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("nonce")
                .short("n")
                .long("nonce")
                .value_name("nonce")
                .help("nonce")
                .takes_value(true),
        )
        .get_matches();
    let numeric_id = value_t!(matches, "id", u64).unwrap_or_else(|e| e.exit());
    let height = value_t!(matches, "height", u64).unwrap_or_else(|e| e.exit());
    let nonce = value_t!(matches, "nonce", u64).unwrap_or_else(|e| e.exit());

    println!("Block Height         : {}", height);
    println!("Numeric ID           : {}", numeric_id);
    println!("Nonce                : {}", nonce);
    let blockinfo = get_blockinfo(height);
    let blockinfo_prev = get_blockinfo(height - 1);

    println!("Generation Signature : {}", blockinfo.generation_signature);
    println!("Base Target          : {}", blockinfo_prev.base_target);
    println!(
        "Net Difficulty       : {}",
        4398046511104 / 240 / blockinfo_prev.base_target
    );
    let gensig = burstmath::decode_gensig(&blockinfo.generation_signature);
    let scoop = burstmath::calculate_scoop(height, &gensig);
    println!("Scoop                : {}", scoop);

    //plot
    let cache = vec![0u8; 262144];

    unsafe {
        noncegen(cache.as_ptr() as *mut c_void, 1, 0, numeric_id, nonce, 1);
    }
    let address = 64 * scoop as usize;

    println!(
        "Hash 1:              : {:x?}",
        &hex::encode(&cache[address..address + 32])
    );
    
    println!(
        "Hash 2: (PoC2)       : {:x?}",
        &hex::encode(&cache[address + 32..address + 64])
    );

    let mirrorscoop = 4095-scoop as usize;
    let mirroraddress = 64 * mirrorscoop as usize;

    println!(
        "Hash 2: (PoC1)       : {:x?}",
        &hex::encode(&cache[mirroraddress + 32..mirroraddress + 64])
    ); 

    let mut deadline: u64 = u64::MAX;
    let mut offset: u64 = 0;
    let mut scoop = vec![0u8; 64];
    scoop.clone_from_slice(&cache[address..address + 64]);

    let mut mirrorscoop = vec![0u8; 64];
    mirrorscoop[0..32].clone_from_slice(&cache[address..address + 32]);
    mirrorscoop[32..64].clone_from_slice(&cache[mirroraddress + 32..mirroraddress + 64]);

    unsafe {
        find_best_deadline_sph(
            mirrorscoop.as_ptr() as *mut c_void,
            1,
            gensig.as_ptr() as *const c_void,
            &mut deadline,
            &mut offset,
        );
    }
    println!("Deadline PoC1 (raw)  : {}", deadline);
    println!("Deadline PoC1 (adj)  : {}", deadline/blockinfo_prev.base_target);
    
    deadline = u64::MAX;
    offset = 0;

    unsafe {
        find_best_deadline_sph(
            scoop.as_ptr() as *mut c_void,
            1,
            gensig.as_ptr() as *const c_void,
            &mut deadline,
            &mut offset,
        );
    }
    println!("Deadline PoC2 (raw)  : {}", deadline);
    println!("Deadline PoC2 (adj)  : {}", deadline/blockinfo_prev.base_target);


}
