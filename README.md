# Bitcoin From Scratch

**A Bitcoin protocol implementation built from scratch in Rust to understand how Bitcoin actually works at the protocol and systems level.**

This project is an attempt to build the core components of Bitcoin from the ground up rather than relying on existing Bitcoin libraries.

The goal is not to create another cryptocurrency or a production-ready Bitcoin client.

The goal is to understand the engineering decisions behind Bitcoin by implementing its fundamental mechanisms ourselves — including transaction serialization, Script execution, UTXO state management, block validation, chain selection, reorganizations, and peer-to-peer communication.

The implementation is written in **Rust**, with the project evolving alongside a deeper study of Bitcoin's protocol specifications and Bitcoin Improvement Proposals (BIPs).

---

# Why Build Bitcoin From Scratch?

Reading about Bitcoin is very different from implementing it.

Concepts such as:

* UTXOs
* transaction validation
* transaction IDs
* Script
* signatures
* block headers
* Proof of Work
* difficulty
* chain selection
* forks
* reorganizations
* mempools
* transaction relay
* peer-to-peer communication

can appear simple individually.

The complexity emerges from how these components interact.

This project is an attempt to make those interactions concrete by implementing them step by step.

---

# Architecture

The project is organized as a Rust workspace so that protocol components can remain separated and independently tested.

```text
bitcoin-from-scratch/
│
├── crates/
│   ├── btc-core/
│   └── ...
│
├── docs/
│
├── src/
│
├── Cargo.toml
└── Cargo.lock
```

The architecture is intentionally evolving as the protocol implementation becomes more complete.

The objective is to keep fundamental Bitcoin primitives separated from higher-level node functionality.

---

# Implemented Components

## Transactions

The project implements Bitcoin transaction structures and serialization/deserialization.

This includes concepts such as:

* Transaction versions
* Inputs
* Outputs
* OutPoints
* TxIDs
* Sequence numbers
* Locktime
* CompactSize encoding
* Transaction serialization
* Transaction deserialization

The implementation also accounts for the differences introduced by SegWit transaction serialization.

---

# UTXO Model

Bitcoin does not maintain account balances in the traditional sense.

Instead, the ledger is represented through **Unspent Transaction Outputs (UTXOs)**.

The project models this state explicitly.

Conceptually:

```text
Transaction
     │
     ├── Inputs ──► consume previous UTXOs
     │
     └── Outputs ─► create new UTXOs
```

Transaction validation therefore becomes a state-transition problem:

```text
Previous UTXO State
        │
        ▼
   Validate TX
        │
        ▼
New UTXO State
```

---

# Bitcoin Script

A Script virtual machine is implemented to execute Bitcoin's stack-based transaction scripts.

The implementation covers Bitcoin Script concepts including operations such as:

```text
OP_DUP
OP_HASH160
OP_EQUAL
OP_EQUALVERIFY
OP_CHECKSIG
OP_IF
OP_ELSE
OP_ENDIF
```

The Script layer is used to understand how Bitcoin spending conditions are actually evaluated rather than treating signatures and addresses as black boxes.

---

# Transaction Validation

Transaction validation combines multiple protocol rules.

The validation pipeline is designed around checks such as:

```text
Transaction
    │
    ├── Structural validation
    │
    ├── Input validation
    │
    ├── UTXO existence
    │
    ├── Script validation
    │
    ├── Signature validation
    │
    ├── Value conservation
    │
    └── Policy / consensus constraints
```

The implementation is being expanded incrementally as additional consensus rules are introduced.

---

# SegWit

Segregated Witness support is being implemented according to the relevant Bitcoin protocol specifications.

This includes concepts such as:

* SegWit transaction serialization
* Marker and flag
* Witness data
* Witness stacks
* Witness transaction IDs
* Signature-hashing differences
* Weight / virtual size concepts

The implementation is being developed with **BIP 141, BIP 143 and BIP 144** as important protocol references.

---

# Blocks

The blockchain layer models blocks and their relationship to previous blocks.

A simplified block relationship is:

```text
Genesis
   │
   ▼
Block A
   │
   ▼
Block B
   │
   ▼
Block C
```

Each block commits to its predecessor through the previous block hash.

Block validation includes structural and consensus-related checks, with additional constraints being added as the implementation progresses.

---

# Proof of Work

The project models Bitcoin's Proof-of-Work mechanism.

The basic mining process is:

```text
Block Header
     │
     ▼
Change Nonce
     │
     ▼
Double SHA-256
     │
     ▼
Hash <= Target ?
   │         │
  No        Yes
   │         │
   └───►    Block Found
```

The implementation also explores:

* Nonce search
* Difficulty targets
* Compact target representation
* Difficulty adjustment
* Block header hashing

---

# Blockchain State and Reorganizations

One of the more complex parts of the project is handling competing branches.

Bitcoin does not simply append every received block to one permanent list.

Nodes can temporarily observe:

```text
        D
       /
A ─ B ─ C
       \
        E ─ F
```

When a competing chain becomes the valid preferred chain, the node must reorganize its active state.

The implementation therefore explores:

* Block ancestry
* Chain traversal
* Common ancestor discovery
* Branch comparison
* Disconnecting blocks
* Reconnecting blocks
* UTXO state transitions during reorgs
* Overlay-based state handling

This part of the project is particularly useful for understanding why a blockchain is more accurately modeled as a **state machine over a block tree**, rather than simply a linked list.

---

# Mempool

The project is being extended toward transaction-pool functionality.

The mempool represents transactions that have been validated by a node but have not yet been included in a block.

```text
Wallet
  │
  ▼
Transaction
  │
  ▼
Node
  │
  ▼
Mempool
  │
  ▼
Block
```

Future work includes transaction relay and more detailed mempool policy.

---

# Peer-to-Peer Networking

The next major layer is the Bitcoin P2P network.

The networking layer will allow nodes to communicate with peers and exchange protocol messages.

The intended flow is:

```text
                ┌─────────────┐
                │    Node A   │
                └──────┬──────┘
                       │
                 P2P Protocol
                       │
                ┌──────▼──────┐
                │    Node B   │
                └──────┬──────┘
                       │
                P2P Protocol
                       │
                ┌──────▼──────┐
                │    Node C   │
                └─────────────┘
```

This layer will eventually support concepts such as:

* Peer discovery
* Connection management
* Version handshake
* Transaction propagation
* Block propagation
* Inventory messages
* Block requests
* Transaction requests
* Peer synchronization

The objective is to move from a local blockchain implementation toward an actual networked Bitcoin node model.

---

# Protocol References

The implementation is guided by Bitcoin's protocol documentation and relevant BIPs.

Important areas include:

* Transaction serialization
* SegWit
* Signature hashing
* P2P protocol
* Script
* Block validation
* Consensus rules

The project intentionally studies the protocol alongside implementation instead of treating Bitcoin's behavior as unexplained magic.

---

# Technology Stack

| Technology                | Purpose                                        |
| ------------------------- | ---------------------------------------------- |
| **Rust**                  | Core implementation                            |
| **Cargo**                 | Build system and workspace management          |
| **SHA-256**               | Hashing and Proof of Work                      |
| **secp256k1 / ECDSA**     | Transaction signatures                         |
| **Bitcoin Script**        | Spending-condition execution                   |
| **UTXO Model**            | Ledger state                                   |
| **Bitcoin Wire Protocol** | Peer-to-peer networking                        |
| **BIPs**                  | Protocol specifications and reference material |

---

# Engineering Concepts

This project is primarily about systems engineering.

It explores:

* State machines
* Immutable data structures
* Cryptographic primitives
* Serialization protocols
* Binary network protocols
* UTXO-based ledger design
* Consensus validation
* Chain selection
* Fork handling
* State rollback
* Reorganization
* Peer-to-peer networking
* Distributed systems
* Rust ownership and borrowing
* Error handling
* Modular architecture

---

# Development Philosophy

The implementation follows a simple principle:

> **Don't hide the protocol behind a library. Implement the mechanism and understand why it works.**

For example, instead of simply calling an existing transaction parser, the project implements Bitcoin serialization and deserialization.

Instead of using an existing Script interpreter, the project builds the VM.

Instead of representing the blockchain as a list of blocks, the project models block relationships and state transitions.

The purpose is to understand the machinery underneath the abstractions.

---

# Project Status

This project is **actively under development**.

The implementation is intentionally being built incrementally.

### Current direction

```text
Cryptographic primitives
        │
        ▼
Transaction primitives
        │
        ▼
Script VM
        │
        ▼
Transaction validation
        │
        ▼
UTXO / Ledger
        │
        ▼
Block validation
        │
        ▼
Blockchain / Reorgs
        │
        ▼
Mempool
        │
        ▼
P2P Networking
        │
        ▼
Networked Bitcoin Node
```

The long-term objective is to have a small but coherent Bitcoin node implementation capable of participating in a controlled Bitcoin-like P2P environment.

---

# Disclaimer

This is an educational implementation and **is not intended to be used with real Bitcoin funds or as a replacement for Bitcoin Core**.

Bitcoin is a security-critical protocol, and production implementations contain substantially more consensus, networking, validation, performance, and security considerations than this project currently implements.

The project exists to learn those systems by building them from first principles.

---

# License

See the repository for license information.
