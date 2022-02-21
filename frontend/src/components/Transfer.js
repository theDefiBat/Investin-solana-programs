import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, idsIndex, MANGO_GROUP_ACCOUNT, MANGO_PROGRAM_ID, platformStateAccount, priceStateAccount, programId, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA, PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools'

import { updatePoolPrices } from './updatePrices';
import {
  IDS,
  MangoClient, MangoGroupLayout, MarginAccountLayout
} from '@blockworks-foundation/mango-client'

export const Transfer = () => {

  const ids= IDS['groups'][idsIndex];


  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleTransfer = async () => {

    console.log("**handleTransfer :")

    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };

    const transaction = new Transaction()
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    let x = await connection.getAccountInfo(fundStateAccount)
    if (x == null) {
      alert("fund account not found")
      return
    }
    console.log(x)
    let fund_data = FUND_DATA.decode(x.data)
    if (!fund_data.is_initialized) {
      alert("fund not initialized!")
      return
    }

    let fundInvestorAccs = []
    for (let i = 0; i < 10; i++) {
      if(fund_data.investors[i].toBase58() !== '11111111111111111111111111111111'){
        fundInvestorAccs.push({
          pubkey: new PublicKey(fund_data.investors[i].toBase58()) ,
          isSigner: false,
          isWritable: true 
        })
       }
    }

    console.log("** fundInvestorAccs:",fundInvestorAccs)


    let fund_mango_account = fund_data.mango_positions.mango_account

    let platData = await connection.getAccountInfo(platformStateAccount)
    let plat_info = PLATFORM_DATA.decode(platData.data)
    console.log("plat info:: ", plat_info)

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    // let mangoCache = await mangoGroup.loadCache(connection)
    console.log("mangoCache:",mangoGroup.mangoCache.toBase58())

    updatePoolPrices(transaction, devnet_pools)
    console.log("after updatePoolPrices:: ")


    // -------------

    const routerPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const fundBaseTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(ids.tokens[0].mintKey));
    const routerBaseTokenAccount = await findAssociatedTokenAddress(routerPDA[0], new PublicKey(ids.tokens[0].mintKey));

    const managerBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), key, transaction);
    const investinBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), adminAccount, transaction);


    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 2,
      },
      data
    )

    console.log("keys : fund_mango_account,plat_info.investin_vault:: ",fund_mango_account.toBase58() ,plat_info.investin_vault.toBase58())
    const transfer_instruction = new TransactionInstruction({
      keys: [
        { pubkey: platformStateAccount, isSigner: false, isWritable: true },
        { pubkey: fundPDA[0], isSigner: false, isWritable: true },

        { pubkey: fund_mango_account, isSigner: false, isWritable: true },
        { pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable: true },
        { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: true },
        { pubkey: MANGO_PROGRAM_ID, isSigner: false, isWritable: false },

        { pubkey: key, isSigner: true, isWritable: true },

        { pubkey: routerBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: fundBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: managerBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: plat_info.investin_vault, isSigner: false, isWritable: true },

        { pubkey: routerPDA[0], isSigner: false, isWritable: true },

        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },


        //investor state accounts
        ...fundInvestorAccs

      ],
      programId,
      data
    });

    transaction.add(transfer_instruction);
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
      <h4>Manager Transfer</h4>

      <button onClick={handleTransfer}>Transfer</button>
      <br />

    </div>
  )
}
