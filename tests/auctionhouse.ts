import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import * as splToken from '@solana/spl-token';
import * as serumAta from '@project-serum/associated-token'
import * as web3 from '@solana/web3.js';
import { Auctionhouse } from '../target/types/auctionhouse';
import * as assert from "assert";
import { keccak_256 } from 'js-sha3';

function lamports(sol: number): number{
  return sol*anchor.web3.LAMPORTS_PER_SOL;
}

function sol(lamports: number): number{
  return lamports/anchor.web3.LAMPORTS_PER_SOL;
}

function randomInt(min: number, max: number): number {
  min = Math.ceil(min);
  max = Math.floor(max);
  return Math.floor(Math.random() * (max - min) + min);
}

function delay(interval: number, message: string): Mocha.Test {
   return it(message, done => {
      setTimeout(() => done(), interval)
   }).timeout(interval + 100)
}

async function airdrop(program, address: web3.PublicKey, lamports: number){
  const air = await program.provider.connection.requestAirdrop(address, lamports);
  await program.provider.connection.confirmTransaction(air);
}

async function getLamportBalance(program, address: web3.PublicKey): Promise<number> {
  let amt = await program.provider.connection.getBalance(address);
  return amt;
}

async function getTokenBalance(program, tokenAccountAddress: web3.PublicKey): Promise<{
  amount: string,
  decimals: number,
  uiAmount: number,
  uiAmountString: number
}> {
  let res = await program.provider.connection.getTokenAccountBalance(tokenAccountAddress);
  return res.value;
}

async function deriveOpenAuction(program,
  ownerAddress: web3.PublicKey,
  mintAddress: web3.PublicKey,
  auctionTitle: string
): Promise<[auctionAddress: web3.PublicKey, bump: number, auctionAta: web3.PublicKey]> {
  const [auctionAddress, bump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("open auction"), ownerAddress.toBytes(), Buffer.from(auctionTitle.slice(0, 32))],
    program.programId
  )
  let auctionAta = await serumAta.getAssociatedTokenAddress(auctionAddress, mintAddress);
  return [auctionAddress, bump, auctionAta];
}

async function deriveSealedAuction(program,
  ownerAddress: web3.PublicKey,
  mintAddress: web3.PublicKey,
  auctionTitle: string
): Promise<[auctionAddress: web3.PublicKey, bump: number, auctionAta: web3.PublicKey]> {
  const [auctionAddress, bump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("sealed auction"), ownerAddress.toBytes(), Buffer.from(auctionTitle.slice(0, 32))],
    program.programId
  )
  let auctionAta = await serumAta.getAssociatedTokenAddress(auctionAddress, mintAddress);
  return [auctionAddress, bump, auctionAta];
}

// Configure the client to use the local cluster.
anchor.setProvider(anchor.Provider.env());

// @ts-ignore
const program = anchor.workspace.Auctionhouse as Program<Auctionhouse>;

let auctionEndDelay = 7000;
let revealPeriodEndDelay = 7000;

describe('open auction', () => {

  let seller;
  let loser;
  let buyer;
  let mintOwner;
  let decimals;
  let mintAmount;
  let mint;
  let sellerAta;
  let buyerAtaAddress;
  let auctionAddress;
  let bump;
  let auctionAta;
  let auctionAccount;
  let losingBid;
  let winningBid;
  let initialAuctionBalance;

  let amt;

  it('init auction', async () => {
    seller = anchor.web3.Keypair.generate();
    loser = anchor.web3.Keypair.generate();
    buyer = anchor.web3.Keypair.generate();
    mintOwner = anchor.web3.Keypair.generate();

    await airdrop(program, seller.publicKey, lamports(5));
    await airdrop(program, loser.publicKey, lamports(5));
    await airdrop(program, buyer.publicKey, lamports(5));
    await airdrop(program, mintOwner.publicKey, lamports(5));

    decimals = 9;
    mintAmount = Math.pow(10, decimals);

    mint = await splToken.Token.createMint(
      program.provider.connection,
      mintOwner,
      mintOwner.publicKey,
      null,
      decimals,
      splToken.TOKEN_PROGRAM_ID,
    );

    sellerAta = await mint.getOrCreateAssociatedAccountInfo(seller.publicKey);
    // dont create the ata now so that the contract will do it in withdraw_item
    buyerAtaAddress = await serumAta.getAssociatedTokenAddress(buyer.publicKey, mint.publicKey);

    amt = await getTokenBalance(program, sellerAta.address);
    assert.equal(amt.amount, 0);

    await mint.mintTo(sellerAta.address, mintOwner.publicKey, [], mintAmount);

    amt = await getTokenBalance(program, sellerAta.address);
    assert.equal(amt.amount, mintAmount);

    let auctionTitle = "spl bidding test";
    let floor = lamports(0.1);
    let increment = lamports(0.05);
    let biddercap = 2;
    let startTime = Math.floor(Date.now() / 1000) - 60;
    let endTime = Math.floor(Date.now() / 1000) + 5;
    let amount = mintAmount;

    [auctionAddress, bump, auctionAta] = await deriveOpenAuction(program, seller.publicKey, mint.publicKey, auctionTitle);

    await program.rpc.createOpenAuction(new anchor.BN(bump),
                                    auctionTitle,
                                    new anchor.BN(floor),
                                    new anchor.BN(increment),
                                    new anchor.BN(startTime),
                                    new anchor.BN(endTime),
                                    new anchor.BN(biddercap),
                                    new anchor.BN(amount), {
        accounts: {
          auction: auctionAddress,
          auctionAta: auctionAta,
          owner: seller.publicKey,
          ownerAta: sellerAta.address,
          mint: mint.publicKey,
          tokenProgram: splToken.TOKEN_PROGRAM_ID,
          ataProgram: serumAta.ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rentSysvar: web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [seller],
    });

    initialAuctionBalance = await getLamportBalance(program, auctionAddress);

    amt = await getTokenBalance(program, sellerAta.address);
    assert.equal(amt.amount, 0);

    amt = await getTokenBalance(program, auctionAta);
    assert.equal(amt.amount, mintAmount);

    // amt = await getLamportBalance(program, seller.publicKey);
    // console.log(sol(lamports(5) - amt));
  });

  it('make losing bid', async () => {
    losingBid = lamports(1);
    let initialBalance = await getLamportBalance(program, auctionAddress);
    let loserBalance = await getLamportBalance(program, loser.publicKey);

    await program.rpc.makeOpenBid(new anchor.BN(losingBid), {
      accounts: {
        auction: auctionAddress,
        bidder: loser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [loser]
    });

    auctionAccount = await program.account.openAuction.fetch(auctionAddress);
    assert.equal(auctionAccount.highestBidder.toBase58(), loser.publicKey.toBase58());
    assert.equal(auctionAccount.highestBid, losingBid);

    amt = await getLamportBalance(program, auctionAddress);
    assert.equal(amt - initialBalance, losingBid);
    amt = await getLamportBalance(program, loser.publicKey);
    assert.equal(loserBalance - amt, losingBid);
  });

  it('make winning bid', async () => {
    winningBid = lamports(2);
    let initialBalance = await getLamportBalance(program, auctionAddress);
    let winnerBalance = await getLamportBalance(program, buyer.publicKey);

    await program.rpc.makeOpenBid(new anchor.BN(winningBid), {
      accounts: {
        auction: auctionAddress,
        bidder: buyer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [buyer]
    });

    auctionAccount = await program.account.openAuction.fetch(auctionAddress);
    assert.equal(auctionAccount.highestBidder.toBase58(), buyer.publicKey.toBase58());
    assert.equal(auctionAccount.highestBid, winningBid);

    amt = await getLamportBalance(program, auctionAddress);
    assert.equal(amt - initialBalance, winningBid);
    amt = await getLamportBalance(program, buyer.publicKey);
    assert.equal(winnerBalance - amt, winningBid);
  });

  it('reclaim losing bid', async () => {
    let initialBalance = await getLamportBalance(program, loser.publicKey);

    await program.rpc.reclaimOpenBid({
      accounts: {
        auction: auctionAddress,
        bidder: loser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [loser]
    });

    amt = await getLamportBalance(program, loser.publicKey);
    assert.equal(amt - initialBalance, losingBid);
  });

  xit('cancel auction', async () => {
    await program.rpc.cancelOpenAuction({
      accounts: {
        auction: auctionAddress,
        owner: seller.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [seller]
    });

    auctionAccount = await program.account.openAuction.fetch(auctionAddress);
    assert.equal(auctionAccount.cancelled, true);
  });

  delay(auctionEndDelay, "delay for auction period to end");

  it('withdraw winning bid', async () => {
    let initialBalance = await getLamportBalance(program, seller.publicKey);

    await program.rpc.withdrawWinningBidOpen({
      accounts: {
        auction: auctionAddress,
        owner: seller.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [seller]
    });

    amt = await getLamportBalance(program, seller.publicKey);
    assert.equal(amt - initialBalance, winningBid);
  });

  it('withdraw winner spl tokens', async () => {
    await program.rpc.withdrawItemOpen({
      accounts: {
        auction: auctionAddress,
        auctionAta: auctionAta,
        highestBidder: buyer.publicKey,
        highestBidderAta: buyerAtaAddress,
        mint: mint.publicKey,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
        ataProgram: serumAta.ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rentSysvar: web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [buyer]
    });

    amt = await getTokenBalance(program, buyerAtaAddress);
    assert.equal(amt.amount, mintAmount);

    // make sure auction hasn't kept any sol
    amt = await getLamportBalance(program, auctionAddress);
    assert.equal(amt, initialAuctionBalance);
  });

  it('fetch auction', async () => {
    const auctionAccounts = await program.account.openAuction.all();
    assert.equal(auctionAccounts.length, 1);
  });

});

describe('sealed auction', () => {

  let seller;
  let loser;
  let buyer;
  let mintOwner;
  let decimals;
  let mintAmount;
  let mint;
  let sellerAta;
  let buyerAtaAddress;
  let auctionAddress;
  let bump;
  let auctionAta;
  let auctionAccount;
  let losingBid;
  let losingBidNonce;
  let losingBidHash;
  let fakeLosingBid;
  let winningBid;
  let winningBidNonce;
  let winningBidHash;
  let fakeWinningBid;
  let initialAuctionBalance;

  let amt;

  it('init auction', async () => {
    seller = anchor.web3.Keypair.generate();
    loser = anchor.web3.Keypair.generate();
    buyer = anchor.web3.Keypair.generate();
    mintOwner = anchor.web3.Keypair.generate();

    await airdrop(program, seller.publicKey, lamports(10));
    await airdrop(program, loser.publicKey, lamports(10));
    await airdrop(program, buyer.publicKey, lamports(10));
    await airdrop(program, mintOwner.publicKey, lamports(10));

    decimals = 9;
    mintAmount = Math.pow(10, decimals);

    mint = await splToken.Token.createMint(
      program.provider.connection,
      mintOwner,
      mintOwner.publicKey,
      null,
      decimals,
      splToken.TOKEN_PROGRAM_ID,
    );

    sellerAta = await mint.getOrCreateAssociatedAccountInfo(seller.publicKey);
    // dont create the ata now so that the contract will do it in withdraw_item
    buyerAtaAddress = await serumAta.getAssociatedTokenAddress(buyer.publicKey, mint.publicKey);

    amt = await getTokenBalance(program, sellerAta.address);
    assert.equal(amt.amount, 0);

    await mint.mintTo(sellerAta.address, mintOwner.publicKey, [], mintAmount);

    amt = await getTokenBalance(program, sellerAta.address);
    assert.equal(amt.amount, mintAmount);

    let auctionTitle = "spl bidding test";
    let floor = lamports(0.1);
    let firstPrice = true;
    let biddercap = 2;
    let startTime = Math.floor(Date.now() / 1000) - 60;
    let endTime = Math.floor(Date.now() / 1000) + 5;
    let revealTime = Math.floor(Date.now() / 1000) + 10;
    let amount = mintAmount;

    [auctionAddress, bump, auctionAta] = await deriveSealedAuction(program, seller.publicKey, mint.publicKey, auctionTitle);

    await program.rpc.createSealedAuction(new anchor.BN(bump),
                                    auctionTitle,
                                    new anchor.BN(floor),
                                    firstPrice,
                                    new anchor.BN(startTime),
                                    new anchor.BN(endTime),
                                    new anchor.BN(revealTime),
                                    new anchor.BN(biddercap),
                                    new anchor.BN(amount), {
        accounts: {
          auction: auctionAddress,
          auctionAta: auctionAta,
          owner: seller.publicKey,
          ownerAta: sellerAta.address,
          mint: mint.publicKey,
          tokenProgram: splToken.TOKEN_PROGRAM_ID,
          ataProgram: serumAta.ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rentSysvar: web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [seller],
    });

    initialAuctionBalance = await getLamportBalance(program, auctionAddress);

    amt = await getTokenBalance(program, sellerAta.address);
    assert.equal(amt.amount, 0);

    amt = await getTokenBalance(program, auctionAta);
    assert.equal(amt.amount, mintAmount);

    // amt = await getLamportBalance(program, seller.publicKey);
    // console.log(sol(lamports(5) - amt));
  });

  it('make losing bid', async () => {
    let initialBalance = await getLamportBalance(program, auctionAddress);
    let loserBalance = await getLamportBalance(program, loser.publicKey);

    fakeLosingBid = lamports(1.5);
    losingBid = lamports(1);
    losingBidNonce = randomInt(100000, 1000000);

    let hash = keccak_256.create();
    hash.update(losingBid.toString());
    hash.update(losingBidNonce.toString());
    losingBidHash = Uint8Array.from(Buffer.from(hash.hex(), 'hex'));

    await program.rpc.makeSealedBid(losingBidHash, new anchor.BN(fakeLosingBid), {
      accounts: {
        auction: auctionAddress,
        bidder: loser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [loser]
    });

    // auctionAccount = await program.account.sealedAuction.fetch(auctionAddress);
    // console.log(auctionAccount);

    amt = await getLamportBalance(program, auctionAddress);
    assert.equal(amt - initialBalance, fakeLosingBid);
    amt = await getLamportBalance(program, loser.publicKey);
    assert.equal(loserBalance - amt, fakeLosingBid);
  });

  it('make winning bid', async () => {
    let initialBalance = await getLamportBalance(program, auctionAddress);
    let winnerBalance = await getLamportBalance(program, buyer.publicKey);

    fakeWinningBid = lamports(2.5);
    winningBid = lamports(2);
    winningBidNonce = randomInt(100000, 1000000);

    let hash = keccak_256.create();
    hash.update(winningBid.toString());
    hash.update(winningBidNonce.toString());
    winningBidHash = Uint8Array.from(Buffer.from(hash.hex(), 'hex'));

    await program.rpc.makeSealedBid(winningBidHash, new anchor.BN(fakeWinningBid), {
      accounts: {
        auction: auctionAddress,
        bidder: buyer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [buyer]
    });

    // auctionAccount = await program.account.sealedAuction.fetch(auctionAddress);
    // console.log(auctionAccount);

    amt = await getLamportBalance(program, auctionAddress);
    assert.equal(amt - initialBalance, fakeWinningBid);
    amt = await getLamportBalance(program, buyer.publicKey);
    assert.equal(winnerBalance - amt, fakeWinningBid);
  });

  xit('cancel auction', async () => {
    await program.rpc.cancelSealedAuction({
      accounts: {
        auction: auctionAddress,
        owner: seller.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [seller]
    });

    auctionAccount = await program.account.sealedAuction.fetch(auctionAddress);
    assert.equal(auctionAccount.cancelled, true);
  });

  delay(auctionEndDelay, "delay for reveal period to start");

  it('reveal winning bid', async () => {
    await program.rpc.revealSealedBid(new anchor.BN(winningBid),
                                      new anchor.BN(winningBidNonce), {
      accounts: {
        auction: auctionAddress,
        bidder: buyer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [buyer]
    });

    auctionAccount = await program.account.sealedAuction.fetch(auctionAddress);

  });

  it('reveal/reclaim losing bid', async () => {
    let initialBalance = await getLamportBalance(program, loser.publicKey);

    await program.rpc.revealSealedBid(new anchor.BN(losingBid),
                                      new anchor.BN(losingBidNonce), {
      accounts: {
        auction: auctionAddress,
        bidder: loser.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [loser]
    });

    amt = await getLamportBalance(program, loser.publicKey);
    assert.equal(amt - initialBalance, fakeLosingBid);
  });

  delay(revealPeriodEndDelay, "delay for reveal period to end");

  it('withdraw winning bid', async () => {
    let initialBalance = await getLamportBalance(program, seller.publicKey);

    await program.rpc.withdrawWinningBidSealed({
      accounts: {
        auction: auctionAddress,
        owner: seller.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [seller]
    });

    amt = await getLamportBalance(program, seller.publicKey);
    assert.equal(amt - initialBalance, winningBid);
  });

  it('withdraw winner spl tokens', async () => {
    let initialBalance = await getLamportBalance(program, buyer.publicKey);

    await program.rpc.withdrawItemSealed({
      accounts: {
        auction: auctionAddress,
        auctionAta: auctionAta,
        highestBidder: buyer.publicKey,
        highestBidderAta: buyerAtaAddress,
        mint: mint.publicKey,
        tokenProgram: splToken.TOKEN_PROGRAM_ID,
        ataProgram: serumAta.ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rentSysvar: web3.SYSVAR_RENT_PUBKEY,
      },
      signers: [buyer]
    });

    amt = await getTokenBalance(program, buyerAtaAddress);
    assert.equal(amt.amount, mintAmount);

    amt = await getLamportBalance(program, buyer.publicKey);
    // buyer refunded delta between fake bid and real bid minus cost to create ATA
    assert.ok(amt - initialBalance > (fakeWinningBid - winningBid - 10000000));

    // make sure auction hasn't kept any sol
    amt = await getLamportBalance(program, auctionAddress);
    assert.equal(amt, initialAuctionBalance);
  });

  it('fetch auction', async () => {
    const auctionAccounts = await program.account.sealedAuction.all();
    assert.equal(auctionAccounts.length, 1);
  });
});