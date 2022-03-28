use anchor_lang::prelude::*;

#[error]
pub enum AuctionError {
    #[msg("Title must be less than 50 characters.")]
    TitleOverflow,
    #[msg("Minimum bid increment must be greater than 0.")]
    InvalidIncrement,
    #[msg("Start time must be in the future and before end time.")]
    InvalidStartTime,
    #[msg("End time must be after start time.")]
    InvalidEndTime,
    #[msg("Bid floor must be at least 1 lamport.")]
    InvalidBidFloor,
    #[msg("Reveal period must end after the auction ends.")]
    InvalidRevealPeriod,
    #[msg("SPL token amount must be greater than 0.")]
    InvalidTokenAmount,
    #[msg("Must bid higher than the floor.")]
    UnderBidFloor,
    #[msg("Must bid at least min_bid_increment higher than max_bid.")]
    InsufficientBid,
    #[msg("Auction is cancelled and only allows reclaiming past bids and the item.")]
    AuctionCancelled,
    #[msg("Auction period has not yet begun.")]
    BidBeforeStart,
    #[msg("Auction period has elapsed.")]
    BidAfterClose,
    #[msg("Maximum number of unique bidders has been reached.")]
    BidderCapReached,
    #[msg("Owner cannot bid on auction.")]
    OwnerCannotBid,
    #[msg("Auction is not over.")]
    AuctionNotOver,
    #[msg("No previous bid associated with this key.")]
    NotBidder,
    #[msg("No winning bid to withdraw.")]
    NoWinningBid,
    #[msg("Auction winner cannot withdraw their bid.")]
    WinnerCannotWithdrawBid,
    #[msg("Winning bid has already been withdrawn.")]
    AlreadyWithdrewBid,
    #[msg("Each key can only have one active sealed bid per auction.")]
    DuplicateSealedBid,
    #[msg("Sealed bids must be accompanied by a non-zero amount of SOL.")]
    MustSendSol,
    #[msg("Reveal period has elapsed.")]
    RevealPeriodOver,
    #[msg("Reveal period is not over.")]
    RevealPeriodNotOver,
    #[msg("Keccak256 of provided bid and nonce does not match the sealed bid hash.")]
    HashMismatch,
    #[msg("Cannot cancel auction during reveal period.")]
    CannotCancelRevealPeriod,
    #[msg("Cannot cancel auction after it has ended.")]
    CannotCancelAfterClose,
    #[msg("Sealed bid cannot be higher than escrowed SOL.")]
    InsufficientSol,
}