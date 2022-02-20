import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, priceStateAccount, connection, programId, TOKEN_PROGRAM_ID, FUND_ACCOUNT_KEY, idsIndex, SYSTEM_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { findAssociatedTokenAddress, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { TEST_TOKENS } from '../utils/tokens'
import { FUND_DATA } from '../utils/programLayouts';
import { devnet_pools } from '../utils/pools'
import { updatePoolPrices } from './updatePrices';
import { IDS } from '@blockworks-foundation/mango-client';


export const MigrateState = () => {

  const ids= IDS['groups'][idsIndex];

    const [fundPDA, setFundPDA] = useState('');
    // const [fundStateAccount, setFundStateAccount] = useState('');

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleMigrate = async () => {
    
        const key = walletProvider?.publicKey;
        
        const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
        const fundStateAccount = await PublicKey.createWithSeed(
          key,
          FUND_ACCOUNT_KEY,
          programId,
        );
        console.log("FUND fundStateAccount:: ", fundStateAccount.toBase58())

        let x = await connection.getAccountInfo(fundStateAccount)
        if (x == null)
        {
          alert("fund account not found")
          return
        }

        if (!key) {
          alert("connect wallet")
          return;
        };

        if (fundStateAccount == ''){
          alert("get info first!")
          return
        }
      
        const transaction = new Transaction()
        const dataLayout = struct([u8('instruction')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode({instruction: 27},data)
        
        const migrate_instruction = new TransactionInstruction({
          keys: [
            {pubkey: key, isSigner: true, isWritable: true },
            {pubkey: fundPDA[0], isSigner: false, isWritable:true},
            {pubkey: fundStateAccount, isSigner: false, isWritable: true},

            // {pubkey: priceStateAccount, isSigner: false, isWritable:true},
            // {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable:true},

            // {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
            // {pubkey: managerBaseTokenAccount, isSigner: false, isWritable:true},
            // {pubkey: investinBaseTokenAccount, isSigner: false, isWritable:true},
            {pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: true},
          ],
          programId,
          data
        });
    
        transaction.add(migrate_instruction);
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("tx perf: ", sign)
        console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
  }
    


    return (
        <div className="form-div">
            <h4>Migrate State </h4>
         
          <br />
          <button onClick={handleMigrate}>Migrate</button>
          <br />
          New State :: {}

        </div>
      )
}

    