import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, priceStateAccount, ORACLE_BTC_DEVNET, ORACLE_ETH_DEVNET, ORACLE_SOL_DEVNET, ORACLE_SRM_DEVNET, MARGIN_ACCOUNT_KEY} from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools'
import { MANGO_TOKENS} from '../utils/tokens'
import { MangoGroupLayout, MarginAccountLayout } from '../utils/MangoLayout';

export const MangoWithdrawToFund = () => {

  const [quantity, setQuantity] = useState(0);
  const [fundPDA, setFundPDA] = useState('')
  const [fundStateAccount, setFundStateAccount] = useState('')

  const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleMangoWithdrawToFund = async () => {
    
        const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };
      const transaction = new Transaction()

      let mango_group_acc = await connection.getAccountInfo(MANGO_GROUP_ACCOUNT)
      let mango_data = MangoGroupLayout.decode(mango_group_acc.data)

      const signer_acc = mango_data.signerKey;
      console.log("signer acc::", signer_acc)

      const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(MANGO_TOKENS['USDC'].mintAddress));
      const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, transaction)
      let x = await connection.getAccountInfo(margin_account_acc)
    if (x == null)
    {
      alert("margin account not found")
      return
    }
    console.log(x)
    let marginStateAccount = MarginAccountLayout.decode(x.data)
      if (fundStateAccount == ''){
        alert("get info first!")
        return
      }
      console.log(marginStateAccount)

      let open_order_acc = []

      open_order_acc.push(marginStateAccount.openOrders[0])
      open_order_acc.push(marginStateAccount.openOrders[1])
      open_order_acc.push(marginStateAccount.openOrders[2])
      open_order_acc.push(marginStateAccount.openOrders[3])

      const dataLayout = struct([u8('instruction'), nu64('quantity')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 12,
          quantity: quantity * ( 10 ** MANGO_TOKENS['USDC'].decimals)
        },
        data
      )
      const instruction = new TransactionInstruction({
        keys: [
            {pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true},
            // TODO: check priceStateAccount
            {pubkey: priceStateAccount, isSigner: false, isWritable: true },
            {pubkey: key, isSigner: true, isWritable: true },
            {pubkey: fundPDA, isSigner: false, isWritable: true },
            {pubkey: MANGO_PROGRAM_ID_V2, isSigner: false, isWritable:true},

            {pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable:true},
            {pubkey: margin_account_acc, isSigner: false, isWritable:true},
            {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
            {pubkey: MANGO_VAULT_ACCOUNT_USDC, isSigner: false, isWritable:true},
            // TODO: signer_acc
            {pubkey: signer_acc, isSigner: false, isWritable:true},
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
            <h4>Mango Withdraw To Fund</h4>
            Quantity ::: {' '}
            <input type="number" value={quantity} onChange={(event) => setQuantity(event.target.value)} />
            <br />
          <button onClick={handleMangoWithdrawToFund}>Mango Withdraw To Fund</button>
          <br />
          <button onClick={handleGetFunds}>GetFundInfo</button>
          <br />
        </div>
    )
}