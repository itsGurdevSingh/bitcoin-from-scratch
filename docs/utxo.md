# UTXO Model

## Overview

Bitcoin uses a UTXO (Unspent Transaction Output) model rather than an account-based model.

Instead of tracking balances for each wallet, the network tracks a set of spendable outputs called UTXOs.

A user's wallet balance is the sum of all UTXOs that can be unlocked by that user.

---

## OutPoint

Every UTXO is uniquely identified by an OutPoint.

```text
(txid, vout)
```

Where:

* `txid` = Transaction ID that created the output
* `vout` = Output index within that transaction

Example:

```text
(tx100, 0)
(tx100, 1)
```

These represent two different outputs created by the same transaction.

---

## UTXO Structure

In this implementation, a UTXO contains:

```rust
pub struct Utxo {
    pub value: u64,
    pub script_pub_key: Script,
    pub is_coinbase: bool,
    pub block_height: u32,
}
```

### value

Amount stored in the output, denominated in satoshis.

### script_pub_key

The locking script that defines the conditions required to spend the UTXO.

Examples include:

* P2PKH
* Multisig
* Timelock scripts

### is_coinbase

Indicates whether the UTXO originated from a coinbase transaction.

Coinbase outputs require a maturity period before they become spendable.

### block_height

The block height at which the UTXO was created.

Used for consensus rules such as coinbase maturity.

---

## UTXO Set

The UTXO set represents the current spendable state of the ledger.

Conceptually:

```rust
HashMap<OutPoint, Utxo>
```

Example:

```text
(tx100, 0) -> 5 BTC
(tx101, 1) -> 2 BTC
(tx105, 0) -> 10 BTC
```

Only unspent outputs exist in the UTXO set.

When an output is spent, it is removed from the set.

---

## Spending a UTXO

Transaction validation follows this process:

1. Locate the referenced OutPoint.
2. Verify the UTXO exists.
3. Verify spending conditions defined by the script.
4. Remove the consumed UTXO from the set.
5. Create new UTXOs from transaction outputs.

Example:

```text
Before:

(tx100, 0) -> 5 BTC

Transaction:

Input:
(tx100, 0)

Outputs:
2 BTC -> Alam
2.99 BTC -> Gurdev

Fee:
0.01 BTC

After:

(tx200, 0) -> 2 BTC
(tx200, 1) -> 2.99 BTC
```

The original UTXO no longer exists.

---

## Design Decisions

### No Balance Storage

The system does not store wallet balances.

Balances are derived from spendable UTXOs.

### No Ownership Field

A UTXO does not directly store an owner.

Ownership is expressed through the locking script (`script_pub_key`).

### No UTXO Mutation

Existing UTXOs cannot be modified.

A UTXO may only be:

* Created
* Spent

This mirrors Bitcoin's immutable transaction model.

---

## Current Scope

Implemented:

* OutPoint
* Utxo
* UtxoSet
* UTXO insertion
* UTXO lookup
* UTXO spending
* UTXO existence checks
* UTXO invariants and tests

```
```
