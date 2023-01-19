import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Bonkinator } from "../target/types/bonkinator";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, SystemProgram, SYSVAR_RENT_PUBKEY, AccountMeta } from "@solana/web3.js";
import { createMint, getAccount, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import assert from 'assert'

describe("bonkinator", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Bonkinator as Program<Bonkinator>;

  it("works as intended", async () => {
    //Test doesn't work with bonk mint restraints on
    const firstBuyer = Keypair.generate();
    const secondBuyer = Keypair.generate();
    const tx1 = await anchor
      .getProvider()
      .connection.requestAirdrop(
        firstBuyer.publicKey,
        LAMPORTS_PER_SOL
      );
    await anchor.getProvider().connection.confirmTransaction(tx1);
    const tx = await anchor
      .getProvider()
      .connection.requestAirdrop(
        secondBuyer.publicKey,
        LAMPORTS_PER_SOL
      );
    await anchor.getProvider().connection.confirmTransaction(tx);

    const bonkMint = await createMint(
      anchor.getProvider().connection,
      secondBuyer,
      secondBuyer.publicKey,
      secondBuyer.publicKey,
      5
    );

    const firstBuyerTA = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      secondBuyer,
      bonkMint,
      firstBuyer.publicKey
    );

    const secondBuyerTA = await getOrCreateAssociatedTokenAccount(
      anchor.getProvider().connection,
      secondBuyer,
      bonkMint,
      secondBuyer.publicKey
    );

    await mintTo(
      anchor.getProvider().connection,
      secondBuyer,
      bonkMint,
      firstBuyerTA.address,
      secondBuyer.publicKey,
      10000000000000
    );

    await mintTo(
      anchor.getProvider().connection,
      secondBuyer,
      bonkMint,
      secondBuyerTA.address,
      secondBuyer.publicKey,
      10000000000000
    );

    console.log("First buyer TA: ", (await getAccount(
      anchor.getProvider().connection,
      firstBuyerTA.address,
    )).amount);

    console.log("Second buyer TA: ", (await getAccount(
      anchor.getProvider().connection,
      secondBuyerTA.address,
    )).amount);

    const [treasury] = await PublicKey.findProgramAddress(
      [Buffer.from("treasury"), bonkMint.toBuffer()],
      program.programId
    );

    //Treasury needs to be initialized only once when the program deploys
    await program.methods.createBonkTokenAccount()
      .accounts({
        payer: firstBuyer.publicKey,
        bonkMint: bonkMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
        treasury,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([firstBuyer])
      .rpc();

    console.log("\nTreasury initialized");
    console.log("Treasury bonk balance: ", (await getAccount(
      anchor.getProvider().connection,
      treasury
    )).amount);

    //Tweet is found and if it's pda account doesn't exist it will be created on first buy
    const [tweet1] = await PublicKey.findProgramAddress(
      [Buffer.from("tweet"), Buffer.from("123")],
      program.programId
    );

    //First buyer buys unclaimed tweet
    await program.methods.buyTweet("123")
      .accounts({
        buyer: firstBuyer.publicKey,
        bonkMint: bonkMint,
        buyerBonkAcc: firstBuyerTA.address,
        systemProgram: SystemProgram.programId,
        tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
        treasury,
        tweet: tweet1,
      })
      .remainingAccounts([])
      .signers(
        [firstBuyer]
      )
      .rpc();

    //Buyer paid 1M Bonk
    console.log("\nBuyer's token account balance: ", (await getAccount(
      anchor.getProvider().connection,
      firstBuyerTA.address
    )).amount);

    //Tweet pda is updated
    let tweetAcc = await program.account.tweet.fetch(tweet1);
    console.log("\nTweet:");
    console.log("Tweet id: ", tweetAcc.tweetId);
    console.log("Tweet owner: ", tweetAcc.owner.toBase58());
    console.log("Tweet price: ", tweetAcc.price.toNumber());

    //Treasury received all the money from initial purchase
    console.log("\nTreasury bonk balance after first buy: ", (await getAccount(
      anchor.getProvider().connection,
      treasury
    )).amount);

    //If an owned tweet is to be bought, owners bonk token account needs to be passed as a remaining account
    let remainingAccounts1: AccountMeta[] = [
      {
        pubkey: firstBuyerTA.address,
        isSigner: false,
        isWritable: true
      }
    ]

    //Second buyer buys the first tweet
    await program.methods.buyTweet("123")
      .accounts({
        buyer: secondBuyer.publicKey,
        bonkMint: bonkMint,
        buyerBonkAcc: secondBuyerTA.address,
        systemProgram: SystemProgram.programId,
        tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
        treasury,
        tweet: tweet1,
      })
      .remainingAccounts(remainingAccounts1)
      .signers(
        [secondBuyer]
      )
      .rpc();

    //Tweet PDA is updated
    tweetAcc = await program.account.tweet.fetch(tweet1);
    console.log("\nTweet:");
    console.log("Tweet id: ", tweetAcc.tweetId);
    console.log("Tweet owner: ", tweetAcc.owner.toBase58());
    console.log("Tweet price: ", tweetAcc.price.toNumber());

    //Treasury gets 10% of the price
    console.log("\nTreasury bonk balance after second buy: ", (await getAccount(
      anchor.getProvider().connection,
      treasury
    )).amount);

    //Owner gets 110% of what he bought the tweet for
    console.log("\nPrevious owner's bonk TA: ", (await getAccount(
      anchor.getProvider().connection,
      firstBuyerTA.address
    )).amount);

    //Buyer pays 20% more than the current tweet price
    console.log("Second buyer's bonk TA: ", (await getAccount(
      anchor.getProvider().connection,
      secondBuyerTA.address
    )).amount);

    //New tweet, different tweet_id
    const [tweet2] = await PublicKey.findProgramAddress(
      [Buffer.from("tweet"), Buffer.from("321")],
      program.programId
    );

    //First purchase of the new tweet
    await program.methods.buyTweet("321")
      .accounts({
        buyer: firstBuyer.publicKey,
        bonkMint: bonkMint,
        buyerBonkAcc: firstBuyerTA.address,
        systemProgram: SystemProgram.programId,
        tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
        treasury,
        tweet: tweet2,
      })
      .remainingAccounts([])
      .signers(
        [firstBuyer]
      )
      .rpc();

    //Tweet pda is updated
    let tweetAcc2 = await program.account.tweet.fetch(tweet2);
    console.log("\nTweet2:");
    console.log("Tweet id: ", tweetAcc2.tweetId);
    console.log("Tweet owner: ", tweetAcc2.owner.toBase58());
    console.log("Tweet price: ", tweetAcc2.price.toNumber());
    console.log("\nTreasury bonk balance after second buy: ", (await getAccount(
      anchor.getProvider().connection,
      treasury
    )).amount);

    try {
      //Try to buy your own tweet
      await program.methods.buyTweet("321")
        .accounts({
          buyer: firstBuyer.publicKey,
          bonkMint: bonkMint,
          buyerBonkAcc: firstBuyerTA.address,
          systemProgram: SystemProgram.programId,
          tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
          treasury,
          tweet: tweet2,
        })
        .remainingAccounts(remainingAccounts1)
        .signers(
          [firstBuyer]
        )
        .rpc();

    } catch (error) {
      assert.equal(
        error.error.errorCode.code,
        "AlreadyOwner",
        error.error.errorMessage
      );
    }
  });
});
