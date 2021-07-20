import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, MARGIN_ACCOUNT_KEY, SOL_USDC_MARKET} from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA } from '../utils/programLayouts';
import { MarginAccountLayout, selfTradeBehaviorLayout } from '../utils/MangoLayout';
import { devnet_pools, pools } from '../utils/pools'
import { MANGO_TOKENS } from '../utils/tokens'
import { updatePoolPrices } from './updatePrices';

import {mangoClosePosition, mangoOpenPosition} from '../utils/mango'
import BN from 'bn.js';

export const MangoPlaceOrder = () => {
    const [size, setSize] = useState(0);
    const [index, setIndex] = useState(0);
    const [side, setSide] = useState('');
  
    const walletProvider = GlobalState.useState(s => s.walletProvider);

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
      const transaction = new Transaction()

      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, transaction)

      if (fundStateAccount == ''){
        alert("get info first!")
        return
      }

      await mangoOpenPosition(connection, margin_account_acc, fundStateAccount, fundPDA[0], walletProvider, index, side, size, null, transaction, false)
      console.log("transaction::", transaction)
      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)

    }

    const handleMangoClose = async () => {
        
    
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
    if (fund_data.mango_positions[0].investor_debt > 0) {
      let invs = await connection.getProgramAccounts(programId, { filters: [{ dataSize: INVESTOR_DATA.span }] });
      const invData = invs.map(f => INVESTOR_DATA.decode(f.account.data))
      let invIndex = invData.findIndex(i => ((i.manager.toBase58() == key.toBase58()) && (i.margin_debt > 0)))
      investor_accs = invs[invIndex].pubkey
      console.log("invDAta::", investor_accs)

    }

    console.log("FUND STTE:: ", fundStateAccount.toBase58())
    const transaction = new Transaction()

    const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, transaction)

    let side = fund_data.mango_positions[0].position_side == 0 ? 'sell' : 'buy'

    
    await mangoClosePosition(connection, margin_account_acc, fundStateAccount, fundPDA[0], walletProvider, index, side, size, null, transaction, investor_accs)
    console.log("transaction::", transaction)
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)

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
          <button onClick={handleMangoClose}>Mango Close Position</button>
          <br />
          <br />
        </div>
    )
}