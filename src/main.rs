use std::collections::BTreeMap;

use anyhow::Result;
use bech32::{FromBase32, ToBase32, Variant};
use clap::error::ErrorKind;
use clap::{command, ArgGroup, CommandFactory, Parser};
use rand::RngCore;
use rand::rngs::OsRng;
use regex::Regex;
use secp256k1::{SecretKey, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::DomainValidationError::{InvalidDomain, NoDomains};

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
        .args(&["kind", "to_hex", "nip5", "gen_keys"]),
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

    #[arg(
        long,
        help = "boolean flag indicating to generate new keys and print them in hex and bech32 format",
    )]
    gen_keys: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.to_hex {        
        // convert bech32 npub/nsec/note to hex (accepts list of bech32's)
        for s in &args.keys {
            let (_, data, _) = bech32::decode(s)?;
            println!("{}", hex::encode(Vec::<u8>::from_base32(&data)?));
        }
        Ok(())
    } else if args.kind.is_some() {
        // convert hex to bech32 npub/nsec/note (accepts list of hex)
        let hrp = args.kind.unwrap();
        for key in &args.keys {
            let encoded = bech32_encode(hrp, key);
            println!("{encoded}");
        }
        Ok(())
    } else if args.nip5.is_some() {
        for nip5_domain in validate_domains(&args.nip5)? {
            let nip5_ids: Nip5Id = fetch_nostr_json(nip5_domain.to_string()).await?;
            for (key, value) in &nip5_ids.names {
                println!("{key},{}", bech32_encode(Prefix::Npub, value));
            }
            // optional flag based stats
            if args.nip_stats {
                println!("domain={}|count={}", nip5_domain, nip5_ids.names.len());
            }
        }
        Ok(())
    } else if args.gen_keys {
        let secp = Secp256k1::new();
    
        let mut bytes = [0u8; 32];
        let mut rng = OsRng;
    
        rng.try_fill_bytes(&mut bytes).unwrap();
    
        let secret_key = SecretKey::from_slice(&bytes).expect("Error generating secret key");
    
        let (pubkey, _) = PublicKey::from_secret_key(&secp, &secret_key).x_only_public_key();
        let hex_pubkey = hex::encode(pubkey.serialize());
        let hex_secret_key = hex::encode(&secret_key[..]);

        let bech32_pubkey = bech32_encode(Prefix::Npub, &hex_pubkey);
        let bech32_secret_key = bech32_encode(Prefix::Nsec, &hex_secret_key);
    
        println!("Hex Public Key: {}", hex_pubkey);
        println!("Hex Secret Key: {}", hex_secret_key);
        println!("Bech32 Public Key: {}", bech32_pubkey);
        println!("Bech32 Secret Key: {}", bech32_secret_key);
        Ok(())
    }
    else {
        Err(
            Args::command() //  in the event of an args bug, this will print an error and exit
                .error(
                    ErrorKind::MissingRequiredArgument,
                    "BUG!!! Should not get here, check input args/groups.",
                )
                .into(),
        )
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

#[derive(Error, Debug)]
enum DomainValidationError {
    #[error("No domains provided")]
    NoDomains,
    #[error("Invalid domain name: `{0}`")]
    InvalidDomain(String),
}

/// Validates all domain inputs from "--nip5" are valid
fn validate_domains(domains: &Option<Vec<String>>) -> Result<Vec<String>, DomainValidationError> {
    domains
        .as_ref()
        .ok_or(NoDomains)?
        .iter()
        .map(|domain| {
            if is_valid_domain_name(domain) {
                Ok(domain.clone())
            } else {
                Err(InvalidDomain(domain.clone()))
            }
        })
        .collect()
}

/// Ensures string is valid domain name format
fn is_valid_domain_name(domain: &str) -> bool {
    let domain_regex = Regex::new(r"(?i)^([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}$").unwrap();
    domain_regex.is_match(domain)
}