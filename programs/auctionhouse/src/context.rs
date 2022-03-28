use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_program, sysvar};
use anchor_spl::token::{Mint};
use crate::account::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(
    bump: u8,
    title: String,
    floor: u64,
    increment: u64,
    start_time: u64,
    end_time: u64,
    bidder_cap: u64,
    token_amount: u64
)]
pub struct CreateOpenAuction<'info> {
    #[account(init,
        seeds=[b"open auction", owner.to_account_info().key.as_ref(), name_seed(&title)],
        bump = bump,
        payer = owner,
        space = OpenAuction::LEN +
        VECTOR_LENGTH_PREFIX + (bidder_cap as usize)*PUBLIC_KEY_LENGTH +
        VECTOR_LENGTH_PREFIX + (bidder_cap as usize)*U64_LENGTH)]
    pub auction: Account<'info, OpenAuction>,
    #[account(mut)]
    pub auction_ata: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub owner_ata: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    #[account(address = spl_associated_token_account::ID)]
    pub ata_program: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct CancelOpenAuction<'info> {
    #[account(mut, has_one = owner)]
    pub auction: Account<'info, OpenAuction>,
    pub owner: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MakeOpenBid<'info> {
    #[account(mut)]
    pub auction: Account<'info, OpenAuction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ReclaimOpenBid<'info> {
    #[account(mut)]
    pub auction: Account<'info, OpenAuction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawItemOpen<'info> {
    #[account(mut, has_one = highest_bidder, has_one = mint)]
    pub auction: Account<'info, OpenAuction>,
    #[account(mut)]
    pub auction_ata: AccountInfo<'info>,
    #[account(mut)]
    pub highest_bidder: Signer<'info>,
    #[account(mut)]
    pub highest_bidder_ata: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    #[account(address = spl_associated_token_account::ID)]
    pub ata_program: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct WithdrawWinningBidOpen<'info> {
    #[account(mut, has_one = owner)]
    pub auction: Account<'info, OpenAuction>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ReclaimItemOpen<'info> {
    #[account(mut, has_one = owner, has_one = mint)]
    pub auction: Account<'info, OpenAuction>,
    #[account(mut)]
    pub auction_ata: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub owner_ata: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    #[account(address = spl_associated_token_account::ID)]
    pub ata_program: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(
    bump: u8,
    title: String,
    floor: u64,
    first_price: bool,
    start_time: u64,
    end_time: u64,
    reveal_period: u64,
    bidder_cap: u64,
    token_amount: u64
)]
pub struct CreateSealedAuction<'info> {
    #[account(init,
        seeds=[b"sealed auction", owner.to_account_info().key.as_ref(), name_seed(&title)],
        bump = bump,
        payer = owner,
        space = SealedAuction::LEN +
        VECTOR_LENGTH_PREFIX + (bidder_cap as usize)*PUBLIC_KEY_LENGTH +
        VECTOR_LENGTH_PREFIX + (bidder_cap as usize)*U8_LENGTH*32 +
        VECTOR_LENGTH_PREFIX + (bidder_cap as usize)*U64_LENGTH)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub auction_ata: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub owner_ata: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    #[account(address = spl_associated_token_account::ID)]
    pub ata_program: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct CancelSealedAuction<'info> {
    #[account(mut, has_one = owner)]
    pub auction: Account<'info, SealedAuction>,
    pub owner: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MakeSealedBid<'info> {
    #[account(mut)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ReclaimSealedBid<'info> {
    #[account(mut)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RevealSealedBid<'info> {
    #[account(mut)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawItemSealed<'info> {
    #[account(mut, has_one = highest_bidder, has_one = mint)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub auction_ata: AccountInfo<'info>,
    #[account(mut)]
    pub highest_bidder: Signer<'info>,
    #[account(mut)]
    pub highest_bidder_ata: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    #[account(address = spl_associated_token_account::ID)]
    pub ata_program: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct WithdrawWinningBidSealed<'info> {
    #[account(mut, has_one = owner)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ReclaimItemSealed<'info> {
    #[account(mut, has_one = owner, has_one = mint)]
    pub auction: Account<'info, SealedAuction>,
    #[account(mut)]
    pub auction_ata: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub owner_ata: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: AccountInfo<'info>,
    #[account(address = spl_associated_token_account::ID)]
    pub ata_program: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>
}