use crate::network::get_blockinfo;
use crate::poc_hashing::{calculate_scoop, decode_gensig, find_best_deadline_rust, noncegen_rust};
use hex;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::u64;

pub fn dlcheck(matches: &clap::ArgMatches) {
    let numeric_id = value_t!(matches, "id", u64).unwrap_or_else(|e| e.exit());
    let height = value_t!(matches, "height", u64).unwrap_or_else(|e| e.exit());
    let nonce = value_t!(matches, "nonce", u64).unwrap_or_else(|e| e.exit());
    let plotfile = value_t!(matches, "plotfile", String).unwrap_or_else(|_| "".to_owned());

    let walleturl = value_t!(matches, "walleturl", String)
        .unwrap_or_else(|_| "https://wallet.burst.cryptoguru.org".to_string());

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
    let gensig = decode_gensig(&blockinfo.generation_signature);
    let scoop = calculate_scoop(height, &gensig);
    println!("Scoop                : {}", scoop);

    //plot
    let mut cache = vec![0u8; 262144];
    noncegen_rust(&mut cache[..], numeric_id, nonce, 1);
    let address = 64 * scoop as usize;

    println!(
        "Hash 1:              : {:x?}",
        &hex::encode(&cache[address..address + 32])
    );

    println!(
        "Hash 2: (PoC1)       : {:x?}",
        &hex::encode(&cache[address + 32..address + 64])
    );

    let mirrorscoop = 4095 - scoop as usize;
    let mirroraddress = 64 * mirrorscoop as usize;

    println!(
        "Hash 2: (PoC2)       : {:x?}",
        &hex::encode(&cache[mirroraddress + 32..mirroraddress + 64])
    );

    let mut scoopdata = vec![0u8; 64];
    scoopdata.clone_from_slice(&cache[address..address + 64]);

    let mut mirrorscoopdata = vec![0u8; 64];
    mirrorscoopdata[0..32].clone_from_slice(&cache[address..address + 32]);
    mirrorscoopdata[32..64].clone_from_slice(&cache[mirroraddress + 32..mirroraddress + 64]);

    let (deadline, _) = find_best_deadline_rust(&scoopdata[..], 1, &gensig);

    println!("Deadline PoC1 (raw)  : {}", deadline);
    println!(
        "Deadline PoC1 (adj)  : {}",
        deadline / blockinfo_prev.base_target
    );

    let (deadline, _) = find_best_deadline_rust(&mirrorscoopdata[..], 1, &gensig);

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

        let (deadline, _) = find_best_deadline_rust(&mirrorscoopdata[..], 1, &gensig);

        println!("Deadline 1 (raw)     : {}", deadline);
        println!(
            "Deadline 1 (adj)     : {}",
            deadline / blockinfo_prev.base_target
        );

        let (deadline, _) = find_best_deadline_rust(&scoopdata[..], 1, &gensig);

        println!("Deadline 2 (raw)     : {}", deadline);
        println!(
            "Deadline 2 (adj)     : {}",
            deadline / blockinfo_prev.base_target
        );
    }
}
