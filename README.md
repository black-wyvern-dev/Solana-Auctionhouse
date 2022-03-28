# Solana Auctionhouse

Auction protocol for open ascending (English), sealed first-price, and sealed second-price (Vickrey) auctions

## Implementation Details

Both types of auctions have SPL tokens (e.g. an NFT) as the auction item and accept bids in SOL.

The maximum number of active bidders is capped by an argument provided on auction creation. This is because the Solana runtime needs to know how much space to allocate for the auction account.

The auction account is a PDA owned by the auctionhouse program to allow it to act as an escrow.

### Open Auction

- The seller creates an open auction and escrows their SPL tokens
- Bidders make public bids and their SOL is escrowed in the auction PDA
- Bidders can reclaim their SOL at any time, unless they're the current highest bidder
- Bidding is cumulative, so a bidder with 70 SOL escrowed can send an extra 10.1 SOL to beat a bid of 80 SOL
- When the auction ends, the winner can withdraw the SPL tokens and the seller can withdraw the winning bid
- The seller can cancel the auction any time before it ends, allowing them to reclaim the SPL tokens and allowing every bidder to reclaim their bids

### Sealed Auction

- The seller creates a sealed auction and escrows their SPL tokens
- Bidders make sealed bids as follows:
    - Compute the `Keccak256` hash of the true bid and some large random nonce
    - Send this hash to the program along with an amount of SOL greater than the true bid
- Bidders can reclaim their SOL at any time
- Only one sealed bid is allowed per bidder
- Once the auction ends, the reveal period starts:
    - Until the reveal period ends, bidders can send their true bid and nonce for the program to verify
    - If the true bid is high enough, it becomes the new highest bid
    - Otherwise, the bidder is refunded all of their escrowed SOL
- Once the reveal period ends, the highest bidder at that time can withdraw the SPL tokens and the seller can withdraw the highest bid
    - The highest bidder is refunded the difference between their escrowed SOL and their true bid
- If this is a second-price auction, then the above logic takes place but with the seller being paid the true bid of the second highest bidder and the highest bidder refunded the difference between their escrowed SOL and the second highest bid
- The seller can cancel the auction any time before the reveal period starts, allowing them to reclaim the SPL tokens

## Quickstart

Install [Anchor](https://github.com/project-serum/anchor) if necessary

```
$ git clone https://github.com/udbhav1/solana-auctionhouse && cd solana-auctionhouse
$ yarn
$ anchor test
```

The tests use a `delay()` function to wait for the auction period and reveal period to end. Depending on how fast your machine runs the test suite, you may need to modify the `auctionEndDelay` and `revealPeriodEndDelay` variables.

## Possible Improvements

- Allow unlimited bidders by having each bidder fund a PDA derived from their public key that contains metadata about their bid
- Allow the owner to close the auction PDA and reclaim its rent
- Add an "instant buy" price that immediately ends the auction when reached
- Allow bids in whitelisted SPL tokens
- Allow multiple mints for the auction item(s) so many different tokens can be auctioned together
