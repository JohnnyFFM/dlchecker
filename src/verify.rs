use crate::poc_hashing::{
    calculate_new_gensig, calculate_scoop, decode_gensig, find_best_deadline_rust, noncegen_rust,
};
use pbr::ProgressBar;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

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
    height: u64,
    scoop: u64,
    deadline: u64,
    elapsed: u64,
    gensig_ok: bool,
    poc_ok: bool,
}

pub fn verify(matches: &clap::ArgMatches) {
    // load csv
    let filename = value_t!(matches, "csv_file", String).unwrap_or_else(|e| e.exit());

    let offset = filename.find(".csv").unwrap_or(filename.len());

    // Replace the range up until the β from the string
    let mut filename2 = filename.clone();
    filename2.replace_range(offset.., "_analysis.csv");

    println!("Input      : {}", &filename);
    println!("Output     : {}", &filename2);
    let mut rdr = csv::Reader::from_path(&filename).unwrap();
    let wtr = csv::Writer::from_path(&filename2).unwrap();
    let wtr = Arc::new(Mutex::new(wtr));

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


    let mut pb = ProgressBar::new(blocks_decoded.len() as u64 - 84002);
    pb.format("│██░│");
    pb.set_width(Some(80));
    pb.message("Verifying : ");
    let pb = Arc::new(Mutex::new(pb));

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
        let gensig_ok = hex::encode(&gensig) == blocks_decoded[i].generation_signature;
        if gensig_ok {
            println!(
                "Gensig Validation Error detected, Height = {}",
                blocks_decoded[i].height
            );
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
        let poc_ok = deadline_adj == blocks_decoded[i].deadline;
        if  poc_ok{
            println!(
                "Deadline Validation Error detected, Height = {}",
                blocks_decoded[i].height
            );
        }
        let mut pb = pb.lock().unwrap();
        pb.inc();

        // save scoop / deadline file
        let info = Info {
            height: blocks_decoded[i].height,
            scoop: scoop as u64,
            deadline: deadline_adj,
            elapsed: blocks_decoded[i - 1].time - blocks_decoded[i].time,
            gensig_ok,
            poc_ok,
        };

        // export to csv
        let mut wtr = wtr.lock().unwrap();
        wtr.serialize(info).expect("error deserialising block info");
    });

    println!("done.");
}
