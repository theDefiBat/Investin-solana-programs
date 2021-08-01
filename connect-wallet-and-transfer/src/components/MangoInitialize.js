import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, MANGO_PROGRAM_ID, RENT_PROGRAM_ID, programId, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, SERUM_PROGRAM_ID_V3, MARGIN_ACCOUNT_KEY_1, MARGIN_ACCOUNT_KEY_2 } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import {  createKeyIfNotExists, signAndSendTransaction } from '../utils/web3';
import { FUND_DATA } from '../utils/programLayouts';
import { MarginAccountLayout } from '../utils/MangoLayout';
import { MangoClient } from '@blockworks-foundation/mango-client';
import { OpenOrders } from '@project-serum/serum';

export const MangoInitialize = () => {

    const [fundPDA, setFundPDA] = useState('')
    const [fundStateAccount, setFundStateAccount] = useState('')

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleMangoInitialize = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()

    //   const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(TEST_TOKENS['USDR'].mintAddress));
      // TODO copy program layout from mango and get span 
      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY_1, MarginAccountLayout.span, transaction)
      const margin_account_acc2 = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY_2, MarginAccountLayout.span, transaction)

      // const margin_info = await connection.getAccountInfo(margin_account_acc)
      // const margin_data = MarginAccountLayout.decode(margin_info)
      // console.log('margin_acc :: ', margin_data)

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
            {pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true},
            {pubkey: key, isSigner: true, isWritable: true },
            {pubkey: fundPDA, isSigner: false, isWritable: true },
            
            {pubkey: MANGO_PROGRAM_ID_V2, isSigner: false, isWritable:true},
            {pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable:true},
            {pubkey: margin_account_acc, isSigner: false, isWritable:true},
            {pubkey: RENT_PROGRAM_ID, isSigner: false, isWritable:true},
            
            
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
    }
const handleMangoOpenOrders = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()
      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY_1, MarginAccountLayout.span, transaction)
      const margin_account_acc2 = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY_2, MarginAccountLayout.span, transaction)

      const client = new MangoClient()
      let marginAccount = await client.getMarginAccount(connection, margin_account_acc, SERUM_PROGRAM_ID_V3)
      let mangoGroup = await client.getMangoGroup(connection, MANGO_GROUP_ACCOUNT)


      const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
      const openOrdersLamports =
        await connection.getMinimumBalanceForRentExemption(
          openOrdersSpace,
          'singleGossip'
        )
      for (let i = 0; i < marginAccount.openOrders.length; i++) {
        if (
          marginAccount.openOrders[i].equals(PublicKey.default)
        ) {
          // open orders missing for this market; create a new one now
          const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
          const openOrdersLamports =
            await connection.getMinimumBalanceForRentExemption(
              openOrdersSpace,
              'singleGossip'
            )
          const accInstr = await createKeyIfNotExists(
            walletProvider,
            "",
            mangoGroup.dexProgramId,
            i.toString(),
            openOrdersSpace,
            transaction
          )
          // openOrdersKeys.push(accInstr)
        } else {
          // openOrdersKeys.push(marginAccount.openOrders[i])
        }
      }
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
    }

    const handleGetFunds = async () => {

        const key = walletProvider?.publicKey;  
        if (!key ) {
          alert("connect wallet")
          return;
        }
        const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
        setFundPDA(fundPDA[0].toBase58())
    
        const fundStateAccount = await PublicKey.createWithSeed(
          key,
          FUND_ACCOUNT_KEY,
          programId,
        );
    
        console.log("FUND STTE:: ", fundStateAccount.toBase58())
        setFundStateAccount(fundStateAccount.toBase58())
    
        let x = await connection.getAccountInfo(fundStateAccount)
        if (x == null)
        {
          alert("fund account not found")
          return
        }
        console.log(x)
        let fundState = FUND_DATA.decode(x.data)
        if (!fundState.is_initialized) {
          alert("fund not initialized!")
          return
        }
        console.log(fundState)

      //   const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, null)
      // const margin_info = await connection.getAccountInfo(margin_account_acc)
      // console.log('margin_key :: ', margin_info)

      // const margin_data = MarginAccountLayout.decode(margin_info)
      // console.log('margin_acc :: ', margin_data)
    }

    return (
        <div className="form-div">
        <h4>Mango Initialize</h4>
          
          <button onClick={handleMangoInitialize}>Mango Margin Account Initialize</button>
          <br />
          <button onClick={handleMangoOpenOrders}>Mango Open order create</button>
          <br />
          <button onClick={handleGetFunds}>GetFundInfo</button>
          <br />
        </div>
    )
}
