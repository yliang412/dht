# A Distributed Hash Table (WIP)

Currently, the system only supports single node concurrent key-value store, but the plan is to implement consistent hashing and make a multi-node key-value store that supports load-balancing and replication.

## Demo
**Server**
```console
$ RUST_LOG=info cargo run --bin server -- --port 15213
```

**Client**
```console
$ cargo run --bin client -- --server-addr "[::1]:15213"
```

```console
dht> help
Dht Client Interface

Usage: dht <COMMAND>

Commands:
  get   Get the value in the store associated with the key
  set   Insert key-value pair into the store
  del   Delete the key-value pair in the store
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

dht> set rust best-language-ever
OK
dht> get rust
best-language-ever
```

