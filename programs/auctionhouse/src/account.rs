use anchor_lang::prelude::*;

#[account]
pub struct OpenAuction {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub token_amount: u64,

    pub start_time: u64,
    pub end_time: u64,
    pub cancelled: bool,

    pub title: String,

    pub bidder_cap: u64,
    pub bidders: Vec<Pubkey>,
    pub bids: Vec<u64>,

    pub highest_bidder: Pubkey,
    pub highest_bid: u64,

    pub bid_floor: u64,
    pub min_bid_increment: u64,

    pub bump: u8,
}

#[account]
pub struct SealedAuction {
    pub first_price: bool,

    pub owner: Pubkey,
    pub mint: Pubkey,
    pub token_amount: u64,

    pub start_time: u64,
    pub end_time: u64,
    pub reveal_period: u64,
    pub cancelled: bool,

    pub title: String,

    pub bidder_cap: u64,
    pub bidders: Vec<Pubkey>,
    pub sealed_bids: Vec<[u8; 32]>,
    pub fake_bids: Vec<u64>,

    pub highest_bidder: Pubkey,
    pub highest_bid: u64,
    pub second_highest_bid: u64,

    pub bid_floor: u64,
    pub winning_bid_withdrawn: bool,

    pub bump: u8,
}

pub const DISCRIMINATOR_LENGTH: usize = 8;
pub const PUBLIC_KEY_LENGTH: usize = 32;
pub const U64_LENGTH: usize = 8;
pub const U8_LENGTH: usize = 1;
pub const ENUM_LENGTH: usize = 1;
pub const BOOL_LENGTH: usize = 1;
pub const STRING_LENGTH_PREFIX: usize = 4;
pub const MAX_TITLE_LENGTH: usize = 50 * 4;
pub const VECTOR_LENGTH_PREFIX: usize = 4;

impl OpenAuction {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH // owner
        + PUBLIC_KEY_LENGTH // mint
        + U64_LENGTH // token amount
        + U64_LENGTH // start time
        + U64_LENGTH // end time
        + BOOL_LENGTH // cancelled
        + STRING_LENGTH_PREFIX + MAX_TITLE_LENGTH // title
        + U64_LENGTH // bidder cap
        + PUBLIC_KEY_LENGTH // highest bidder
        + U64_LENGTH // highest bid
        + U64_LENGTH // bid floor
        + U64_LENGTH // min bid increment
        + U8_LENGTH; // bump
}

impl SealedAuction {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + BOOL_LENGTH // first price
        + PUBLIC_KEY_LENGTH // owner
        + PUBLIC_KEY_LENGTH // mint
        + U64_LENGTH // token amount
        + U64_LENGTH // start time
        + U64_LENGTH // end time
        + U64_LENGTH // reveal period
        + BOOL_LENGTH // cancelled
        + STRING_LENGTH_PREFIX + MAX_TITLE_LENGTH // title
        + U64_LENGTH // bidder cap
        + PUBLIC_KEY_LENGTH // highest bidder
        + U64_LENGTH // highest bid
        + U64_LENGTH // second highest bid
        + U64_LENGTH // bid floor
        + BOOL_LENGTH // winning bid withdrawn
        + U8_LENGTH; // bump
}