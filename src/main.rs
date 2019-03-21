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
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
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
        .arg(
            Arg::with_name("plotfile")
                .short("f")
                .long("file")
                .value_name("plotfile")
                .help("plot file")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("walleturl")
                .short("w")
                .long("wallet")
                .value_name("wallet url")
                .help("wallet url including protocol (eg. https://burst.megash.it/)")
                .required(false)
                .takes_value(true),
        )
        .get_matches();
    let numeric_id = value_t!(matches, "id", u64).unwrap_or_else(|e| e.exit());
    let height = value_t!(matches, "height", u64).unwrap_or_else(|e| e.exit());
    let nonce = value_t!(matches, "nonce", u64).unwrap_or_else(|e| e.exit());
    let plotfile = value_t!(matches, "plotfile", String).unwrap_or_else(|_| "".to_owned());

    let walleturl = value_t!(matches, "walleturl", String).unwrap_or_else(|_| {
        "https://wallet.burst.cryptoguru.org".to_string()
    });

    println!("Block Height         : {}", height);
    println!("Numeric ID           : {}", numeric_id);
    println!("Nonce                : {}", nonce);
    println!("Wallet Url           : {}", walleturl);
    let blockinfo = get_blockinfo(height, walleturl.clone());
    let blockinfo_prev = get_blockinfo(height - 1, walleturl.clone());

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

    let mirrorscoop = 4095 - scoop as usize;
    let mirroraddress = 64 * mirrorscoop as usize;

    println!(
        "Hash 2: (PoC1)       : {:x?}",
        &hex::encode(&cache[mirroraddress + 32..mirroraddress + 64])
    );

    let mut deadline: u64 = u64::MAX;
    let mut offset: u64 = 0;
    let mut scoopdata = vec![0u8; 64];
    scoopdata.clone_from_slice(&cache[address..address + 64]);

    let mut mirrorscoopdata = vec![0u8; 64];
    mirrorscoopdata[0..32].clone_from_slice(&cache[address..address + 32]);
    mirrorscoopdata[32..64].clone_from_slice(&cache[mirroraddress + 32..mirroraddress + 64]);

    unsafe {
        find_best_deadline_sph(
            mirrorscoopdata.as_ptr() as *mut c_void,
            1,
            gensig.as_ptr() as *const c_void,
            &mut deadline,
            &mut offset,
        );
    }
    println!("Deadline PoC1 (raw)  : {}", deadline);
    println!(
        "Deadline PoC1 (adj)  : {}",
        deadline / blockinfo_prev.base_target
    );

    deadline = u64::MAX;
    offset = 0;

    unsafe {
        find_best_deadline_sph(
            scoopdata.as_ptr() as *mut c_void,
            1,
            gensig.as_ptr() as *const c_void,
            &mut deadline,
            &mut offset,
        );
    }
    println!("Deadline PoC2 (raw)  : {}", deadline);
    println!(
        "Deadline PoC2 (adj)  : {}",
        deadline / blockinfo_prev.base_target
    );

    if plotfile != "" {
        println!("Plotfile Name        : {}", plotfile);

        let plotfile = Path::new(&plotfile);
        if !plotfile.is_file() {
            return;
        }

        let name = plotfile.file_name().unwrap().to_str().unwrap();
        let parts: Vec<&str> = name.split('_').collect();
        match parts.len() {
            3 => {
                println!("Plotfile Format      : PoC2");
            }
            4 => {
                println!("Plotfile Format      : PoC1");
                if parts[2].parse::<u64>().unwrap() != parts[3].parse::<u64>().unwrap() {
                    println!("unoptimized files are not supported.");
                    return;
                }
            }
            _ => {
                return;
            }
        }

        let account_id = parts[0].parse::<u64>().unwrap();
        if numeric_id != account_id {
            println!("numeric id's not matching!");
            return;
        }
        let start_nonce = parts[1].parse::<u64>().unwrap();
        let nonces = parts[2].parse::<u64>().unwrap();

        let size = fs::metadata(plotfile).unwrap().len();
        let exp_size = nonces * 4096 * 64;
        if size != exp_size as u64 {
            println!("expected plot size {} but got {}", exp_size, size);
            return;
        }
        let mut file = OpenOptions::new().read(true).open(plotfile).unwrap();

        if !(nonce >= start_nonce && nonce < start_nonce + nonces) {
            println!("File doent contain requested nonce");
            return;
        }

        let address = scoop as u64 * 64 * 4096 + (nonce - start_nonce) * 64;

        file.seek(SeekFrom::Start(address)).unwrap();
        file.read_exact(&mut scoopdata[0..64]).unwrap();

        println!(
            "Hash 1:              : {:x?}",
            &hex::encode(&scoopdata[0..32])
        );

        println!(
            "Hash 2:              : {:x?}",
            &hex::encode(&scoopdata[32..64])
        );

        let address = mirrorscoop as u64 * 64 * 4096 + (nonce - start_nonce) * 64 + 32;
        mirrorscoopdata[0..32].clone_from_slice(&scoopdata[0..32]);

        file.seek(SeekFrom::Start(address)).unwrap();
        file.read_exact(&mut mirrorscoopdata[32..64]).unwrap();
        println!(
            "Hash 2: (Mirror)     : {:x?}",
            &hex::encode(&mirrorscoopdata[32..64])
        );

        deadline = u64::MAX;
        offset = 0;

        unsafe {
            find_best_deadline_sph(
                scoopdata.as_ptr() as *mut c_void,
                1,
                gensig.as_ptr() as *const c_void,
                &mut deadline,
                &mut offset,
            );
        }
        println!("Deadline 1 (raw)     : {}", deadline);
        println!(
            "Deadline 1 (adj)     : {}",
            deadline / blockinfo_prev.base_target
        );

        deadline = u64::MAX;
        offset = 0;

        unsafe {
            find_best_deadline_sph(
                mirrorscoopdata.as_ptr() as *mut c_void,
                1,
                gensig.as_ptr() as *const c_void,
                &mut deadline,
                &mut offset,
            );
        }
        println!("Deadline 2 (raw)     : {}", deadline);
        println!(
            "Deadline 2 (adj)     : {}",
            deadline / blockinfo_prev.base_target
        );
    }
}
