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
} from '@solana/web3.js';
import fs, { accessSync } from 'mz/fs';
import path from 'path';
import * as borsh from 'borsh';
import { AccountLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";

import {
  getPayer,
  getRpcUrl,
  newAccountWithLamports,
  readAccountFromFile,
} from './utils';

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

let clientAccount: Account = new Account([170,163,68,150,44,133,201,245,30,119,40,77,63,59,122,83,141,220,73,191,120,225,216,8,242,136,58,37,77,78,21,163,187,208,141,20,240,90,186,164,158,134,192,215,22,208,231,141,119,62,233,181,190,219,113,114,225,50,226,144,149,206,19,37]);
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
  const GREETING_SEED = 'hello';
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
      GREETING_SIZE,
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payerAccount.publicKey,
        basePubkey: payerAccount.publicKey,
        seed: GREETING_SEED,
        newAccountPubkey: greetedPubkey,
        lamports,
        space: GREETING_SIZE,
        programId,
      }),
    );
    await sendAndConfirmTransaction(connection, transaction, [payerAccount]);
  }
}

/**
 * Say hello
 */
export async function sayHello(): Promise<void> {
  console.log('Saying hello to', greetedPubkey.toBase58());


  // let transaction = new Transaction().add(
  //     SystemProgram.assign({
  //       accountPubkey: clientAccount.publicKey,
  //       programId: programId
  //     }),
  // );
  // await sendAndConfirmTransaction(connection, transaction, [payerAccount, clientAccount]);

  // console.log("ownership changed")
  // let tempAccount = new Account([76,162,14,102,97,153,129,172,124,60,75,15,239,229,202,112,217,91,253,85,244,116,5,197,13,164,80,118,55,40,173,181,168,244,248,237,205,181,4,21,112,99,3,82,42,216,225,44,44,61,155,162,11,100,136,85,92,20,6,126,145,196,106,252]);
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

const instruction = new TransactionInstruction({
    keys: [
      {pubkey: new PublicKey("4UCsKKcosY6bC8S2Jnm4absYk159StJURTCG3CD3NjDH"), isSigner: false, isWritable: true},
      //{pubkey: greetedPubkey, isSigner: false, isWritable: true}, 
      {pubkey: clientAccount.publicKey, isSigner: true, isWritable:true},
      {pubkey: new PublicKey("5GtmfC4UTuUSzcZWovLBzvRGh8ozdnizhGGjfoYNuPcp"), isSigner: false, isWritable: true},
      {pubkey: new PublicKey("2n7DVeeRNmGcp3zVRZuRktcSr7ML7En9KdeboXqaDPXr"), isSigner: false, isWritable: true},
      {pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: true},
      {pubkey: new PublicKey("37hNAmbGNSKcAJgBfeSBS1239t9o992QqKhXo1VGJUdK"), isSigner: false, isWritable: true},
      { pubkey: PDA[0], isSigner: false, isWritable: false}
    ],
    programId,
    data: Buffer.from([1,50]), // All instructions are hellos
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
