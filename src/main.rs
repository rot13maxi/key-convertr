use bech32::FromBase32;
use bech32::{ToBase32, Variant};
use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
enum Prefix {
    Npub,
    Nsec,
    Note,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "the kind of entity you are converting")]
    kind: Option<Prefix>,

    #[arg(long, help = "you want to convert from bech32 to hex")]
    to_hex: bool,

    #[arg(help = "the key or note id that you want to convert")]
    key: String,
}

fn main() {
    let args = Args::parse();
    if !args.to_hex && args.kind.is_none() {
        println!("You need to either specify a `kind` of key you are converting (to go from hex to bech32) or specify `to-hex` (to go to hex)");
        return;
    }

    if args.to_hex && args.kind.is_some() {
        println!("provide either `--to-hex` OR a `--kind` to convert to. one or the other.");
        return;
    }

    if args.to_hex {
        // convert npub to hex (accepts comma separated list of npubs)
        let split = args.key.as_str().trim_end().split(',');
        for s in split {
            let (_, data, _) = bech32::decode(s).expect("could not decode data");
            println!("{}", hex::encode(Vec::<u8>::from_base32(&data).unwrap()));
        }
    } else {
        // convert hex to npub (accepts comma separated list of hex)
        let split = args.key.as_str().trim_end().split(',');
        let hrp = match args.kind.unwrap() {
            Prefix::Npub => "npub",
            Prefix::Nsec => "nsec",
            Prefix::Note => "note",
        };
        for s in split {
            let encoded = bech32::encode(
                hrp,
                hex::decode(s)
                    .expect("could not decode provided key/note")
                    .to_base32(),
                Variant::Bech32,
            )
            .expect("Could not bech32-encode data");
            println!("{}", encoded);
        }
    }
}
