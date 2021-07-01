import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, MARGIN_ACCOUNT_KEY, ORACLE_BTC_DEVNET, ORACLE_ETH_DEVNET, ORACLE_SOL_DEVNET, ORACLE_SRM_DEVNET} from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, signAndSendTransaction } from '../utils/web3';
import { MarginAccountLayout } from '../utils/MangoLayout';
import { MANGO_TOKENS } from '../utils/tokens'

export const MangoWithdrawInvestor = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleMangoWithdrawInvestor = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
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

      const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(MANGO_TOKENS['USDC'].mintAddress));
      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, transaction)
      const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, MPDA.toBase58().substr(0, 32), INVESTOR_DATA.span)

      console.log("margin_acc::", margin_account_acc)
      let margin_data = await connection.getAccountInfo(margin_account_acc);
      let margin_dec = MarginAccountLayout.decode(margin_data.data)
      console.log("margin data", margin_dec)

      if (fundStateAccount == ''){
        alert("get fund info first!")
        return
      }

      let x = await connection.getAccountInfo(margin_account_acc)
        if (x == null)
        {
            alert("margin account not found")
            return
        }
        console.log(x)
        let marginStateAccount = MarginAccountLayout.decode(x.data)

      let open_order_acc = []
      open_order_acc.push(marginStateAccount.openOrders[0])
      open_order_acc.push(marginStateAccount.openOrders[1])
      open_order_acc.push(marginStateAccount.openOrders[2])
      open_order_acc.push(marginStateAccount.openOrders[3])

      const dataLayout = struct([u8('instruction'), nu64('quantity')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 13,
        //   TODO: 
        //   token_index:
        },
        data
      )
      const instruction = new TransactionInstruction({
        keys: [
            {pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true},
            {pubkey: investerStateAccount, isSigner: false, isWritable: true },
            {pubkey: fundPDA, isSigner: false, isWritable: true },
            {pubkey: key, isSigner: true, isWritable: true },
            {pubkey: MANGO_PROGRAM_ID_V2, isSigner: false, isWritable:true},

            {pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable:true},
            {pubkey: margin_account_acc, isSigner: false, isWritable:true},
            {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
            {pubkey: MANGO_VAULT_ACCOUNT_USDC, isSigner: false, isWritable:true},
            // TODO: signer_acc
            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable:true},
            {pubkey: CLOCK_PROGRAM_ID, isSigner: false, isWritable:true},
            // open order account
            {pubkey: open_order_acc[0], isSigner: false, isWritable:true},
            {pubkey: open_order_acc[1], isSigner: false, isWritable:true},
            {pubkey: open_order_acc[2], isSigner: false, isWritable:true},
            {pubkey: open_order_acc[3], isSigner: false, isWritable:true},
            // oracle accounts
            {pubkey: ORACLE_BTC_DEVNET, isSigner: false, isWritable:true},
            {pubkey: ORACLE_ETH_DEVNET, isSigner: false, isWritable:true},
            {pubkey: ORACLE_SOL_DEVNET, isSigner: false, isWritable:true},
            {pubkey: ORACLE_SRM_DEVNET, isSigner: false, isWritable:true},
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
    
    return (
        <div className="form-div">
            <h4>Mango Withdraw Investor</h4>
            <br />
          <button onClick={handleMangoWithdrawInvestor}>Mango Withdraw Investor</button>
          <br />
        </div>
    )
}
