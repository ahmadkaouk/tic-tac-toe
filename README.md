# Tic-Tac-Toe Game

This is a simple Tic-Tac-Toe game written in Rust using CosmWasm. It is a two-player game, where each player takes turns to mark a 3x3 grid. The player who succeeds in placing three of their marks in a horizontal, vertical, or diagonal row is the winner.

## How to play

The cells of the grid are numbered from 0 to 8, as shown below:

```
 0 | 1 | 2
---+---+---
 3 | 4 | 5
---+---+---
 6 | 7 | 8
```

- All state of the game live on-chain. State includes open games(invitations), games currently in progress and completed games.
- Any user can submit a transaction to the network to invite others to start a game (i.e. create an open game).
- Other users may submit transactions to accept invitations. When an invitation is accepted, the game starts.
- Both users submit transactions to the network to make their moves until the game is complete.
- The game needs to support multiple concurrent games sessions/players. 
### Roles of X and O

Roles of "X" and "O" are defined as follows: The user's public keys are concatenated and the result is hashed. If the first bit of the output is 0, then the game's initiator (whoever posted the invitation) plays "O" and the second player plays "X" and vice versa. “X” has the first moves.


## Smart Contract Interface

The smart contract interface is defined in the [contract](./contract) directory. The contract is written in Rust and uses the [CosmWasm](https:://github.com/CosmWasm/cosmwasm) framework. The contract is compiled to WebAssembly and deployed to the blockchain. The contract is written in a way that it can be used with any blockchain that supports CosmWasm.

The contract exposes the following endpoints:

### Instantiate

The contract is instantiated with the following Message:

```rust
pub struct InstantiateMsg {}
```

### Execute

The contract can execute the following Messages:

```rust
pub enum ExecuteMsg {
    /// Invite a player to play a game.
    Invite {
        /// The address of the player to invite.
        guest: String,
    },
    /// Accept an invitation to play a game.
    Accept {
        /// The address of the player who invited you.
        host: String,
    },
    /// Reject an invitation to play a game.
    Reject {
        /// The address of the player who invited you.
        host: String,
    },
    /// Play a move in the game.
    Play {
        /// The address of the host of the game.
        host: String,
        /// The address of the guest of the game.
        guest: String,
        /// The cell to play in.
        cell: usize,
    },
}
```

### Query

The contract can be queried with the following Messages:

```rust
pub enum QueryMsg {
    /// Get all the games between two players.
    Games {
        /// The address of the host of the game.
        host: String,
        /// The address of the guest of the game.
        guest: String,
    },
    /// Get all the games for all players.
    AllGamesList {},
}
```

## Building

### Smart contracts
To build the contract, run:

```bash
cargo run-script optimize
```

This will use the CosmWasm optimizer to reduce the size of the contract.

### Schemas

To generate the JSON schema for the contract, run:

```bash
cargo schema
```

## Test

To run tests, run:

```bash
cargo test
```
