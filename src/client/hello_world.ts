/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import {
  Account,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  Keypair,
} from '@solana/web3.js';

import {getPoolByTokenMintAddresses} from "./liquidity"
import fs, { accessSync } from 'mz/fs';
import path from 'path';
import * as borsh from 'borsh';
import { AccountLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
// @ts-ignore
import { u8, nu64, struct } from 'buffer-layout'
import {
  getPayer,
  getRpcUrl,
  newAccountWithLamports,
  readAccountFromFile,
} from './utils';
import { swap , loadInfo} from './swap';

/**
 * Connection to the network
 */
let connection: Connection;

/**
 * Connection to the network
 */
let payerAccount: Account;

/**
 * Hello world's program id
 */
let programId: PublicKey;

/**
 * The public key of the account we are saying hello to
 */
let greetedPubkey: PublicKey;

let clientAccount: Account = new Account([116,252,167,101,186,83,65,192,133,216,186,17,79,88,19,249,12,85,255,140,19,101,4,233,105,80,14,111,133,107,123,3,217,180,92,0,197,5,141,20,70,238,87,223,135,91,117,53,187,81,22,117,90,239,30,15,88,200,147,207,126,182,198,209]);
//let clientKey: PublicKey = new PublicKey("3it9nJcySN83WcDdDrM4ts7m8Pvg74GuMZvprn7PZQhD");

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, '../../dist/program');

/**
 * Path to program shared object file which should be deployed on chain.
 * This file is created when running either:
 *   - `npm run build:program-c`
 *   - `npm run build:program-rust`
 */
const PROGRAM_SO_PATH = path.join(PROGRAM_PATH, 'helloworld.so');

/**
 * Path to the keypair of the deployed program.
 * This file is created when running `solana program deploy dist/program/helloworld.so`
 */
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'helloworld-keypair.json');

/**
 * The state of a greeting account managed by the hello world program
 */
class GreetingAccount {
  counter = 0;
  constructor(fields: {counter: number} | undefined = undefined) {
    if (fields) {
      this.counter = fields.counter;
    }
  }
}

/**
 * Borsh schema definition for greeting accounts
 */
const GreetingSchema = new Map([
  [GreetingAccount, {kind: 'struct', fields: [['counter', 'u32']]}],
]);


/**
 * The expected size of each greeting account.
 */
const GREETING_SIZE = borsh.serialize(GreetingSchema, new GreetingAccount())
  .length;

/**
 * Establish a connection to the cluster
 */
export async function establishConnection(): Promise<void> {
  const rpcUrl = await getRpcUrl();
  connection = new Connection(rpcUrl, 'confirmed');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', rpcUrl, version);
}

/**
 * Establish an account to pay for everything
 */
export async function establishPayer(): Promise<void> {
  let fees = 0;
  if (!payerAccount) {
    const {feeCalculator} = await connection.getRecentBlockhash();

    // Calculate the cost to fund the greeter account
    fees += await connection.getMinimumBalanceForRentExemption(GREETING_SIZE);

    // Calculate the cost of sending transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag

    try {
      // Get payer from cli config
      payerAccount = await getPayer();
    } catch (err) {
      // Fund a new payer via airdrop
      payerAccount = await newAccountWithLamports(connection, fees);
    }
  }

  const lamports = await connection.getBalance(payerAccount.publicKey);
  if (lamports < fees) {
    // This should only happen when using cli config keypair
    const sig = await connection.requestAirdrop(
      payerAccount.publicKey,
      fees - lamports,
    );
    await connection.confirmTransaction(sig);
  }

  console.log(
    'Using account',
    payerAccount.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'SOL to pay for fees',
  );
}

/**
 * Check if the hello world BPF program has been deployed
 */
export async function checkProgram(): Promise<void> {
  // Read program id from keypair file
  try {
    const programAccount = await readAccountFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programAccount.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy dist/program/helloworld.so\``,
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        'Program needs to be deployed with `solana program deploy dist/program/helloworld.so`',
      );
    } else {
      throw new Error('Program needs to be built and deployed');
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);

  // Derive the address of a greeting account from the program so that it's easy to find later.
  const GREETING_SEED = 'demouts';
  greetedPubkey = await PublicKey.createWithSeed(
    payerAccount.publicKey,
    GREETING_SEED,
    programId,
  );

  // Check if the greeting account has already been created
  const greetedAccount = await connection.getAccountInfo(greetedPubkey);
  if (greetedAccount === null) {
    console.log(
      'Creating account',
      greetedPubkey.toBase58(),
      'to say hello to',
    );
    const lamports = await connection.getMinimumBalanceForRentExemption(
      //GREETING_SIZE,
      72,
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payerAccount.publicKey,
        basePubkey: payerAccount.publicKey,
        seed: GREETING_SEED,
        newAccountPubkey: greetedPubkey,
        lamports,
        // space: GREETING_SIZE,
        space: 72,
        programId,
      }),
    );
    console.log(greetedPubkey.toBase58())
    await sendAndConfirmTransaction(connection, transaction, [payerAccount]);
  }
}

/**
 * Say hello
 */
export async function sayHello(): Promise<void> {
  console.log('Saying hello to', greetedPubkey.toBase58());


  // console.log("ownership changed")
//   let tempAccount = new Account([76,162,14,102,97,153,129,172,124,60,75,15,239,229,202,112,217,91,253,85,244,116,5,197,13,164,80,118,55,40,173,181,168,244,248,237,205,181,4,21,112,99,3,82,42,216,225,44,44,61,155,162,11,100,136,85,92,20,6,126,145,196,106,252]);
//   const tempTokenAccount = new Account();
//   const createTempTokenAccountIx = SystemProgram.createAccount({
//     programId: TOKEN_PROGRAM_ID,
//     space: AccountLayout.span,
//     lamports: await connection.getMinimumBalanceForRentExemption(AccountLayout.span, 'singleGossip'),
//     fromPubkey: clientAccount.publicKey,
//     newAccountPubkey: tempTokenAccount.publicKey
// });
// const initTempAccountIx = Token.createInitAccountInstruction(TOKEN_PROGRAM_ID, new PublicKey("6SPZWybQ9hi63usgh9vFecp2hczKT8jLakmUUXZ2JhUp"), tempTokenAccount.publicKey, clientAccount.publicKey);

// const tx = new Transaction().add(createTempTokenAccountIx, initTempAccountIx);

// await connection.sendTransaction(tx, [clientAccount, tempTokenAccount], {skipPreflight: false, preflightCommitment: 'singleGossip'});

// console.log("new account created: ", tempTokenAccount.publicKey.toBase58());



const PDA = await PublicKey.findProgramAddress([Buffer.from("fund")], programId);
const MPDA = await PublicKey.findProgramAddress([Buffer.from("manager")], programId);

console.log("PDA:", PDA[0].toBase58())
console.log("MPDA: ", MPDA[0].toBase58())
const dataLayout = struct([u8('instruction'), nu64('amount')])
const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction: 1,
      amount: 10*1000000
    },
    data
  )
const instruction = new TransactionInstruction({
    keys: [
      {pubkey: greetedPubkey, isSigner: false, isWritable: true}, 
      {pubkey: clientAccount.publicKey, isSigner: true, isWritable:true},
      {pubkey: new PublicKey("5AasjtqKzRsD98XsUGZfjW5eoq3MVHLn2VMNoEZZnqvK"), isSigner: false, isWritable: true},
      {pubkey: new PublicKey("4LuQ3tEVtpMFQUUNi9EQbFeKCXJE1BtBVCkR9zHo4mES"), isSigner: false, isWritable: true},
      {pubkey: new PublicKey("8FEaa7yPbe2X5YWhrxmNrXKJA1mGq9nZEBShaKNKSC8U"), isSigner: false, isWritable: true},
      { pubkey: PDA[0], isSigner: false, isWritable: false},
      { pubkey: MPDA[0], isSigner: false, isWritable: false},
      {pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: true},
    ],
    programId,
    data
  });
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payerAccount, clientAccount],
  );
}

/**
 * Report the number of times the greeted account has been said hello to
 */
export async function reportGreetings(): Promise<void> {
  const accountInfo = await connection.getAccountInfo(greetedPubkey);
  if (accountInfo === null) {
    throw 'Error: cannot find the greeted account';
  }
  const greeting = borsh.deserialize(
    GreetingSchema,
    GreetingAccount,
    accountInfo.data,
  );
  console.log(
    greetedPubkey.toBase58(),
    'has been greeted',
    greeting.counter,
    'time(s)',
  );
}

export function swapInstruction(
  programId: PublicKey,
  RprogramId: PublicKey,
  // tokenProgramId: PublicKey,
  // amm
  ammId: PublicKey,
  ammAuthority: PublicKey,
  ammOpenOrders: PublicKey,
  ammTargetOrders: PublicKey,
  poolCoinTokenAccount: PublicKey,
  poolPcTokenAccount: PublicKey,
  // serum
  serumProgramId: PublicKey,
  serumMarket: PublicKey,
  serumBids: PublicKey,
  serumAsks: PublicKey,
  serumEventQueue: PublicKey,
  serumCoinVaultAccount: PublicKey,
  serumPcVaultAccount: PublicKey,
  serumVaultSigner: PublicKey,
  // user
  userSourceTokenAccount: PublicKey,
  userDestTokenAccount: PublicKey,
  userOwner: PublicKey,

  amountIn: number,
  minAmountOut: number
): TransactionInstruction {
  
  const dataLayout = struct([u8('instruction1'), u8('instruction'), nu64('amountIn'), nu64('minAmountOut')])

  const keys = [
    // raydium pool
    { pubkey: RprogramId, isSigner: false, isWritable: true },
    // spl token
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
    // amm
    { pubkey: ammId, isSigner: false, isWritable: true },
    { pubkey: ammAuthority, isSigner: false, isWritable: true },
    { pubkey: ammOpenOrders, isSigner: false, isWritable: true },
    { pubkey: ammTargetOrders, isSigner: false, isWritable: true },
    { pubkey: poolCoinTokenAccount, isSigner: false, isWritable: true },
    { pubkey: poolPcTokenAccount, isSigner: false, isWritable: true },
    // serum
    { pubkey: serumProgramId, isSigner: false, isWritable: true },
    { pubkey: serumMarket, isSigner: false, isWritable: true },
    { pubkey: serumBids, isSigner: false, isWritable: true },
    { pubkey: serumAsks, isSigner: false, isWritable: true },
    { pubkey: serumEventQueue, isSigner: false, isWritable: true },
    { pubkey: serumCoinVaultAccount, isSigner: false, isWritable: true },
    { pubkey: serumPcVaultAccount, isSigner: false, isWritable: true },
    { pubkey: serumVaultSigner, isSigner: false, isWritable: true },
    { pubkey: userSourceTokenAccount, isSigner: false, isWritable: true },
    { pubkey: userDestTokenAccount, isSigner: false, isWritable: true },
    { pubkey: userOwner, isSigner: false, isWritable: true }
  ]

  const data = Buffer.alloc(dataLayout.span)
  dataLayout.encode(
    {
      instruction1: 3,
      instruction: 9,
      amountIn,
      minAmountOut
    },
    data
  )

  return new TransactionInstruction({
    keys,
    programId,
    data
  })
}

export async function swapToken() {

  let programId = new PublicKey("ChmfNvH4Q15C7XXGC132UTbLbdUrQF5hUXFejyG22UZ2")
  let wallet = new Account([116,252,167,101,186,83,65,192,133,216,186,17,79,88,19,249,12,85,255,140,19,101,4,233,105,80,14,111,133,107,123,3,217,180,92,0,197,5,141,20,70,238,87,223,135,91,117,53,187,81,22,117,90,239,30,15,88,200,147,207,126,182,198,209])

  let pools = await loadInfo(connection)

  const MPDA = await PublicKey.findProgramAddress([Buffer.from("manager")], programId);

  let pool = getPoolByTokenMintAddresses("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R")
  if(pool) {
    //let txid = await swap(connection, wallet, pools[pool.lp.mintAddress], "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", 
      //"4LuQ3tEVtpMFQUUNi9EQbFeKCXJE1BtBVCkR9zHo4mES", "3xBXRmLvyXy9a8F3kkgocExWok5yByYqFXRVbCmcsHz1", "10", 0.1)
      //console.log("Txid: ", txid.toString())
      const poolInfo = pools[pool.lp.mintAddress]
      const transaction = new Transaction()
      transaction.add(
        swapInstruction(
          programId,
          new PublicKey(poolInfo.programId),
          new PublicKey(poolInfo.ammId),
          new PublicKey(poolInfo.ammAuthority),
          new PublicKey(poolInfo.ammOpenOrders),
          new PublicKey(poolInfo.ammTargetOrders),
          new PublicKey(poolInfo.poolCoinTokenAccount),
          new PublicKey(poolInfo.poolPcTokenAccount),
          new PublicKey(poolInfo.serumProgramId),
          new PublicKey(poolInfo.serumMarket),
          new PublicKey(poolInfo.serumBids),
          new PublicKey(poolInfo.serumAsks),
          new PublicKey(poolInfo.serumEventQueue),
          new PublicKey(poolInfo.serumCoinVaultAccount),
          new PublicKey(poolInfo.serumPcVaultAccount),
          new PublicKey(poolInfo.serumVaultSigner),
          new PublicKey("8xhHD3ZBvU3HrUqr9bdRP1fGi8dJUNxM6vSBnUmgQoqt"),
          new PublicKey("8FEaa7yPbe2X5YWhrxmNrXKJA1mGq9nZEBShaKNKSC8U"),
          //wallet.publicKey,
          MPDA[0],
          Math.floor(0.51*(1000000)),
          Math.floor(100)
        ))
        return await connection.sendTransaction(transaction, [wallet])
  }
}