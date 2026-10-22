# Accounts and transfers

Moving native tokens between accounts, at every combination of public and private.

> **Not yet walked through.** This chapter documents the command surface, verified against `lez/wallet/src/cli/programs/native_token_transfer.rs`, and deliberately shows no command output. The version it replaces quoted transcripts nobody had produced, and several of them were impossible. Writing the step-by-step walkthrough needs someone to run it against a sequencer and paste what actually comes back.

## Making accounts

```bash
wallet account new public  --label alice
wallet account new private --label alice-private
wallet account list
```

A label is unique across the wallet and usable anywhere an account id is. New accounts are uninitialized until something writes to them.

The payer of a transfer must already hold tokens: a fresh account cannot fund its own first transaction. On a local devnet, start from a genesis-funded account.

## Sending

One command, and which flags you use depends on what you know about the recipient.

```bash
wallet auth-transfer send --from <account> --amount <n> <recipient flags>
```

| Recipient | Flag | What it takes |
|---|---|---|
| An account this wallet owns, public or private | `--to` | a base58 account id with its `Public/` or `Private/` prefix, or a label |
| Someone else's private account | `--to-npk` and `--to-vpk` | the nullifier public key as 32-byte hex, and the ML-KEM-768 viewing key as hex, 1184 bytes |
| Someone else's private account, from a file | `--to-keys` | a path to a keys file holding npk and vpk on separate lines. Replaces the two flags above |
| A public account that has never been used | `--to-pk` | the recipient's public key |

`--from` and `--amount` are always required. The recipient flags are mutually exclusive in the combinations you would expect: `--to-keys` conflicts with `--to-npk` and `--to-vpk`, and `--to-pk` conflicts with all of them.

The rule the command's own documentation states: for a private recipient, `--to` and the key pair are mutually exclusive patterns. Use `--to` for an account this wallet owns, and the keys for anyone else's.

## Getting someone's keys

A private account's address is derived from its two public keys, so paying one means holding them. The owner exports them:

```bash
wallet account show-keys --account-id Private/<id> > alice.keys
```

That file is what `--to-keys` takes. `show-keys` works only for private accounts.

There is no identifier to choose. A private account id is `SHA256(prefix ‖ npk ‖ vpk)`, so one key pair is exactly one regular private account. Material written before `v0.4.0` describes a 32-byte identifier that diversified them; it was removed. See [Which interface you are writing for](../understand/which-interface.md).

## Where the money lands, and why it matters

This is the part most easily got wrong, and it decides which command you run next.

If the recipient is an account this wallet owns, the credit is applied in the transaction. Read it back with `wallet account get`, after `wallet account sync-private` for a private account.

If the recipient is someone else's private account, named by keys rather than owned here, the credit cannot be applied: only its owner can produce the proof that advances it. So it is stored on chain and waits. The recipient collects it:

```bash
wallet pending list
wallet pending receive --position <n>
```

No amount of `wallet account sync-private` will surface it, because it is not in the account yet. [Pending messages](pending-messages.md) explains why that asymmetry exists.

## Reading the result

```bash
wallet account get --scope balance --account-id Public/<id>
wallet account sync-private
wallet account get --scope balance --account-id Private/<id>
```

The sync is not optional for a private account, and it must come first. [Reading the chain](reading-the-chain.md) covers the rest.

## What is still missing here

A worked example with real ids and real output, for each of the four recipient cases. A note on what a failed transfer looks like. And the public-to-private and private-to-private combinations timed, since a private sender pays proving time that a public one does not.
