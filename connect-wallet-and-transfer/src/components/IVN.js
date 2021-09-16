import React, { useState } from 'react'
import { createAssociatedTokenAccountIfNotExist, findAssociatedTokenAddress, signAndSendTransaction } from '../utils/web3'
import { connection } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct } from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';

export const ivnProgId = new PublicKey('DNZb9BgKyXdCyqXQUaH8Na4Labko15M8RTgCfKyGUNPW')

export const ABIVN_TOKEN = {
    symbol: 'ABIVN',
    name: 'allBridge IVN',
    mintAddress: '2ZfTbMeJqfTRR2wmMYCbT2W9tkEwEZwf4u5x5LMiCsj1',
    decimals: 9
}
export const IVN_TOKEN = {
    symbol: 'IVN',
    name: 'Investin Token',
    mintAddress: 'iVNcrNE9BRZBC9Aqf753iZiZfbszeAVUoikgT9yvr2a',
    decimals: 6
  }

export const IVN = () => {
    const [amount, setAmount] = useState(0);

    const walletProvider = GlobalState.useState(s => s.walletProvider);
  
    const handleIVNSwap = async () => {
        const transaction = new Transaction()
        const key = walletProvider.publicKey
        console.log("publick:: ", key)
        let pdaAcc = await PublicKey.findProgramAddress(
            [Buffer.from('swap')],
            ivnProgId
          )
        console.log("PDA:: ", pdaAcc[0].toBase58())

        const dataLayout = struct([nu64('amount')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
          {
            amount: amount * 10**ABIVN_TOKEN.decimals
          },
          data
        )
      
        const abivn_token_acc = await findAssociatedTokenAddress(key, new PublicKey(ABIVN_TOKEN.mintAddress))
        const abivn_vault_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ABIVN_TOKEN.mintAddress), pdaAcc[0], transaction)

        const ivn_token_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(IVN_TOKEN.mintAddress), key, transaction)
        const ivn_vault_acc = await findAssociatedTokenAddress(pdaAcc[0], new PublicKey(IVN_TOKEN.mintAddress))
        
        const instruction = new TransactionInstruction({
          keys: [
            {pubkey: pdaAcc[0], isSigner: false, isWritable: false},
            
            {pubkey: abivn_token_acc, isSigner: false, isWritable: true},
            {pubkey: abivn_vault_acc, isSigner: false, isWritable: true},
            
            {pubkey: ivn_vault_acc, isSigner: false, isWritable: true},
            {pubkey: ivn_token_acc, isSigner: false, isWritable: true},
            
            {pubkey: key, isSigner: true, isWritable: true},
            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
          ],
          programId: ivnProgId,
          data: data
        });

        transaction.add(instruction)
        transaction.feePayer = walletProvider?.publicKey;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", transaction);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("signature tx:: ", sign)
    }

    return (
        <div className="form-div">
        <h4>abIVN => IVN</h4>
        amount ::: {' '}
        <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
        <br />
        <button onClick={handleIVNSwap}>Swap</button>
        </div>
    )
}