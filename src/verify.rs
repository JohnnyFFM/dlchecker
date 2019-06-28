use crate::poc_hashing::{
    calculate_new_gensig, calculate_scoop, decode_gensig, find_best_deadline_rust, noncegen_rust,
};
use pbr::ProgressBar;
use rayon::prelude::*;

#[serde(default)]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Block {
    height: u64,
    time: u64,
    #[serde(rename = "baseTarget")]
    base_target: u64,
    #[serde(rename = "plotterId")]
    plotter_id: u64,
    nonce: u64,
    #[serde(rename = "generationSignature")]
    generation_signature: String,
    deadline: u64,
}

#[serde(default)]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Info {
    scoop: u64,
    deadline: u64,
    elapsed: u64,
}

pub fn verify(matches: &clap::ArgMatches) {
    // load csv
    let filename = value_t!(matches, "csv_file", String).unwrap_or_else(|e| e.exit());

    let offset = filename.find(".csv").unwrap_or(filename.len());

    // Replace the range up until the β from the string
    let mut filename2 = filename.clone();
    filename2.replace_range(offset.., "_scoop_dl.csv");

    println!("Input      : {}", &filename);
    println!("Output     : {}", &filename2);
    let mut rdr = csv::Reader::from_path(&filename).unwrap();
    let mut wtr = csv::Writer::from_path(&filename2).unwrap();

    let blocks = rdr
        .records()
        .collect::<Result<Vec<csv::StringRecord>, csv::Error>>()
        .expect("error collecting blocks");
    println!("#Blocks    : {}", blocks.len());

    let mut blocks_decoded = Vec::new();
    for i in 0..blocks.len() {
        let block: Block = blocks[i]
            .deserialize(None)
            .expect("error deserializing block");
        blocks_decoded.push(block);
    }

    let mut pass = true;
    let mut pass2 = true;
    let mut pb = ProgressBar::new(blocks_decoded.len() as u64 - 84002);
    pb.format("│██░│");
    pb.set_width(Some(80));
    pb.message("Verifying : ");

    //for i in 1..blocks_decoded.len() {
    (84002..blocks_decoded.len()).into_par_iter().for_each(|i| {
        //skip premine
        //println!("{:?}", blocks_decoded[i]);
        // verify header
        let last_gensig = &blocks_decoded[i - 1].generation_signature;
        let last_generator = blocks_decoded[i - 1].plotter_id;
        let last_gensig = decode_gensig(last_gensig);
        let gensig = calculate_new_gensig(last_generator, &last_gensig);
        //println!("Gensig calced from prev block : {}",  &hex::encode(&gensig));
        //println!("Gensig stored in chain        : {}", blocks_decoded[i].generation_signature);
        if hex::encode(&gensig) != blocks_decoded[i].generation_signature {
            if pass {
                pass = false;
                println!(
                    "Gensig Validation Error detected, StartHeight = {}",
                    blocks_decoded[i].height
                );
            }
        } else {
            if !pass {
                println!(
                    "Gensig Validation Error detected, EndHeight = {}",
                    blocks_decoded[i].height
                );
                pass = true;
            }
        }

        // verify PoC
        let scoop = calculate_scoop(blocks_decoded[i].height, &gensig);

        //plot
        let mut cache = vec![0u8; 262144];
        noncegen_rust(
            &mut cache[..],
            blocks_decoded[i].plotter_id,
            blocks_decoded[i].nonce,
            1,
        );

        let mut poc2scoopdata = vec![0u8; 64];
        let address = 64 * scoop as usize;
        let mirrorscoop = 4095 - scoop as usize;
        let mirroraddress = 64 * mirrorscoop as usize;
        poc2scoopdata[0..32].clone_from_slice(&cache[address..address + 32]);
        poc2scoopdata[32..64].clone_from_slice(&cache[mirroraddress + 32..mirroraddress + 64]);

        let (deadline, _) = find_best_deadline_rust(&poc2scoopdata[..], 1, &gensig);
        let deadline_adj = deadline / blocks_decoded[i - 1].base_target;
        if deadline_adj != blocks_decoded[i].deadline {
            if pass2 {
                pass2 = false;
                println!(
                    "Deadline Validation Error detected, StartHeight = {}",
                    blocks_decoded[i].height
                );
            }
        } else {
            if !pass2 {
                println!(
                    "Deadline Validation Error detected, EndHeight = {}",
                    blocks_decoded[i].height
                );
                pass2 = true;
            }
        }
        pb.inc();

        // save scoop / deadline file
        let info = Info {
            scoop: scoop as u64,
            deadline: deadline_adj,
            elapsed: blocks_decoded[i - 1].time - blocks_decoded[i].time,
        };

        // export to csv
        wtr.serialize(info).expect("error deserialising block info");
    });
    if !pass {
        println!(
            "Gensig Validation Error detected, EndHeight = {}",
            blocks_decoded[blocks_decoded.len() - 1].height
        );
    }
    if !pass2 {
        println!(
            "Deadline Validation Error detected, EndHeight = {}",
            blocks_decoded[blocks_decoded.len() - 1].height
        );
    }

    println!("done.");
}
