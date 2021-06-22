import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, MARGIN_ACCOUNT_KEY} from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA } from '../utils/programLayouts';
import { MarginAccountLayout } from '../utils/MangoLayout';
import { devnet_pools, pools } from '../utils/pools'
import { MANGO_TOKENS } from '../utils/tokens'
import { updatePoolPrices } from './updatePrices';

export const MangoDeposit = () => {

  const [quantity, setQuantity] = useState(0);
  const [fundPDA, setFundPDA] = useState('')
  const [fundStateAccount, setFundStateAccount] = useState('')

  const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleMangoDeposit = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()

      const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(MANGO_TOKENS['USDC'].mintAddress));
      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, transaction)

      if (fundStateAccount == ''){
        alert("get info first!")
        return
      }

      const dataLayout = struct([u8('instruction'), nu64('quantity')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 8,
          quantity: quantity * ( 10 ** MANGO_TOKENS['USDC'].decimals)
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
            {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
            {pubkey: MANGO_VAULT_ACCOUNT_USDC, isSigner: false, isWritable:true},
            
            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable:true},
            {pubkey: CLOCK_PROGRAM_ID, isSigner: false, isWritable:true},
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
    }
    
    return (
        <div className="form-div">
            <h4>Mango Deposit</h4>
            Quantity ::: {' '}
            <input type="number" value={quantity} onChange={(event) => setQuantity(event.target.value)} />
            <br />
          <button onClick={handleMangoDeposit}>Mango Deposit</button>
          <br />
          <button onClick={handleGetFunds}>GetFundInfo</button>
          <br />
        </div>
    )
}
