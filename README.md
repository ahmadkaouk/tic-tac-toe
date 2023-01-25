
## Building the contract
To create a Wasm binary for the contract, run the following command:
```bash
cargo build --target wasm32-unknown-unknown --release
```

## TODO
- Do we need to add a limit for the number of games that can be created by a user ?
- Do we need to have some kind of a elapsed block time to clear outdated games ?
- Does the player need to pay some fee to create a game ?