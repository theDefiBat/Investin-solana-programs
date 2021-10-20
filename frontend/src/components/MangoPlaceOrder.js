import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, MARGIN_ACCOUNT_KEY_1, SOL_USDC_MARKET, MARGIN_ACCOUNT_KEY_2} from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA } from '../utils/programLayouts';
import { MarginAccountLayout, selfTradeBehaviorLayout } from '../utils/MangoLayout';
import { devnet_pools, pools } from '../utils/pools'
import { MANGO_TOKENS } from '../utils/tokens'
import { updatePoolPrices } from './updatePrices';

import {mangoClosePosition, mangoOpenPosition} from '../utils/mango'
import BN from 'bn.js';
import { MangoClient } from '@blockworks-foundation/mango-client';
import { OpenOrders } from '@project-serum/serum';

export const MangoPlaceOrder = () => {
    const [size, setSize] = useState(0);
    const [index, setIndex] = useState(0);
    const [side, setSide] = useState('');
    
    const walletProvider = GlobalState.useState(s => s.walletProvider);

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
    console.log("index:: ", index)
  
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
          key.toBase58().substr(0,20) + pos_index.toString() + index.toString(),
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

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const fundStateAccount = await PublicKey.createWithSeed(
        key,
        FUND_ACCOUNT_KEY,
        programId,
      );
  
      console.log("FUND STTE:: ", fundStateAccount.toBase58())
      let fund_info = await connection.getAccountInfo(fundStateAccount);
      const fund_data = FUND_DATA.decode(fund_info.data);
      let pos_index = fund_data.mango_positions[0].state != 0 ? 1 : 0

      const transaction = new Transaction()

      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, pos_index == 0 ? MARGIN_ACCOUNT_KEY_1 : MARGIN_ACCOUNT_KEY_2, MarginAccountLayout.span, transaction)

      if (fundStateAccount == ''){
        alert("get info first!")
        return
      }

      let seed = key.toBase58().substr(0,20) + pos_index.toString()
      await mangoOpenPosition(connection, margin_account_acc, fundStateAccount, fundPDA[0], walletProvider, index, side, size, null, transaction, false, seed)
      console.log("transaction::", transaction)
      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

    }

    const handleMangoClose = async (pos_index) => {
        
    
      const key = walletProvider?.publicKey;

    if (!key ) {
      alert("connect wallet")
      return;
    };
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );
    if (fundStateAccount == ''){
      alert("get info first!")
      return
    }
    let investor_accs = PublicKey.default
    const fund_info = await connection.getAccountInfo(fundStateAccount)
    const fund_data = FUND_DATA.decode(fund_info.data)
    if (fund_data.mango_positions[0].fund_share < 1                                                                                             ) {
      let invs = await connection.getProgramAccounts(programId, { filters: [{ dataSize: INVESTOR_DATA.span }] });
      const invData = invs.map(f => INVESTOR_DATA.decode(f.account.data))
      let invIndex = invData.findIndex(i => ((i.manager.toBase58() == key.toBase58()) && (i.margin_debt[pos_index] > 0)))
      console.log("invindex: ", invIndex)
      investor_accs = invIndex >= 0 ? invs[invIndex].pubkey: PublicKey.default
      //console.log("invDAta::", investor_accs)
    }

    console.log("FUND STTE:: ", fundStateAccount.toBase58())
    const transaction = new Transaction()

    const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, pos_index == 0 ? MARGIN_ACCOUNT_KEY_1 : MARGIN_ACCOUNT_KEY_2, MarginAccountLayout.span, transaction)

    
    let side = fund_data.mango_positions[pos_index].position_side == 0 ? 'sell' : 'buy'

    console.log("side:: ", side)    
    let seed = key.toBase58().substr(0,20) + pos_index.toString()

    await mangoClosePosition(connection, margin_account_acc, fundStateAccount, fundPDA[0], walletProvider, index, side, size, null, transaction, investor_accs, seed)
    console.log("transaction::", transaction)
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

  }


    return (
        <div className="form-div">
            <h4>Mango Place</h4>
            Size ::: {' '}
            <input type="number" value={size} onChange={(event) => setSize(event.target.value)} />
            <br />
            <label htmlFor="side">Buy/Sell</label>

            <select name="side" width = "100px" onClick={(event) => setSide(event.target.value)}>
            <option value="buy">Buy</option>
            <option value="sell">Sell</option>
            </select>

            <select name="side" width = "100px" onClick={(event) => setIndex(event.target.value)}>
            <option value={0}>BTC</option>
            <option value={1}>ETH</option>
            <option value={2}>SOL</option>
            <option value={3}>SRM</option>

            </select>

          <button onClick={handleMangoPlace}>Mango Open Position</button>
          <button onClick={() => handleMangoClose(0)}>Mango Close Position 1</button>
          <button onClick={() => handleMangoClose(1)}>Mango Close Position 2</button>

          <br />
          <button onClick={handleMangoOpenOrders}>Open order init </button>
          <br />
        </div>
    )
}