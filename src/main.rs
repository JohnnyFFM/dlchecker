#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;

mod dlcheck;
mod network;
mod poc_hashing;
mod shabal256;
mod extract;

use crate::dlcheck::dlcheck;
use crate::extract::{WalletType, extract};
use clap::AppSettings::{ArgRequiredElseHelp, DeriveDisplayOrder, SubcommandRequiredElseHelp};
use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("Deadliner Checker")
        .version("0.1")
        .author("JohnnyFFM")
        .about("a deadline checker written in rust")
        .setting(SubcommandRequiredElseHelp)
        .setting(ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("check")
                .about("Calculates a deadline for a specific height, id and nonce. Optionally check against a plot file.")
                .setting(ArgRequiredElseHelp)
                .setting(DeriveDisplayOrder)
            .arg(
                Arg::with_name("height")
                    .short("h")
                    .long("height")
                    .value_name("height")
                    .help("block height")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("id")
                    .short("i")
                    .long("numeric_id")
                    .value_name("id")
                    .help("numeric id")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("nonce")
                    .short("n")
                    .long("nonce")
                    .value_name("nonce")
                    .help("nonce")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("plotfile")
                    .short("f")
                    .long("file")
                    .value_name("plotfile")
                    .help("plot file (optional)")
                    .required(false)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("walleturl")
                    .short("w")
                    .long("wallet")
                    .value_name("wallet url")
                    .help("wallet url including protocol and path (eg. https://burst.megash.it/burst) (optional)")
                    .required(false)
                    .takes_value(true),
            )
        ).subcommand(
            SubCommand::with_name("extract")
                .about("Extract a PoC blockchain's verification headers from a (online)wallet or database into a .csv file")
                .setting(ArgRequiredElseHelp)
                .setting(DeriveDisplayOrder)
            .arg(
                Arg::with_name("wallet")
                    .short("w")
                    .long("wallet")
                    .value_name("wallet url/db")
                    .help("wallet url including protocol and path (eg. http://localhost:8125/burst) or database location")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("wallet_type")
                    .short("t")
                    .long("type")
                    .value_name("wallet_type")
                    .help("wallet type")
                    .possible_values(&WalletType::variants())
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("cookie")
                    .short("c")
                    .long("cookie")
                    .value_name("authentication cookie")
                    .help("path to .cookie (for BhdRpc)")
                    .takes_value(true)
                    .conflicts_with_all(&["username", "password"])
                    .required_if("server_type", &WalletType::BhdRpc.to_string()),
            )
            .arg(
                Arg::with_name("username")
                    .short("u")
                    .long("user")
                    .value_name("username")
                    .help("username (for BurstDB)")
                    .takes_value(true)
                    .required_if("server_type", &WalletType::BurstDB.to_string()),
            )
            .arg(
                Arg::with_name("pass")
                    .short("p")
                    .long("password")
                    .value_name("password")
                    .help("password (for BurstDB)")
                    .takes_value(true)
                    .required_if("server_type", &WalletType::BurstDB.to_string()),
            )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("check") {
        dlcheck(matches);
    }
    if let Some(matches) = matches.subcommand_matches("extract") {
        extract(matches);
    }
}
