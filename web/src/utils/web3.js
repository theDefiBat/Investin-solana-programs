import { initializeAccount } from "@project-serum/serum/lib/token-instructions";
import { PublicKey, sendAndConfirmTransaction, SystemProgram, Transaction, sendTransaction, Account, TransactionInstruction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, connection, RENT_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID } from "./constants";

const GREETING_SIZE = 171;

export async function setWalletTransaction(
  instruction,
  publicKey
) {
  const transaction = new Transaction();
  transaction.add(instruction);
  transaction.feePayer = publicKey;
  let hash = await connection.getRecentBlockhash('finalized');
  console.log("blockhash", hash);
  transaction.recentBlockhash = hash.blockhash;
  return transaction;
}

export async function signAndSendTransaction(
  wallet,
  transaction
) {
  console.log(`wallet :::`, wallet)
  let signedTrans = await wallet.signTransaction(transaction);
  console.log("sign transaction");
  let signature = await connection.sendRawTransaction(signedTrans.serialize(),{
    skipPreflight: true,
    preflightCommitment: 'finalized'
  });
  console.log("send raw transaction");
  return signature;
}

export const createKeyIfNotExists = async (wallet, payerAccount, programId, seed, size) => {
  const greetedPubkey = await PublicKey.createWithSeed(
    wallet.publicKey,
    seed,
    programId,
  );

  console.log(`greetedPubkey :: `, greetedPubkey)

  // Check if the greeting account has already been created
  const greetedAccount = await connection.getAccountInfo(greetedPubkey);

  console.log(`greetedAccount ::: `, greetedAccount)
  if (greetedAccount === null) {
    console.log(
      'Creating account',
      greetedPubkey.toBase58(),
      'to say hello to',
    );
    const lamports = await connection.getMinimumBalanceForRentExemption(
      size,
    );

    // const transaction = new Transaction().add(

    // );
    console.log(`lamports :::: `, lamports)
    const transaction = await setWalletTransaction(
      SystemProgram.createAccountWithSeed({
        fromPubkey: wallet.publicKey,
        basePubkey: wallet.publicKey,
        seed: seed,
        newAccountPubkey: greetedPubkey,
        lamports,
        space: size,
        programId,
      }), wallet.publicKey)

    // await sendAndConfirmTransaction(connection, transaction, [payerAccount]);

    await signAndSendTransaction(wallet, transaction)
  }
  return greetedPubkey;
}

export async function createProgramAccountIfNotExist(
  connection,
  account,
  owner,
  programId,
  lamports,
  size,
  transaction,
  signer
) {
  let publicKey

  if (account) {
    publicKey = new PublicKey(account)
  } else {
    const newAccount = new Account()
    publicKey = newAccount.publicKey

    transaction.add(
      SystemProgram.createAccount({
        fromPubkey: owner,
        newAccountPubkey: publicKey,
        lamports: lamports ?? (await connection.getMinimumBalanceForRentExemption(size)),
        space: size,
        programId
      })
    )

    signer.push(newAccount)
  }

  return publicKey
}


export async function createTokenAccountIfNotExist(
  connection,
  account,
  owner,
  mintAddress,
  lamports,
  transaction,
  signer
) {
  let publicKey

  if (account) {
    publicKey = new PublicKey(account)
  } else {
    publicKey = await createProgramAccountIfNotExist(
      connection,
      account,
      owner,
      TOKEN_PROGRAM_ID,
      lamports,
      390,
      transaction,
      signer
    )

    transaction.add(
      initializeAccount({
        account: publicKey,
        mint: new PublicKey(mintAddress),
        owner
      })
    )
  }

  return publicKey
}

// associated address 


export async function findProgramAddress(seeds, programId) {
  const [publicKey, nonce] = await PublicKey.findProgramAddress(seeds, programId)
  return { publicKey, nonce }
}


export async function findAssociatedTokenAddress(walletAddress, tokenMintAddress) {
  const { publicKey } = await findProgramAddress(
    [walletAddress.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), tokenMintAddress.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )
  return publicKey
}



export async function createAssociatedTokenAccountIfNotExist(
  wallet,
  tokenMintAddress,
  owner,
) {
  const associatedTokenAddress = await findAssociatedTokenAddress(owner, tokenMintAddress)
  
  const tokenAccount = await connection.getAccountInfo(associatedTokenAddress);
  
  if (tokenAccount == null)
  {
    const keys = [
      {
        pubkey: wallet.publicKey,
        isSigner: true,
        isWritable: true
      },
      {
        pubkey: associatedTokenAddress,
        isSigner: false,
        isWritable: true
      },
      {
        pubkey: owner,
        isSigner: false,
        isWritable: false
      },
      {
        pubkey: tokenMintAddress,
        isSigner: false,
        isWritable: false
      },
      {
        pubkey: SYSTEM_PROGRAM_ID,
        isSigner: false,
        isWritable: false
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false
      },
      {
        pubkey: RENT_PROGRAM_ID,
        isSigner: false,
        isWritable: false
      }
    ]
    const transaction = await setWalletTransaction(
      new TransactionInstruction({
        keys,
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        data: Buffer.from([])
      }),
      wallet.publicKey)
    await signAndSendTransaction(wallet, transaction)
  }
  return associatedTokenAddress
}

export const commitment = 'confirmed'


export async function signTransaction(
  connection,
  wallet,
  transaction,
  signers = []
) {
  transaction.recentBlockhash = (await connection.getRecentBlockhash(commitment)).blockhash
  transaction.setSigners(wallet.publicKey, ...signers.map((s) => s.publicKey))
  if (signers.length > 0) {
    transaction.partialSign(...signers)
  }
  return await wallet.signTransaction(transaction)
}

export async function sendNewTransaction(
  connection,
  wallet,
  transaction,
  signers = [],
) {
  const signedTransaction = await signTransaction(connection, wallet, transaction, signers)
  return await sendSignedTransaction(connection, signedTransaction)
}

export async function sendSignedTransaction(connection, signedTransaction) {
  const rawTransaction = signedTransaction.serialize()

  const txid = await connection.sendRawTransaction(rawTransaction, {
    skipPreflight: true,
    preflightCommitment: commitment
  })

  return txid
}