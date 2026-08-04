# token-faucet

SPL token faucet: anyone mints themselves tokens, bounded by a per-wallet rate limit and a global
max supply, both set by the faucet's admin. One deployment hosts any number of independent
faucets, each namespaced by a `u64` seed; whoever initializes a faucet becomes its admin.

Anchor program. anchor-lang `1.1.2` (with `init-if-needed`), anchor-spl `1.1.2` via
`token_interface`, so it runs against SPL Token or Token-2022. Program ID:
`2m78e8ia8SuG9jxnLFtVBmvi5TsL8zaMWq3FRENAbRzZ`.

## Accounts and seeds

| Account         | Seeds                                   | Notes |
|-----------------|-----------------------------------------|-------|
| `faucet_config` | `["token-faucet", seed_le]`             | `admin`, `max_supply`, `mint_timeout`, `mint_limit`, `bump` — 65 bytes (8 + 57) |
| `mint`          | `["mint", seed_le]`                     | Mint authority is this PDA itself, so only the program can ever mint |
| `mint_timeout`  | `["mint-timeout", seed_le, payer]`      | Per-wallet, per-faucet rate-limit window — 24 bytes (8 + 16), no bump stored |
| user token account | ATA of (payer/recipient, mint)       | `init_if_needed`, payer funds it |

Each faucet's admin is the pubkey that signed its `initialize`, stored in `faucet_config.admin`
and enforced with `has_one = admin` on `update_config`. No keypair material lives in the repo.

## Instructions

| Instruction | Access | Behavior |
|-------------|--------|----------|
| `initialize(seed, decimals, max_supply, mint_timeout, mint_limit)` | anyone; signer becomes admin | Creates config + mint for one faucet instance; rejects a negative `mint_timeout` |
| `update_config(seed, max_supply, mint_timeout, mint_limit)` | that faucet's admin | Rejects `max_supply` below the already-minted supply and a negative `mint_timeout` |
| `mint_token(seed, amount)` | anyone | Rate-limited self-mint into the payer's ATA |
| `transfer_token(seed, amount)` | token holder | `transfer_checked` to any recipient; recipient ATA created if needed, sender pays |

## Rate limiting

Fixed window per wallet. On `mint_token`: if `now >= timeout_reset`, the window resets —
`amount_minted = amount`, `timeout_reset = now + config.mint_timeout`; otherwise `amount`
accumulates into `amount_minted`. Then two gates: `amount_minted <= mint_limit`
(`MintTimeoutExceeded`) and `mint.supply + amount <= max_supply` (`MintExceedsMaxSupply`), all
arithmetic checked. The window end is fixed by the first mint of the window; later mints don't
extend it. A fresh `MintTimeout` account has `timeout_reset = 0`, so the first mint always takes
the reset branch.

## Validation

Config updates require the stored admin as signer (`has_one = admin`); every PDA re-derived from
its seeds (`faucet_config` with its stored bump, `mint` and `mint_timeout` re-found each time);
`mint_timeout` must be non-negative; ATAs constrained to (authority, mint, token_program) so a
wrong token account fails derivation; `recipient` is an `UncheckedAccount` on purpose — it only
serves as the ATA derivation key. Minting CPIs are signed with the mint PDA's seeds.

## Scope

The rate limit is per wallet — a permissionless faucet cannot distinguish one person's wallets —
so the hard bound on total issuance is `max_supply`; `MintTimeout` accounts live for the lifetime
of the faucet.

## Build and test

```bash
anchor build   # produces target/deploy/token_faucet.so, which the test embeds
cargo test
```

`programs/token-faucet/tests/test_token_faucet.rs` runs in LiteSVM with a freshly generated
admin keypair: initialize (asserts the mint authority is the mint PDA and the stored admin is
the initializer), a mint rejected against the zeroed config, a non-admin `update_config`
rejected, a successful `update_config`, a successful mint, and a full-balance transfer to
another wallet's ATA.
