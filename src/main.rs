use crate::DomainValidationError::{InvalidDomain, NoDomains};
use crate::KeyValidationError::{InvalidKeyDecode, InvalidKeyEncode};
use anyhow::Result;
use bech32::{FromBase32, ToBase32, Variant};
use clap::error::ErrorKind;
use clap::{command, ArgGroup, CommandFactory, Parser};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use tokio::task::JoinHandle;

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
            let encoded = bech32_encode(hrp, key).unwrap();
            println!("{encoded}");
        }
        Ok(())
    } else if args.nip5.is_some() {
        let valid_domains = validate_domains(&args.nip5)?;
        let mut handles: Vec<JoinHandle<_>> = Vec::with_capacity(valid_domains.len());
        for nip5_domain in valid_domains {
            let handle = tokio::spawn(async move {
                let nip5_ids = fetch_nostr_json(nip5_domain.to_string()).await.unwrap();
                let mut nip5results = nip5_ids
                    .names
                    .iter()
                    .map(|val| {
                        format!(
                            "{},{}",
                            val.0,
                            bech32_encode(Prefix::Npub, val.1)
                                .unwrap_or_else(|err| err.to_string())
                        )
                    })
                    .collect::<Vec<String>>();
                if args.nip_stats {
                    //  add stats if '-s' flag enabled
                    nip5results.push(format!(
                        "domain={}|count={}",
                        nip5_domain,
                        nip5_ids.names.len()
                    ));
                }
                nip5results
            });
            handles.push(handle);
        }
        for task in handles {
            task.await
                .map_err(|err| println!("{}", err))
                .unwrap()
                .iter()
                .for_each(|text| println!("{}", text));
        }
        Ok(())
    } else {
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

#[derive(Error, Debug)]
enum KeyValidationError {
    #[error("KeyValidationError::InvalidKeyDecode:could not decode provided key={0}")]
    InvalidKeyDecode(String),
    #[error("KeyValidationError::InvalidKeyEncode:could not encode provided key={0}")]
    InvalidKeyEncode(String),
}
/// Converts a hex encoded string to bech32 format for given a Prefix (hrp)
fn bech32_encode(hrp: Prefix, hex_key: &String) -> Result<String, KeyValidationError> {
    bech32::encode(
        &hrp.to_string(),
        hex::decode(hex_key)
            .map_err(|_| InvalidKeyDecode(hex_key.to_string()))?
            .to_base32(),
        Variant::Bech32,
    )
    .map_err(|_| InvalidKeyEncode(hex_key.to_string()))
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

#[derive(Error, Debug)]
enum DomainValidationError {
    #[error("No domains provided")]
    NoDomains,
    #[error("Invalid domain name: `{0}`")]
    InvalidDomain(String),
}
