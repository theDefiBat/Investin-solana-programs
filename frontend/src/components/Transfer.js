import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, idsIndex, MANGO_GROUP_ACCOUNT, MANGO_PROGRAM_ID, platformStateAccount, priceStateAccount, programId, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, FUND_PDA_DATA, INVESTOR_DATA, PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, orcaPools, pools, raydiumPools } from '../utils/pools'

import { updatePoolPrices } from './updatePrices';
import {
  IDS,
  MangoClient, MangoGroupLayout, MarginAccountLayout
} from '@blockworks-foundation/mango-client'
import { TOKENS } from '../utils/tokens';

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
    // const fundStateAccount = await PublicKey.createWithSeed(
    //   key,
    //   FUND_ACCOUNT_KEY,
    //   programId,
    // );

    let fundPDAState = await connection.getAccountInfo(fundPDA[0])
    if (fundPDAState == null) {
      alert("fundPDAState account not found")
      return
    }
    console.log("fundPDAState:",fundPDAState)
    let fund_data = FUND_PDA_DATA.decode(fundPDAState.data)
    if (!fund_data.is_initialized) {
      alert("fund not initialized!")
      return
    }
    console.log("fund_data::",fund_data)

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
    let platform_data = PLATFORM_DATA.decode(platData.data)
    console.log("plat info:: ", platform_data)

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    // let mangoCache = await mangoGroup.loadCache(connection)
    console.log("mangoCache:",mangoGroup.mangoCache.toBase58())


    let filt_pools = []
    let WSOLWhitelisted = false;
    let MSOLWhitelisted = false;

    for (let i = 1; i<8; i++) {
      if (fund_data.tokens[i].balance > 0) {
        let mint = platform_data.token_list[fund_data.tokens[i].index[fund_data.tokens[i].mux]].mint;
        if(mint.toBase58() === TOKENS.WSOL.mintAddress){
          WSOLWhitelisted=true;
        } else if(mint.toBase58() === TOKENS.MSOL.mintAddress){
          MSOLWhitelisted=true;
        }
        if(fund_data.tokens[i].mux === 0){
          let x = raydiumPools.find(p => p.coin.mintAddress == mint.toBase58())
          filt_pools.push(x)
        } else {
          let x = orcaPools.find(p => p.coin.mintAddress == mint.toBase58())
          filt_pools.push(x)
        }
      }  
    }
    //send WSOL everytime 
    if(!WSOLWhitelisted){
      const wsol_usdc_pool = raydiumPools.find(p => p.name == 'WSOL-USDC');
      console.log("pushing WSOL pool")
      filt_pools.push(wsol_usdc_pool)
    }
    if(!MSOLWhitelisted){
      const msol_usdc_pool = orcaPools.find(p => p.name == 'MSOL-USDC');
      console.log("pushing MSOL pool")
      filt_pools.push(msol_usdc_pool)
    }
    console.log("filt_pools:",filt_pools)
    // updatePoolPrices(transaction, devnet_pools)
    // console.log("after updatePoolPrices:: ")


    // -------------

    const routerPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const fundBaseTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(TOKENS.USDC.mintAddress));
    const routerBaseTokenAccount = await findAssociatedTokenAddress(routerPDA[0], new PublicKey(TOKENS.USDC.mintAddress));

    const managerBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TOKENS.USDC.mintAddress), key, transaction);
    const investinBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TOKENS.USDC.mintAddress), adminAccount, transaction);


    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 2,
      },
      data
    )

    console.log("keys : fund_mango_account,platform_data.investin_vault:: ",fund_mango_account.toBase58() ,platform_data.investin_vault.toBase58())
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
        { pubkey: platform_data.investin_vault, isSigner: false, isWritable: true },

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
