# Transaction crawler

Library to fetch transactions involving a set of target addresses:

- from an RPC node,
- in inverse chronological order target-wise,
- with a configurable amount of concurrent requests

See & run the example for more information: `RUST_LOG=info cargo run --example print_to_terminal` (note that the ingestion rate will be severely limited by the default RPC endpoint)
