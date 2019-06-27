use crate::poc_hashing::{calculate_scoop, decode_gensig, find_best_deadline_rust, noncegen_rust, calculate_new_gensig};

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
    let blocks = rdr
        .records()
        .collect::<Result<Vec<csv::StringRecord>, csv::Error>>().expect("error collecting blocks");
    println!("#Blocks    : {}", blocks.len());

    let mut blocks_decoded = Vec::new();
    for i in 0..blocks.len(){
        let block: Block = blocks[i].deserialize(None).expect("error deserializing block");
        blocks_decoded.push(block);
    }

    let mut pass = true;
    for i in 1..blocks_decoded.len(){
        //println!("{:?}", blocks_decoded[i]);
        // verify header
        let last_gensig = &blocks_decoded[i-1].generation_signature;
        let last_generator = blocks_decoded[i-1].plotter_id;
        let last_gensig = decode_gensig(last_gensig);
        let gensig = calculate_new_gensig(last_generator, &last_gensig);
        //println!("New Gensig                : {}",  &hex::encode(&gensig));
        //println!("New Gensig                : {}", blocks_decoded[i].generation_signature);
        if hex::encode(&gensig) != blocks_decoded[i].generation_signature {
            if pass{
                pass = false;
                print!("Validation Error detected, StartHeight = {}",blocks_decoded[i].height );
            }
        } else {
            if !pass{
                println!(" EndHeight = {}",blocks_decoded[i].height );
                pass =true;
            }
        }
        // verify PoC
        // save scoop / deadline file
    }
    if !pass {
        println!(" EndHeight = {}",blocks_decoded[blocks_decoded.len()-1].height );
    }
    println!("done.");
}