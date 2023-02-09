use bech32::{FromBase32, ToBase32, Variant};
use clap::error::ErrorKind;
use clap::{command, ArgGroup, CommandFactory, Parser};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
enum Prefix {
    Npub,
    Nsec,
    Note,
}

// Display 'trait' needed for enum "to_string()"
impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Prefix::Npub => write!(f, "npub"),
            Prefix::Nsec => write!(f, "nsec"),
            Prefix::Note => write!(f, "note"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Nip5Id {
    names: BTreeMap<String, String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    //  input error handling: only exactly ONE of 'n' args can be used
    ArgGroup::new("convert_method")
        .required(true)
        .args(&["kind", "to_hex", "nip5"]),
))]
struct Args {
    #[arg(
        short,
        long,
        help = "the kind of entity (npub/nsec/note) being converted from hex to bech32-formatted string",
        requires = "keys"
    )]
    kind: Option<Prefix>,

    #[arg(
        long,
        help = "boolean flag indicating to convert keys from bech32 to hex",
        requires = "keys"
    )]
    to_hex: bool,

    #[arg(
        use_value_delimiter = true,
        value_delimiter = ',',
        help = "the keys or note ids that you want to convert (either hex or bech32)"
    )]
    keys: Vec<String>,

    #[arg(
        long,
        use_value_delimiter = true,
        value_delimiter = ',',
        help = "nip5 dns-based identifiers (domain names)"
    )]
    nip5: Option<Vec<String>>,

    #[arg(
        short = 's',
        long,
        requires = "nip5",
        help = "nip5 identifiers stats logging"
    )]
    nip_stats: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.to_hex {
        // convert bech32 npub/nsec/note to hex (accepts list of bech32's)
        for s in &args.keys {
            let (_, data, _) = bech32::decode(s).expect("could not decode data");
            println!("{}", hex::encode(Vec::<u8>::from_base32(&data).unwrap()));
        }
    } else if args.kind.is_some() {
        // convert hex to bech32 npub/nsec/note (accepts list of hex)
        let hrp = args.kind.unwrap();
        for key in &args.keys {
            let encoded = bech32_encode(hrp, key);
            println!("{encoded}");
        }
    } else if args.nip5.is_some() {
        validate_domains(&args.nip5);
        for nip5_domain in &args.nip5.unwrap() {
            let _result = fetch_nostr_json(nip5_domain.to_string()).await;
            let nip5_ids: Nip5Id = _result.expect(
                &("Error while fetching nostr.json from nip5-domain=".to_owned() + nip5_domain),
            );
            for (key, value) in &nip5_ids.names {
                println!("{key},{}", bech32_encode(Prefix::Npub, value));
            }
            // optional flag based stats
            if args.nip_stats {
                println!("domain={}|count={}", nip5_domain, nip5_ids.names.len());
            }
        }
    } else {
        Args::command() //  in the event of an args bug, this will print an error and exit
            .error(
                ErrorKind::MissingRequiredArgument,
                "BUG!!! Should not get here, check input args/groups.",
            )
            .exit();
    }
}

/// Converts a hex encoded string to bech32 format for given a Prefix (hrp)
fn bech32_encode(hrp: Prefix, hex_key: &String) -> String {
    bech32::encode(
        &hrp.to_string(),
        hex::decode(hex_key)
            .expect(&("could not decode provided key/note id=".to_owned() + hex_key))
            .to_base32(),
        Variant::Bech32,
    )
    .expect("Could not bech32-encode data")
}

/// Makes GET request to NIP-05 domain to get nostr.json, and converts public keys from hex to bech32 format
async fn fetch_nostr_json(nip5_domain: String) -> Result<Nip5Id, reqwest::Error> {
    let nip5_url = format!("{}{}{}", "https://", nip5_domain, "/.well-known/nostr.json");
    let json: Nip5Id = reqwest::Client::new()
        .get(nip5_url)
        .send()
        .await?
        .json()
        .await?;
    Ok(json)
}

/// Validates all domain inputs from "--nip5" are valid
fn validate_domains(domains: &Option<Vec<String>>) {
    match domains {
        Some(domain_list) => {
            for domain in domain_list {
                if !is_valid_domain_name(domain) {
                    Args::command()
                        .error(
                            ErrorKind::InvalidValue,
                            format!("NIP5 domains - invalid domain name detected: {domain}"),
                        )
                        .exit();
                }
            }
        }
        None => println!("No domains provided"),
    }
}

/// Ensures string is valid domain name format
fn is_valid_domain_name(domain: &str) -> bool {
    let domain_regex = Regex::new(r"(?i)^([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}$").unwrap();
    domain_regex.is_match(domain)
}
