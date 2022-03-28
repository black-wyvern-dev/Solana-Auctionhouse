pub mod account;
pub mod context;
pub mod error;
pub mod utils;
use anchor_lang::prelude::*;
use account::*;
use context::*;
use error::*;
use utils::*;

declare_id!("6tEWNsQDT8KZ2EDZRBa4CHRTxPESk6tvSJEwiddwSxkh");

#[program]
pub mod auctionhouse {
    use super::*;
    pub fn create_open_auction(
        ctx: Context<CreateOpenAuction>,
        bump: u8,
        title: String,
        floor: u64,
        increment: u64,
        start_time: u64,
        end_time: u64,
        bidder_cap: u64,
        token_amount: u64,
    ) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;
        let auction_ata = &ctx.accounts.auction_ata;
        let owner = &ctx.accounts.owner;
        let owner_ata = &ctx.accounts.owner_ata;
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let ata_program = &ctx.accounts.ata_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(title.chars().count() <= 50, Err(AuctionError::TitleOverflow.into()));
        require!(increment != 0, Err(AuctionError::InvalidIncrement.into()));
        require!(token_amount != 0, Err(AuctionError::InvalidTokenAmount.into()));
        require!(start_time < end_time, Err(AuctionError::InvalidStartTime.into()));
        require!(cur_time > start_time || start_time == 0, Err(AuctionError::InvalidStartTime.into()));
        require!(cur_time < end_time, Err(AuctionError::InvalidEndTime.into()));
        require!(floor > 0, Err(AuctionError::InvalidBidFloor.into()));

        auction.owner = *owner.key;
        auction.mint = mint.key();
        auction.token_amount = token_amount;

        auction.start_time = if start_time == 0 { cur_time } else { start_time };
        auction.end_time = end_time;
        auction.cancelled = false;

        auction.title = title;

        auction.bidder_cap = bidder_cap;
        auction.highest_bid = 0;
        auction.bid_floor = floor;
        auction.min_bid_increment = increment;

        auction.bump = bump;

        create_ata(
            owner.to_account_info(),
            auction.to_account_info(),
            mint.to_account_info(),
            auction_ata.to_account_info(),
            token_program.to_account_info(),
            ata_program.to_account_info(),
            system_program.to_account_info(),
            rent_sysvar.to_account_info()
        )?;

        transfer_spl(
            owner.to_account_info(),
            owner_ata.to_account_info(),
            auction_ata.to_account_info(),
            token_amount,
            token_program.to_account_info(),
            &[]
        )?;

        Ok(())
    }

    pub fn cancel_open_auction(ctx: Context<CancelOpenAuction>) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(cur_time < auction.end_time, Err(AuctionError::CannotCancelAfterClose.into()));

        auction.cancelled = true;

        Ok(())
    }

    pub fn make_open_bid(ctx: Context<MakeOpenBid>, amount: u64) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;
        let bidder: &Signer = &ctx.accounts.bidder;
        let system_program = &ctx.accounts.system_program;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.start_time, Err(AuctionError::BidBeforeStart.into()));
        require!(cur_time < auction.end_time, Err(AuctionError::BidAfterClose.into()));
        require!(*bidder.key != auction.owner, Err(AuctionError::OwnerCannotBid.into()));

        let index = auction.bidders.iter().position(|&x| x == *bidder.key);

        // new amount plus already bid amount
        let mut total_bid = amount;
        let mut new_bidder = false;

        if let None = index {
            require!(
                auction.bidders.len() < (auction.bidder_cap as usize),
                Err(AuctionError::BidderCapReached.into())
            );
            new_bidder = true;
        } else {
            total_bid += auction.bids[index.unwrap()];
        }

        require!(
            total_bid > auction.bid_floor,
            Err(AuctionError::UnderBidFloor.into())
        );
        require!(
            total_bid > (auction.highest_bid + auction.min_bid_increment),
            Err(AuctionError::InsufficientBid.into())
        );

        if new_bidder {
            auction.bidders.push(*bidder.key);
            auction.bids.push(total_bid);
        } else {
            auction.bids[index.unwrap()] = total_bid;
        }

        auction.highest_bidder = *bidder.key;
        auction.highest_bid = total_bid;

        transfer_sol(
            bidder.to_account_info(),
            auction.to_account_info(),
            amount,
            system_program.to_account_info()
        )?;

        Ok(())
    }

    pub fn reclaim_open_bid(ctx: Context<ReclaimOpenBid>) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;
        let bidder: &Signer = &ctx.accounts.bidder;

        let index = auction.bidders.iter().position(|&x| x == *bidder.key);

        if let None = index {
            return Err(AuctionError::NotBidder.into())
        } else if *bidder.key == auction.highest_bidder && !auction.cancelled {
            return Err(AuctionError::WinnerCannotWithdrawBid.into())
        } else {
            let bid = auction.bids[index.unwrap()];

            auction.bidders.remove(index.unwrap());
            auction.bids.remove(index.unwrap());

            let src = &mut auction.to_account_info();
            let dst = &mut bidder.to_account_info();

            transfer_from_owned_account(src, dst, bid)?;
        }

        Ok(())
    }

    pub fn withdraw_item_open(ctx: Context<WithdrawItemOpen>) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;
        let auction_ata = &ctx.accounts.auction_ata;
        let winner = &ctx.accounts.highest_bidder;
        let winner_ata = &ctx.accounts.highest_bidder_ata;
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let ata_program = &ctx.accounts.ata_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.end_time, Err(AuctionError::AuctionNotOver.into()));

        let amount = auction.token_amount;

        if winner_ata.to_account_info().data_is_empty() {
            create_ata(
                winner.to_account_info(),
                winner.to_account_info(),
                mint.to_account_info(),
                winner_ata.to_account_info(),
                token_program.to_account_info(),
                ata_program.to_account_info(),
                system_program.to_account_info(),
                rent_sysvar.to_account_info()
            )?;
        }

        transfer_spl(
            auction.to_account_info(),
            auction_ata.to_account_info(),
            winner_ata.to_account_info(),
            amount,
            token_program.to_account_info(),
            &[&[b"open auction", auction.owner.as_ref(), name_seed(&auction.title), &[auction.bump]]]
        )?;

        Ok(())
    }

    pub fn withdraw_winning_bid_open(ctx: Context<WithdrawWinningBidOpen>) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;
        let owner: &Signer = &ctx.accounts.owner;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.end_time, Err(AuctionError::AuctionNotOver.into()));

        let index = auction.bidders.iter().position(|&x| x == auction.highest_bidder);
        if let None = index {
            return Err(AuctionError::NoWinningBid.into())
        } else {
            let winning_bid = auction.bids[index.unwrap()];

            require!(winning_bid > 0, Err(AuctionError::AlreadyWithdrewBid.into()));

            auction.bids[index.unwrap()] = 0;

            let src = &mut auction.to_account_info();
            let dst = &mut owner.to_account_info();

            transfer_from_owned_account(src, dst, winning_bid)?;
        }

        Ok(())
    }

    pub fn reclaim_item_open(ctx: Context<ReclaimItemOpen>) -> ProgramResult {
        let auction: &mut Account<OpenAuction> = &mut ctx.accounts.auction;
        let auction_ata = &ctx.accounts.auction_ata;
        let owner = &ctx.accounts.owner;
        let owner_ata = &ctx.accounts.owner_ata;
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let ata_program = &ctx.accounts.ata_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(
            (auction.highest_bid == 0 && cur_time > auction.end_time) || auction.cancelled,
            Err(AuctionError::AuctionNotOver.into())
        );

        let amount = auction.token_amount;

        if owner_ata.to_account_info().data_is_empty() {
            create_ata(
                owner.to_account_info(),
                owner.to_account_info(),
                mint.to_account_info(),
                owner_ata.to_account_info(),
                token_program.to_account_info(),
                ata_program.to_account_info(),
                system_program.to_account_info(),
                rent_sysvar.to_account_info()
            )?;
        }

        transfer_spl(
            auction.to_account_info(),
            auction_ata.to_account_info(),
            owner_ata.to_account_info(),
            amount,
            token_program.to_account_info(),
            &[&[b"open auction", auction.owner.as_ref(), name_seed(&auction.title), &[auction.bump]]]
        )?;

        Ok(())
    }

    pub fn create_sealed_auction(
        ctx: Context<CreateSealedAuction>,
        bump: u8,
        title: String,
        floor: u64,
        first_price: bool,
        start_time: u64,
        end_time: u64,
        reveal_period: u64,
        bidder_cap: u64,
        token_amount: u64,
    ) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let auction_ata = &ctx.accounts.auction_ata;
        let owner = &ctx.accounts.owner;
        let owner_ata = &ctx.accounts.owner_ata;
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let ata_program = &ctx.accounts.ata_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(title.chars().count() <= 50, Err(AuctionError::TitleOverflow.into()));
        require!(token_amount != 0, Err(AuctionError::InvalidTokenAmount.into()));
        require!(start_time < end_time, Err(AuctionError::InvalidStartTime.into()));
        require!(cur_time > start_time || start_time == 0, Err(AuctionError::InvalidStartTime.into()));
        require!(cur_time < end_time, Err(AuctionError::InvalidEndTime.into()));
        require!(reveal_period > end_time, Err(AuctionError::InvalidRevealPeriod.into()));
        require!(floor > 0, Err(AuctionError::InvalidBidFloor.into()));

        auction.first_price = first_price;

        auction.owner = *owner.key;
        auction.mint = mint.key();
        auction.token_amount = token_amount;

        auction.start_time = if start_time == 0 { cur_time } else { start_time };
        auction.end_time = end_time;
        auction.reveal_period = reveal_period;
        auction.cancelled = false;

        auction.title = title;

        auction.bidder_cap = bidder_cap;
        auction.highest_bid = 0;
        auction.second_highest_bid = 0;
        auction.bid_floor = floor;
        auction.winning_bid_withdrawn = false;

        auction.bump = bump;

        create_ata(
            owner.to_account_info(),
            auction.to_account_info(),
            mint.to_account_info(),
            auction_ata.to_account_info(),
            token_program.to_account_info(),
            ata_program.to_account_info(),
            system_program.to_account_info(),
            rent_sysvar.to_account_info()
        )?;

        transfer_spl(
            owner.to_account_info(),
            owner_ata.to_account_info(),
            auction_ata.to_account_info(),
            token_amount,
            token_program.to_account_info(),
            &[]
        )?;

        Ok(())
    }

    pub fn cancel_sealed_auction(ctx: Context<CancelSealedAuction>) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(cur_time < auction.end_time, Err(AuctionError::CannotCancelRevealPeriod.into()));

        auction.cancelled = true;

        Ok(())
    }

    pub fn make_sealed_bid(ctx: Context<MakeSealedBid>, bid_hash: [u8; 32], amount: u64) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let bidder: &Signer = &ctx.accounts.bidder;
        let system_program = &ctx.accounts.system_program;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.start_time, Err(AuctionError::BidBeforeStart.into()));
        require!(cur_time < auction.end_time, Err(AuctionError::BidAfterClose.into()));
        require!(*bidder.key != auction.owner, Err(AuctionError::OwnerCannotBid.into()));
        require!(amount > 0, Err(AuctionError::MustSendSol.into()));

        let index = auction.bidders.iter().position(|&x| x == *bidder.key);

        if let None = index {
            require!(
                auction.bidders.len() < (auction.bidder_cap as usize),
                Err(AuctionError::BidderCapReached.into())
            );
            auction.bidders.push(*bidder.key);
            auction.sealed_bids.push(bid_hash);
            auction.fake_bids.push(amount);
        } else {
            return Err(AuctionError::DuplicateSealedBid.into());
        }

        transfer_sol(
            bidder.to_account_info(),
            auction.to_account_info(),
            amount,
            system_program.to_account_info()
        )?;

        Ok(())
    }

    pub fn reclaim_sealed_bid(ctx: Context<ReclaimSealedBid>) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let bidder: &Signer = &ctx.accounts.bidder;

        let index = auction.bidders.iter().position(|&x| x == *bidder.key);

        if let None = index {
            return Err(AuctionError::NotBidder.into())
        } else {
            let fake_bid = auction.fake_bids[index.unwrap()];

            auction.bidders.remove(index.unwrap());
            auction.sealed_bids.remove(index.unwrap());
            auction.fake_bids.remove(index.unwrap());

            let src = &mut auction.to_account_info();
            let dst = &mut bidder.to_account_info();

            transfer_from_owned_account(src, dst, fake_bid)?;
        }

        Ok(())
    }

    pub fn reveal_sealed_bid(ctx: Context<RevealSealedBid>, bid: u64, nonce: u64) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let bidder: &Signer = &ctx.accounts.bidder;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.end_time, Err(AuctionError::AuctionNotOver.into()));
        require!(cur_time < auction.reveal_period, Err(AuctionError::RevealPeriodOver.into()));

        let index = auction.bidders.iter().position(|&x| x == *bidder.key);

        if let None = index {
            return Err(AuctionError::NotBidder.into())
        } else {
            let fake_bid = auction.fake_bids[index.unwrap()];

            let bid_hash = auction.sealed_bids[index.unwrap()];
            let proposed_hash = compute_bid_hash(bid, nonce);

            require!(bid_hash == proposed_hash, Err(AuctionError::HashMismatch.into()));
            require!(bid > auction.bid_floor, Err(AuctionError::UnderBidFloor.into()));
            require!(fake_bid >= bid, Err(AuctionError::InsufficientSol.into()));

            if bid > auction.highest_bid {
                auction.second_highest_bid = auction.highest_bid;
                auction.highest_bidder = *bidder.key;
                auction.highest_bid = bid;
            } else {
                auction.bidders.remove(index.unwrap());
                auction.sealed_bids.remove(index.unwrap());
                auction.fake_bids.remove(index.unwrap());

                let src = &mut auction.to_account_info();
                let dst = &mut bidder.to_account_info();

                transfer_from_owned_account(src, dst, fake_bid)?;
            }
        }

        Ok(())
    }

    pub fn withdraw_item_sealed(ctx: Context<WithdrawItemSealed>) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let auction_ata = &ctx.accounts.auction_ata;
        let winner = &ctx.accounts.highest_bidder;
        let winner_ata = &ctx.accounts.highest_bidder_ata;
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let ata_program = &ctx.accounts.ata_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.reveal_period, Err(AuctionError::RevealPeriodNotOver.into()));

        let amount = auction.token_amount;

        if winner_ata.to_account_info().data_is_empty() {
            create_ata(
                winner.to_account_info(),
                winner.to_account_info(),
                mint.to_account_info(),
                winner_ata.to_account_info(),
                token_program.to_account_info(),
                ata_program.to_account_info(),
                system_program.to_account_info(),
                rent_sysvar.to_account_info()
            )?;
        }

        transfer_spl(
            auction.to_account_info(),
            auction_ata.to_account_info(),
            winner_ata.to_account_info(),
            amount,
            token_program.to_account_info(),
            &[&[b"sealed auction", auction.owner.as_ref(), name_seed(&auction.title), &[auction.bump]]]
        )?;

        // refund difference between sent SOL and real bid SOL
        // must happen last bc solana does a pre-cpi account balance check
        // so we can't transfer sol before create_ata or solana will think smth is wrong
        let index = auction.bidders.iter().position(|&x| x == *winner.key);
        let fake_bid = auction.fake_bids[index.unwrap()];
        let bid_delta;
        if auction.first_price || auction.second_highest_bid == 0 {
            bid_delta = fake_bid - auction.highest_bid;
        } else {
            bid_delta = fake_bid - auction.second_highest_bid;
        }

        let src = &mut auction.to_account_info();
        let dst = &mut winner.to_account_info();
        transfer_from_owned_account(src, dst, bid_delta)?;

        Ok(())
    }

    pub fn withdraw_winning_bid_sealed(ctx: Context<WithdrawWinningBidSealed>) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let owner: &Signer = &ctx.accounts.owner;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(!auction.cancelled, Err(AuctionError::AuctionCancelled.into()));
        require!(cur_time > auction.end_time, Err(AuctionError::AuctionNotOver.into()));
        require!(cur_time > auction.reveal_period,Err(AuctionError::RevealPeriodNotOver.into()));

        let index = auction.bidders.iter().position(|&x| x == auction.highest_bidder);
        if let None = index {
            return Err(AuctionError::NoWinningBid.into())
        } else {
            let amount;
            if auction.first_price || auction.second_highest_bid == 0 {
                amount = auction.highest_bid;
            } else {
                amount = auction.second_highest_bid;
            }

            require!(
                !auction.winning_bid_withdrawn,
                Err(AuctionError::AlreadyWithdrewBid.into())
            );

            auction.winning_bid_withdrawn = true;

            let src = &mut auction.to_account_info();
            let dst = &mut owner.to_account_info();

            transfer_from_owned_account(src, dst, amount)?;
        }

        Ok(())
    }

    pub fn reclaim_item_sealed(ctx: Context<ReclaimItemSealed>) -> ProgramResult {
        let auction: &mut Account<SealedAuction> = &mut ctx.accounts.auction;
        let auction_ata = &ctx.accounts.auction_ata;
        let owner = &ctx.accounts.owner;
        let owner_ata = &ctx.accounts.owner_ata;
        let mint = &ctx.accounts.mint;
        let token_program = &ctx.accounts.token_program;
        let ata_program = &ctx.accounts.ata_program;
        let system_program = &ctx.accounts.system_program;
        let rent_sysvar = &ctx.accounts.rent_sysvar;

        let clock: Clock = Clock::get().unwrap();
        let cur_time: u64 = clock.unix_timestamp as u64;

        require!(
            (auction.highest_bid == 0 && cur_time > auction.reveal_period) || auction.cancelled,
            Err(AuctionError::RevealPeriodNotOver.into())
        );

        let amount = auction.token_amount;

        if owner_ata.to_account_info().data_is_empty() {
            create_ata(
                owner.to_account_info(),
                owner.to_account_info(),
                mint.to_account_info(),
                owner_ata.to_account_info(),
                token_program.to_account_info(),
                ata_program.to_account_info(),
                system_program.to_account_info(),
                rent_sysvar.to_account_info()
            )?;
        }

        transfer_spl(
            auction.to_account_info(),
            auction_ata.to_account_info(),
            owner_ata.to_account_info(),
            amount,
            token_program.to_account_info(),
            &[&[b"sealed auction", auction.owner.as_ref(), name_seed(&auction.title), &[auction.bump]]]
        )?;

        Ok(())
    }
}