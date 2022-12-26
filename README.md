# Key-Convertr

People are copy-pasting nostr private keys into webpages to convert between the original hex-encoding and bech32-encoding (specified in [NIP-19](https://github.com/nostr-protocol/nips/blob/master/19.md)). 

This is kind of bonkers, so here's a command-line utility that you can run to convert public/private keys between hex-encoding and the NIP-19 bech32 encoding. IT can also convert note id's.

To use it, you need `cargo` installed. Then from the `key-convertr` directory, just do `cargo install --path .`. It will put a program called `key-convertr` in your `$HOME/.cargo/bin` directory. 

Then, to convert from a hex-encoded pubkey to a bech32-encoded pubkey, you can do

```shell
$> key-convertr --kind npub 3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
```

Just provide the hex-encoded key or note-id and a `--kind` argument. The `kind`s supported are:
- npub
- nsec
- note

To convert from an npub to hex-encoding, you can do

```shell
$> key-convertr --to-hex npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6
3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d
```

