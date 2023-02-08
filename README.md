# Key-Convertr
People are copy-pasting nostr private keys into webpages to convert between the original hex-encoding and bech32-encoding (specified in [NIP-19](https://github.com/nostr-protocol/nips/blob/master/19.md)).

This is kind of bonkers, so here's a command-line utility that you can run to convert public/private keys between hex-encoding and the NIP-19 bech32 encoding. It can also convert note id's, and even convert a list of public keys from NIP5 configured domains from hex to bech32 in single shot.

## Features:
>Note: "keys" + "nip5" arguments accept multiple inputs for bulk operations
- convert from bech32 (npub/nsec/note) to hex
- convert from hex to bech32 (npub/nsec/note)
- supports NIP-05 domain identifiers:
    - calls given nip5 domain to get nostr.json containing user pubkeys (located at domain.com/.well-known/nostr.json)
    - extracts and converts all pubkeys from hex to bech32 format
    - as specified in [NIP-05] (https://github.com/nostr-protocol/nips/blob/master/05.md)
---
## Installation
### Building with Cargo
If you have `cargo` installed, then from the `key-convertr` directory, just do `cargo install --path .`. It will put a program called `key-convertr` in your `$HOME/.cargo/bin` directory.

Then, to convert from a hex-encoded pubkey to a bech32-encoded pubkey, you can do

```shell
$> key-convertr --kind npub 3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
```

### Running with Docker
If you have docker installed, you can simply do `docker run --rm ghcr.io/rot13maxi/key-convertr:main` to download and run the latest `main` revision of the tool. Then just follow the instructions below. For example,
```shell
$> docker run --rm ghcr.io/rot13maxi/key-convertr:main --kind npub 3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
```

### Building with Docker
If you have docker installed, you can do `docker build -f docker/Dockerfile -t key-convertr .` to build a container (no local cargo install required!). Then just do `docker run --rm key-convertr [args]` to run it. For example:

```shell
$> docker run --rm key-convertr --kind npub 3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
```
---
## Usage

Just provide the hex-encoded key or note-id and a `--kind` argument. The `kind`s supported are:
- npub
- nsec
- note

To convert from an `bech32(npub/nsec/note) to hex-encoding`, you can do

```shell
$> key-convertr --to-hex npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
```

To convert from an `hex-encoding to bech32 (npub/nsec/note)`, you can do

```shell
$> key-convertr --kind npub 3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
```
To convert list of public keys for a given `NIP-05 domain identifier`, you can
```shell
$> key-convertr --nip5 satoshivibes.com
Aurelius,npub169yu4d6xl8a6d4xvfyhstjdxr0cc7qtfq5pch5kwv4k2wkapa3pq8v2thg
frontrunbitcoin,npub199samvtne4sahhdkr6dcq3mauuqs7k7r3eulufp2y2lf04zewglqtu9mfd
lukeonchain,npub138guayty78ch9k42n3uyz5ch3jcaa3u390647hwq0c83m2lypekq6wk36k
```
- you can also enable NIP5 related stats via "--nip-stats" or "-s";
    - this prints out summary stats in this format: `domain=SomeName.com|count=21`
    - this output could then be parsed (if needed) by other dev tools/libraries
```shell
$> key-convertr --nip5 strike.me --nip-stats
ZAurelius,npub169yu4d6xl8a6d4xvfyhstjdxr0cc7qtfq5pch5kwv4k2wkapa3pq8v2thg
frontrunbitcoin,npub199samvtne4sahhdkr6dcq3mauuqs7k7r3eulufp2y2lf04zewglqtu9mfd
lukeonchain,npub138guayty78ch9k42n3uyz5ch3jcaa3u390647hwq0c83m2lypekq6wk36k
domain=satoshivibes.com|count=3
```

You can also pass `multiple keys/notes` to the tool by putting a comma or space between them:
- example with multiple "keys"
```shell
$> key-convertr --kind npub 3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d,863883611bdbe6291c081fb8775908a7ab0cb04b608405ec1e85e9f938020a98
npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
npub1scugxcgmm0nzj8qgr7u8wkgg574sevztvzzqtmq7sh5ljwqzp2vqf45w5j
```
- example with multiple NIP5 domains (output omitted due to size)
```
$> key-convertr --nip5 nostrplebs.com,strike.me,satoshivibes.com
```
---
## TODO
Optimizations:
- multi threaded "--nip5" logic (multiple domains converted in parallel; some lists will be very large i.e nostrplebs.com)