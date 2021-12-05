import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, MANGO_PROGRAM_ID, RENT_PROGRAM_ID, programId, MANGO_GROUP_ACCOUNT, SERUM_PROGRAM_ID_V3, MARGIN_ACCOUNT_KEY_1, MARGIN_ACCOUNT_KEY_2, idsIndex } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import {  createKeyIfNotExists, signAndSendTransaction } from '../utils/web3';
import { FUND_DATA } from '../utils/programLayouts';
import { MarginAccountLayout } from '../utils/MangoLayout';
import { OpenOrders } from '@project-serum/serum';
import { IDS, MangoAccountLayout } from '@blockworks-foundation/mango-client';

export const MangoInitialize = () => {

  const ids= IDS['groups'][idsIndex];


    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleMangoInitialize = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()

      const fundStateAccount = await PublicKey.createWithSeed(
        key,
        FUND_ACCOUNT_KEY,
        programId,
      );
      console.log("fundStateAccount:: ", fundStateAccount.toBase58())
      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
   
      const mangoAccount = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID, MARGIN_ACCOUNT_KEY_1, MangoAccountLayout.span, transaction)
      console.log("mangoAccount created::",mangoAccount.toBase58())

     

      const dataLayout = struct([u8('instruction')])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 8,
        },
        data
      )
        const instruction = new TransactionInstruction({
        keys: [
            {pubkey: fundStateAccount, isSigner: false, isWritable: true},
            {pubkey: key, isSigner: true, isWritable: true },
            {pubkey: fundPDA[0], isSigner: false, isWritable: true },
            
            {pubkey: MANGO_PROGRAM_ID, isSigner: false, isWritable:false},
            {pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable:true},
            {pubkey: mangoAccount, isSigner: false, isWritable:true},
        ],
        programId,
        data
        });
  
        transaction.add(instruction);
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
        console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

        
    }

    

    return (
        <div className="form-div">
        <h4>Mango Initialize</h4>
          
          <button onClick={handleMangoInitialize}>Mango Margin Account Initialize</button>
          <br />
          
        </div>
    )
}
