import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount } from '../utils/web3';

export const Deposit = () => {

  const [amount, setAmount] = useState(0);
  const [fundPDA, setFundPDA] = useState('');

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const fundAccount = GlobalState.useState(s => s.createFundPublicKey);

  const handleDeposit = async () => {

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };

    const clientAccount = await connection.getAccountInfo(key);
 
    const baseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'));

    const transaction = new Transaction()

    const PDA = await PublicKey.findProgramAddress([key.toBuffer()], programId);
    const MPDA = new PublicKey('FqV8b3zWwLLT8SgkP3jQoZntbv5WAuxpNYmVk9HtR2yQ')

    const associatedTokenAddress1 = await createAssociatedTokenAccount(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), PDA[0], transaction);

    const investerStateAccount = await createKeyIfNotExists(walletProvider, clientAccount, programId, (64 + 16 + 2))


    console.log("PDA:", PDA[0].toBase58())
    console.log("MPDA: ", MPDA.toBase58())
    const dataLayout = struct([u8('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1,
        amount: amount * 1000000000
      },
      data
    )

    // DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w
    // HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o
    // HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: new PublicKey('A6R1siU7eCLiTFTswtofYeUtaRCPWn65yT4VhfY1vPEK'), isSigner: false, isWritable: true }, //fund State Account
        { pubkey: investerStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: baseTokenAccount, isSigner: false, isWritable: true }, // Investor Base Token Account
        { pubkey: associatedTokenAddress1, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: MPDA, isSigner: false, isWritable: false },
        { pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: true },
      ],
      programId,
      data
    });

    // let trans = await setWalletTransaction(instruction, key);


    // let signature = await signAndSendTransaction(walletProvider, trans);
    // let result = await connection.confirmTransaction(signature, "singleGossip");
    // console.log("money sent", result);
    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
  }


  return (
    <div className="form-div">
      <h4>Deposit</h4>
      amount ::: {' '}
      <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
      <br />
      fund pda :: {' '}
      <input type="text"
        onChange={(e) => setFundPDA(e.target.value)}
        value={fundPDA}
      />
      <button onClick={handleDeposit}>Deposit</button>
    </div>
  )
}


