import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , MANGO_GROUP_ACCOUNT, SOL_USDC_MARKET, SYSTEM_PROGRAM_ID, idsIndex, MARGIN_ACCOUNT_KEY_1, PERP_ACCOUNT_KEY_1, MANGO_PROGRAM_ID, MANGO_TOKENS, PERP_MARKETS, platformStateAccount} from '../utils/constants';
import { nu64, struct, u8 ,u32, ns64} from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, FUND_PDA_DATA, INVESTOR_DATA, PLATFORM_DATA } from '../utils/programLayouts';
import { MarginAccountLayout, selfTradeBehaviorLayout } from '../utils/MangoLayout';
import { devnet_pools, pools } from '../utils/pools'
import { updatePoolPrices } from './updatePrices';

import {mangoClosePosition, mangoOpenPosition} from '../utils/mango'
import BN from 'bn.js';
import { IDS, MangoAccountLayout, MangoClient, NodeBankLayout, perpAccountLayout, PerpMarketLayout, u64 } from '@blockworks-foundation/mango-client';
import { OpenOrders } from '@project-serum/serum';

export const MangoPlaceOrder = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const ids= IDS['groups'][idsIndex];

    const [size, setSize] = useState(0);
    const [price, setPrice] = useState(0)
    const [lendAmount, setLendAmount] = useState(0)
    const [orderPerpIndex, setOrderPerpIndex] = useState(0);
    const [addRemovePerpIndex, setAddRemovePerpIndex] = useState(0);
    const [addRemoveTokenIndex, setAddRemoveTokenIndex] = useState(0);

    const [lendTokenIndex, setLendTokenIndex] = useState(0)
    const [side, setSide] = useState(0);
    

    const handleMangoOpenOrders = async () => {
    
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

    console.log("FUND STTE:: ", fundStateAccount.toBase58())
    let fund_info = await connection.getAccountInfo(fundStateAccount);
    const fund_data = FUND_DATA.decode(fund_info.data);

    let pos_index = fund_data.no_of_margin_positions;
    console.log("pos_index", pos_index)
    console.log("orderPerpIndex:: ", orderPerpIndex)
  
    const client = new MangoClient()
    let mangoGroup = await client.getMangoGroup(connection, MANGO_GROUP_ACCOUNT)

        // open orders missing for this market; create a new one now
        const openOrdersSpace = OpenOrders.getLayout(mangoGroup.dexProgramId).span
        const openOrdersLamports =
          await connection.getMinimumBalanceForRentExemption(
            openOrdersSpace,
            'singleGossip'
          )
        let accInstr = await createKeyIfNotExists(
          walletProvider,
          "",
          mangoGroup.dexProgramId,
          key.toBase58().substr(0,20) + pos_index.toString() + orderPerpIndex.toString(),
          openOrdersSpace,
          transaction
        )
      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

     }

     

     const handlePerpMarketOrder = async () => {
       console.log("---** handleAddPerpMarketselected orderPerpIndex, size, side :", orderPerpIndex, size, side)
        const key = walletProvider?.publicKey;
        if (!key) {
            alert("connect wallet")
            return;
        };
        const transaction = new Transaction()

        // const perpAccount = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID, PERP_ACCOUNT_KEY_1, perpAccountLayout.span, transaction)
        // console.log("mangoAccount created::",perpAccount.toBase58())
        // const mango_token_index = 

        const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
        // const fundStateAccount = await PublicKey.createWithSeed(
        //     key,
        //     FUND_ACCOUNT_KEY,
        //     programId,
        // );
        let fundStateInfo = await connection.getAccountInfo((fundPDA[0]))
        let fundState = FUND_PDA_DATA.decode(fundStateInfo.data) 
    
        let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
        let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
        let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[1].nodeKeys[0]))
        let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
    
        
        const BTCBaseLotSize = 2 // baseLotSize / quoteLotSize
        console.log("size::::",size/(PERP_MARKETS[orderPerpIndex].contractSize)  )


        const maxSlippage = 0.15;
        // const price = 
        const priceAfterSlippage = price * (1 + (side === 0 ? 1 : -1) * maxSlippage)
        const baseUnit = Math.pow(10, PERP_MARKETS[orderPerpIndex].baseDecimals);
        const quoteUnit = Math.pow(10, PERP_MARKETS[orderPerpIndex].quoteDecimals);

        console.log("PERP_MARKETS[orderPerpIndex]:",PERP_MARKETS[orderPerpIndex])

        console.log("maxSlippage,price,priceAfterSlippage ,baseUnit,quoteUnit ::",maxSlippage,price,priceAfterSlippage ,baseUnit,quoteUnit)

        // const nativePrice = new BN(priceAfterSlippage * quoteUnit)
        //   .mul(new BN(PERP_MARKETS[orderPerpIndex].baseLotSize))
        //   .div((new BN(PERP_MARKETS[orderPerpIndex].quoteLotSize)).mul(new BN(baseUnit)));

        const nativePrice = (priceAfterSlippage * quoteUnit *  PERP_MARKETS[orderPerpIndex].baseLotSize )/(PERP_MARKETS[orderPerpIndex].quoteLotSize * baseUnit )
        console.log("nativePrice::",nativePrice)

        const dataLayout = struct([u8('instruction'), u8('perp_market_id'), u8('side'), nu64('price'), nu64('quantity')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
                instruction: 10,
                perp_market_id: orderPerpIndex,
                side : side,
                price: price,//nativePrice,
                quantity: size/(PERP_MARKETS[orderPerpIndex].contractSize) 
            },
            data
        )
          
        const keys = [
          { pubkey: fundPDA[0], isSigner: false, isWritable: true },
          { pubkey: key, isSigner: true, isWritable: true },
         
          { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
          { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
          { pubkey: fundState.mango_positions.mango_account , isSigner: false, isWritable: true },
         
          // { pubkey: fundPDA[0], isSigner: false, isWritable: false },
          { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: false },
          
          { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].publicKey), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].bidsKey), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].asksKey) , isSigner: false, isWritable: true },
          { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].eventsKey) , isSigner: false, isWritable: true }, 
          
          { pubkey: PublicKey.default, isSigner: false, isWritable: false },
      ]
    
      for(let i=0; i<keys.length;i++) {
        console.log("key:",i, keys[i].pubkey.toBase58())
      }
        const instruction = new TransactionInstruction({
            keys: keys,
            programId: programId,
            data
        });
    
        transaction.add(instruction);
        console.log(`transaction ::: `, transaction)
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash("finalized");
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;
    
        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("tx::: ", sign)
        console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
    
        
     }

     const callFromKeeper = async () => {
      const key = walletProvider?.publicKey;
      if (!key) {
          alert("connect wallet")
          return;
      };

      let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
      let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))


     const sign =  await client.cacheRootBanks(new PublicKey(ids.publicKey),
          mangoGroup.mangoCache,
          [new PublicKey(MANGO_TOKENS[0].rootKey),new PublicKey(MANGO_TOKENS[2].rootKey)],
          walletProvider
       )
     }

    const handleMangoPerpDeposit = async () => {

      const key = walletProvider?.publicKey;
      if (!key) {
          alert("connect wallet")
          return;
      };
      const transaction = new Transaction()
      // const fundStateAccount = await PublicKey.createWithSeed(
      //     key,
      //     FUND_ACCOUNT_KEY,
      //     programId,
      // );
      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      let fundStateInfo = await connection.getAccountInfo(fundPDA[0])
      let fundState = FUND_PDA_DATA.decode(fundStateInfo.data)
  
      let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
      let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))

     
      let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[lendTokenIndex].nodeKeys[0]))
      let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
  
      console.log("lendAmount::::",lendTokenIndex, lendAmount,ids.tokens[lendTokenIndex])
      // console.log("fundStateAccount::::",fundStateAccount.toBase58())
      console.log("fundState.fund_pda::::",fundState.fund_pda.toBase58())

      console.log("nodeBank.vault::::",nodeBank.vault.toBase58())

      console.log("fundState.mango_positions.mango_account::::",fundState.mango_positions.mango_account.toBase58())
      console.log("mangoGroup.mangoCache::::",mangoGroup.mangoCache.toBase58())

     

      const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
      if(!platformDataAcc){
        alert('platform state not initilaized');
        return;
      }
      const platformState = PLATFORM_DATA.decode(platformDataAcc.data)

      //TODO :
      // findTokenSlotIndex ids.tokens[lendTokenIndex].mintKey
      let fundTokenSlot = -1;
      console.log("check for mint:",ids.tokens[lendTokenIndex].mintKey)
      for(let i=0; i<fundState.tokens.length;i++){
        const t = fundState.tokens[i];
        console.log("i - mint:",i,platformState.token_list[t.index[t.mux]]?.mint.toBase58())
        if(platformState.token_list[t.index[t.mux]]?.mint.toBase58()==(ids.tokens[lendTokenIndex].mintKey)){
          fundTokenSlot=i;
          console.log("found:",i)
          break;
        }
      }
      if(fundTokenSlot==-1){
        alert('token not whitelisted on fund');
        return;
      } 
      // findMangoTokenIndex DONE

       //should be dynamic based on token
       const fundVault = fundState.tokens[fundTokenSlot].vault;
       console.log("fundVault::::",fundVault.toBase58())

      const dataLayout = struct([u8('instruction'),u8('token_slot_index'), u8('mango_token_index'), nu64('quantity')])
      const data = Buffer.alloc(dataLayout.span)
      console.log("fundTokenSlot:",fundTokenSlot)
      console.log("MANGO_TOKENS[lendTokenIndex].mangoTokenIndex:",MANGO_TOKENS[lendTokenIndex].mangoTokenIndex)

      dataLayout.encode(
          {
              instruction: 9,
              token_slot_index: fundTokenSlot,
              mango_token_index: MANGO_TOKENS[lendTokenIndex].mangoTokenIndex,
              quantity: lendAmount * 10 ** ids.tokens[lendTokenIndex].decimals
          },
          data
      )

      const keys = [
        { pubkey: fundState.fund_pda, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        // { pubkey: fundState.fund_pda, isSigner: false, isWritable: true },

        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },

        { pubkey: fundState.mango_positions.mango_account , isSigner: false, isWritable: true },
        { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.tokens[lendTokenIndex].rootKey), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.tokens[lendTokenIndex].nodeKeys[0]), isSigner: false, isWritable: true },
        { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: fundVault, isSigner: false, isWritable: true },
       ]
    
    for(let i=0; i<keys.length;i++) {
      console.log("key:",i, keys[i].pubkey.toBase58())
    }
   
      const instruction = new TransactionInstruction({
          keys: keys,
          programId: programId,
          data
      });
  
      transaction.add(instruction);
      console.log(`transaction ::: `, transaction)
      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash("finalized");
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;
  
      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("tx::: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

    }

   const handleMangoPerpWithdraw = async ( ) => {

    const key = walletProvider?.publicKey;
    if (!key) {
        alert("connect wallet")
        return;
    };
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    // const fundStateAccount = await PublicKey.createWithSeed(
    //     key,
    //     FUND_ACCOUNT_KEY,
    //     programId,
    // );
    let fundStateInfo = await connection.getAccountInfo(fundPDA[0])
    let fundState = FUND_PDA_DATA.decode(fundStateInfo.data) 

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[lendTokenIndex].nodeKeys[0]))
    let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)


    const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
    if(!platformDataAcc){
      alert('platform state not initilaized');
      return;
    }
    const platformState = PLATFORM_DATA.decode(platformDataAcc.data)

    //TODO :
    // findTokenSlotIndex ids.tokens[lendTokenIndex].mintKey
    let fundTokenSlot = -1;
    for(let i=0; i<fundState.tokens.length;i++){
      if(platformState.token_list[fundState.tokens[i].index[fundState.tokens[i].mux]]?.mint.toBase58() === ids.tokens[lendTokenIndex].mintKey){
        fundTokenSlot = i; 
        break;
      }
    }
    if(fundTokenSlot==-1){
      alert('token not whitelisted on fund');
      return;
    }
    // findMangoTokenIndex DONE

    //should be dynamic based on token
      const fundVault = fundState.tokens[fundTokenSlot].vault;
      console.log("fundVault::::",fundVault.toBase58())

    const transaction = new Transaction()
    console.log("fundTokenSlot:",fundTokenSlot)
    const dataLayout = struct([u8('instruction'),u8('token_slot_index'),u8('mango_token_index'),nu64('quantity')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 13,
            token_slot_index: fundTokenSlot,
            mango_token_index: MANGO_TOKENS[lendTokenIndex].mangoTokenIndex,
            quantity: lendAmount * 10 ** ids.tokens[lendTokenIndex].decimals
        },
        data
    )
      
    const keys = [
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },

      { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
      { pubkey: fundState.mango_positions.mango_account , isSigner: false, isWritable: true },
      // { pubkey: fundPDA[0], isSigner: false, isWritable: false },
      { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: false },
      { pubkey: new PublicKey(ids.tokens[lendTokenIndex].rootKey), isSigner: false, isWritable: false },
      { pubkey: new PublicKey(ids.tokens[lendTokenIndex].nodeKeys[0]), isSigner: false, isWritable: true },
      { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
      { pubkey: fundVault, isSigner: false, isWritable: true }, // Fund Vault
      { pubkey: mangoGroup.signerKey, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: PublicKey.default, isSigner: false, isWritable: false },
  ]

  for(let i=0; i<keys.length;i++) {
    console.log("key:",i, keys[i].pubkey.toBase58())
  }
    const instruction = new TransactionInstruction({
        keys: keys,
        programId: programId,
        data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

    }

    const handleConsumeEvents = async () => {

      const key = walletProvider?.publicKey;
      if (!key) {
          alert("connect wallet")
          return;
      };

      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const fundStateAccount = await PublicKey.createWithSeed(
          key,
          FUND_ACCOUNT_KEY,
          programId,
      );
      let fundStateInfo = await connection.getAccountInfo((fundStateAccount))
      let fundState = FUND_DATA.decode(fundStateInfo.data) 
  

      const transaction = new Transaction()

      let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
      let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
      let perpMarketX = await client.getPerpMarket(new PublicKey(ids.perpMarkets[1].publicKey))

      // client.consumeEvents(mangoGroup,perpMarketX )
    

    //   const dataLayout = struct([u8('instruction'),u8('limit')])
    //   const data = Buffer.alloc(dataLayout.span)
    //   dataLayout.encode(
    //       {
    //           instruction: 15,
    //           limit: 2,
    //       },
    //       data
    //   )

    //   const keys = [
       
    //     // { pubkey: key, isSigner: true, isWritable: true },
    //     // { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
    //     { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: false },
    //     { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: false },
    //     { pubkey: new PublicKey(ids.perpMarkets[1].publicKey), isSigner: false, isWritable: true },
    //     { pubkey: new PublicKey(ids.perpMarkets[1].eventsKey) , isSigner: false, isWritable: true }, 
    //     { pubkey: fundState.mango_positions.mango_account , isSigner: false, isWritable: true },

    // ]
  
    // for(let i=0; i<keys.length;i++) {
    //   console.log("key:",i, keys[i].pubkey.toBase58())
    // }
    //   const instruction = new TransactionInstruction({
    //       keys: keys,
    //       programId: programId,
    //       data
    //   });
  
    //   transaction.add(instruction);
    //   console.log(`transaction ::: `, transaction)
    //   transaction.feePayer = key;
    //   let hash = await connection.getRecentBlockhash("finalized");
    //   console.log("blockhash", hash);
    //   transaction.recentBlockhash = hash.blockhash;
  
    //   const sign = await signAndSendTransaction(walletProvider, transaction);
    //   console.log("tx::: ", sign)
    //   console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
    }


    return (
        <div className="form-div">

        <h4>LEND TOKENS</h4>
          Amount :::  <input type="number" value={lendAmount} onChange={(event) => setLendAmount(event.target.value)} />
          <select name="side" width = "100px" onChange={(event) => setLendTokenIndex(event.target.value)}>
              {
               ids.tokens.map( (i,index) => <option value={index}>{i.symbol}</option> )
              }
            </select>
          <button onClick={handleMangoPerpDeposit}>DEPOSIT</button>
          <button onClick={handleMangoPerpWithdraw}>WITHDRAW</button>



          <br/><hr/><br/>
          <h4>ADD/REMOVE PERP</h4>
          <select name="side" width = "100px" onChange={(event) => setAddRemovePerpIndex(event.target.value)}>
              <option value={0}>MNGO-PERP</option>
              <option value={1}>BTC-PERP</option>
              <option value={2}>ETH-PERP</option>
              <option value={3}>SOL-PERP</option>
              <option value={4}>SRM-PERP</option>
              <option value={5}>RAY-PERP</option>
              <option value={6}>FTT-PERP</option>
              <option value={7}>ADA-PERP</option>
            </select>
          {/* <button onClick={handleAddPerpMarket}>ADD</button>
          <button onClick={handleRemovePerpMarket}>REMOVE</button> */}

          <br/><hr/><br/>

            <h4>Mango Place</h4> 
            Size ::: {' '}
            <input type="number" value={size} onChange={(event) => setSize(event.target.value)} />
            <br />
            Price ::: {' '}
            <input type="number" value={price} onChange={(event) => setPrice(event.target.value)} />
            <br />
            <label htmlFor="side">Buy/Sell</label><br/>

            <select name="side" width = "100px" onChange={(event) => setSide(parseInt(event.target.value))}>
              <option value={0}>Buy</option>
              <option value={1}>Sell</option>
            </select>

            <select name="side" width = "100px" onChange={(event) => setOrderPerpIndex(parseInt(event.target.value))}>
              <option value={0}>MNGO-PERP</option>
              <option value={1}>BTC-PERP</option>
              <option value={2}>ETH-PERP</option>
              <option value={3}>SOL-PERP</option>
              <option value={4}>SRM-PERP</option>
              <option value={5}>RAY-PERP</option>
              <option value={6}>FTT-PERP</option>
              <option value={7}>ADA-PERP</option>
            </select>

          <button onClick={handlePerpMarketOrder}>ORDER</button>

          <br />
          <button onClick={handleMangoOpenOrders}>Open order init</button>
          <br />
          <button onClick={handleConsumeEvents}> Consume Events </button>

          <button onClick={callFromKeeper}> KEEPER </button>

          
          <br />
        </div>
    )
}