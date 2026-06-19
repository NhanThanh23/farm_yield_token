# farm_yield_token

## Project Title
farm_yield_token

## Project Description
Smallholder farmers often have to sell their harvest at a discount the moment it is collected because they cannot wait for distributors to find them. `farm_yield_token` turns that future harvest into a tokenized claim on the Stellar testnet: a farmer first registers a proof of harvest on-chain, then mints yield tokens against it, and distributors buy those tokens in advance. When a distributor purchases yield tokens the contract credits the farmer with sale proceeds that can be claimed later, giving the farmer working capital without having to give up the harvest itself.

## Project Vision
The long-term goal is to bring transparent, low-cost agricultural financing to any farmer with a phone and a Stellar wallet, by replacing paper receipts and verbal agreements with a shared on-chain record of who grew what, how much, and at what quality. Over time the same contract shape can power cooperatives, crop insurance products, and traceable supply chains for specialty goods (coffee, cocoa, rice) where proof of provenance matters to the end buyer.

## Key Features
- **Harvest deposit (`deposit_harvest`)** — a farmer registers a proof of harvest with a `harvest_id`, weight in kilograms, and a 1-100 quality score. Each registration is tied to the farmer's Stellar address via `require_auth`.
- **Yield tokenization (`mint_yield`)** — the same farmer mints a quantity of yield tokens against the deposit and sets the per-token price. A harvest can only be minted once, which prevents double issuance.
- **Distributor purchase (`purchase_yield`)** — distributors buy yield tokens in advance; the contract automatically tracks how many tokens are still available and credits the farmer's proceeds balance.
- **Proceeds claim (`claim_proceeds`)** — the farmer can pull the accumulated payment out of the contract at any time after a sale. The balance is reset to zero on claim to prevent double-spend.
- **Read views (`get_harvest`, `is_sold_out`)** — anyone can query the remaining yield for a harvest and check whether the full issuance has been sold to distributors.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** supply_chain dApp — see `contracts/farm_yield_token/src/lib.rs` for the full farm_yield_token business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CAX6OZ632GMT22BXBLXSWZGWKKPLGDQ5WP7AUPVCFIPKLEAGPXS46QNH`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/bfb49d43d4dad5cbcaaf15c625e26cdf7016f1122c60b1e23a6d72694f871ec4`


## Future Scope
- Replace the in-contract accounting units with a real payment asset (e.g. USDC on Stellar) by integrating a Stellar Asset Contract for the actual sale settlement.
- Add per-distributor holdings so yield tokens can be transferred, traded, or used as collateral between issuance and the physical delivery of the harvest.
- Add oracle-driven quality and weight verification (e.g. signed attestations from cooperatives or IoT sensors) before a harvest can be minted.
- Add partial-claim support and dispute resolution so that buyers can raise quality issues that automatically refund a portion of the proceeds.
- Build a small React/Freighter frontend that walks a farmer through deposit -> mint and a distributor through purchase -> track, with the contract ID wired in from this README.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `farm_yield_token` (supply_chain)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
