import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , MANGO_GROUP_ACCOUNT, SOL_USDC_MARKET, SYSTEM_PROGRAM_ID} from '../utils/constants';
import { nu64, struct, u8 ,u32, ns64} from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA } from '../utils/programLayouts';
import { MarginAccountLayout, selfTradeBehaviorLayout } from '../utils/MangoLayout';
import { devnet_pools, pools } from '../utils/pools'
import { updatePoolPrices } from './updatePrices';

import {mangoClosePosition, mangoOpenPosition} from '../utils/mango'
import BN from 'bn.js';
import { IDS, MangoClient, NodeBankLayout } from '@blockworks-foundation/mango-client';
import { OpenOrders } from '@project-serum/serum';

export const MangoPlaceOrder = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  let ids;
  if(process.env.REACT_APP_NETWORK==='devnet'){
     ids = IDS['groups'][2]
  } else {
     ids = IDS['groups'][0]
  }
    const [size, setSize] = useState(0);
    const [lendAmount, setLendAmount] = useState(0)
    const [orderPerpIndex, setOrderPerpIndex] = useState(0);
    const [addRemovePerpIndex, setAddRemovePerpIndex] = useState(0);
    const [addRemoveTokenIndex, setAddRemoveTokenIndex] = useState(0);
    const [side, setSide] = useState('');
    

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
      console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

     }

    const handleMangoPlace = async () => {

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
  
      let fundStateInfo = await connection.getAccountInfo(fundStateAccount)
      let fundState = FUND_DATA.decode(fundStateInfo.data)
      
      let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
      let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
      let mangoAcc = await client.getMangoAccount(fundState.mango_positions.margin_account, ids.serumProgramId)
      // let mangoAcc = await client.getMangoAccount(new PublicKey('9rzuDYREjQ1UoiXgU2gJmixik5J2vSn5DoWitzKAmeJm'), ids.serumProgramId)
      // const lotSizerPrice = fundState.mango_positions.margin_account === 1 ? 10 : 100000;
      // let mangoCache = await mangoGroup.loadCache(connection)
      // let price = (mangoCache.priceCache[fundState.perp_market_index].price * lotSizerPrice)
      // let price_adj = mangoAcc.perpAccounts[fundState.perp_market_index].basePosition > 0 ? price * 0.95 : price * 1.05
  
      const transaction = new Transaction()
  
      const dataLayout = struct([u32('instruction'), ns64('quantity'), nu64('client_order_id'), u8('side'), u8('order_type')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
          {
              instruction: 8,
              // price: price_adj,
              quantity: Math.abs(mangoAcc.perpAccounts[orderPerpIndex].basePosition),
              client_order_id: 333,
              side: side,
              order_type: 0
          },
          data
      )
  
      const instruction = new TransactionInstruction({
          keys: [
              { pubkey: fundStateAccount, isSigner: false, isWritable: true },
              { pubkey: key, isSigner: true, isWritable: true },
  
              { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
              { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
              { pubkey: fundState.mango_positions.margin_account, isSigner: false, isWritable: true },
              { pubkey: fundPDA[0], isSigner: false, isWritable: true },
              { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: true },
  
              { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].publicKey), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].bidsKey), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].asksKey), isSigner: false, isWritable: true },
              { pubkey: new PublicKey(ids.perpMarkets[orderPerpIndex].eventsKey), isSigner: false, isWritable: true },
  
              { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
  
          ],
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
      return sign;
     }

     const handleAddPerpMarket = async () => {
       console.log("selected market :", addRemovePerpIndex)
     }

     const handleRemovePerpMarket = async () => {
        console.log("selected market :", addRemovePerpIndex)
      }

     const handleMangoPerpDeposit = async () => {

      const key = walletProvider?.publicKey;
      if (!key) {
          alert("connect wallet")
          return;
      };
      const transaction = new Transaction()
      const fundStateAccount = await PublicKey.createWithSeed(
          key,
          FUND_ACCOUNT_KEY,
          programId,
      );
      let fundStateInfo = await connection.getAccountInfo((fundStateAccount))
      let fundState = FUND_DATA.decode(fundStateInfo.data)
  
      let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
      let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
  
      let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[addRemoveTokenIndex].nodeKeys[0]))
      let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
  
      const dataLayout = struct([u32('instruction'),u8('token_index'), nu64('quantity')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
          {
              instruction: 6,
              token_index: addRemoveTokenIndex,
              quantity: lendAmount * 10 ** ids.tokens[addRemoveTokenIndex].decimals
          },
          data
      )
  
      const instruction = new TransactionInstruction({
          keys: [
              { pubkey: fundStateAccount, isSigner: false, isWritable: true },
              { pubkey: key, isSigner: true, isWritable: true },
              { pubkey: fundState.fund_pda, isSigner: false, isWritable: false },

              { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
              { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: false },

              { pubkey: fundState.mango_positions.margin_account , isSigner: false, isWritable: true },
              { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: false },
              { pubkey: new PublicKey(ids.tokens[addRemoveTokenIndex].rootKey), isSigner: false, isWritable: false },
              { pubkey: new PublicKey(ids.tokens[addRemoveTokenIndex].nodeKeys[0]), isSigner: false, isWritable: true },
              { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
              { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
              { pubkey: fundState.vault_key, isSigner: false, isWritable: true },
          ],
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
      return sign;
    }

  const handleMangoPerpWithdraw = async ( ) => {

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

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[addRemoveTokenIndex].nodeKeys[0]))
    let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
    const transaction = new Transaction()

    const dataLayout = struct([u32('instruction'),u8('token_index'),nu64('quantity')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
        {
            instruction: 7,
            token_index: addRemoveTokenIndex,
            quantity: lendAmount * 10 ** ids.tokens[addRemoveTokenIndex].decimals
        },
        data
    )

    const instruction = new TransactionInstruction({
        keys: [
            { pubkey: fundStateAccount, isSigner: false, isWritable: true },
            { pubkey: key, isSigner: true, isWritable: true },
            { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },

            { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
            { pubkey: fundState.mango_positions.margin_account , isSigner: false, isWritable: true },
            { pubkey: fundPDA[0], isSigner: false, isWritable: false },
            { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: false },
            { pubkey: new PublicKey(ids.tokens[addRemoveTokenIndex].rootKey), isSigner: false, isWritable: false },
            { pubkey: new PublicKey(ids.tokens[addRemoveTokenIndex].nodeKeys[0]), isSigner: false, isWritable: true },
            { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
            { pubkey: fundState.vault_key, isSigner: false, isWritable: true }, // Fund Vault
            { pubkey: mangoGroup.signerKey, isSigner: false, isWritable: true },
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
            { pubkey: PublicKey.default, isSigner: false, isWritable: true },
        ],
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
    return sign;
  }


    return (
        <div className="form-div">

        <h4>LEND TOKENS</h4>
          Amount :::  <input type="number" value={size} onChange={(event) => setLendAmount(event.target.value)} />
          <select name="side" width = "100px" onClick={(event) => setAddRemoveTokenIndex(event.target.value)}>
              {
               ids.tokens.map( (i,index) => <option value={index}>{i.symbol}</option> )
              }
            </select>
          <button onClick={handleMangoPerpDeposit}>DEPOSIT</button>
          <button onClick={handleMangoPerpWithdraw}>WITHDRAW</button>

          <br/><hr/><br/>
          <h4>ADD/REMOVE PERP</h4>
          <select name="side" width = "100px" onClick={(event) => setAddRemovePerpIndex(event.target.value)}>
              <option value={0}>MNGO-PERP</option>
              <option value={1}>BTC-PERP</option>
              <option value={2}>ETH-PERP</option>
              <option value={3}>SOL-PERP</option>
              <option value={4}>SRM-PERP</option>
              <option value={5}>RAY-PERP</option>
              <option value={6}>FTT-PERP</option>
              <option value={7}>ADA-PERP</option>
            </select>
          <button onClick={handleAddPerpMarket}>ADD</button>
          <button onClick={handleRemovePerpMarket}>REMOVE</button>

          <br/><hr/><br/>

            <h4>Mango Place</h4> Size ::: {' '}
            <input type="number" value={size} onChange={(event) => setSize(event.target.value)} />
            <br />
            <label htmlFor="side">Buy/Sell</label><br/>

            <select name="side" width = "100px" onClick={(event) => setSide(event.target.value)}>
              <option value="buy">Buy</option>
              <option value="sell">Sell</option>
            </select>

            <select name="side" width = "100px" onClick={(event) => setOrderPerpIndex(event.target.value)}>
              <option value={0}>MNGO-PERP</option>
              <option value={1}>BTC-PERP</option>
              <option value={2}>ETH-PERP</option>
              <option value={3}>SOL-PERP</option>
              <option value={4}>SRM-PERP</option>
              <option value={5}>RAY-PERP</option>
              <option value={6}>FTT-PERP</option>
              <option value={7}>ADA-PERP</option>
            </select>

          <button onClick={handleMangoPlace}>ORDER</button>

          <br />
          <button onClick={handleMangoOpenOrders}>Open order init</button>
          <br />
        </div>
    )
}