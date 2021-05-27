import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount } from '../utils/web3';

export const Transfer = () => {

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleTransfer = async () => {
    
        const key = walletProvider?.publicKey;

        if (!key) {
          alert("connect wallet")
          return;
        };

        const [fundPDA, setFundPDA] = useState('');
        const [routerPDA, setRouterPDA] = useState('');



        const clientAccount = await connection.getAccountInfo(key);
        
        const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'));
        const routerBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(routerPDA), new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'));
        const managerBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'));

        const transaction = new Transaction()

        const dataLayout = struct([u8('instruction')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
            instruction: 2,
            },
            data
        )
        ss
        const transfer_instruction = new TransactionInstruction({
        keys: [
        {pubkey: new PublicKey("A6R1siU7eCLiTFTswtofYeUtaRCPWn65yT4VhfY1vPEK"), isSigner: false, isWritable: true},
        { pubkey: key, isSigner: true, isWritable: true },
        {pubkey: routerBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: managerBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: new PublicKey(routerPDA), isSigner: false, isWritable:true},

        {pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: true},
        
        // Pool Token accounts
        {pubkey: new PublicKey("DUn4i71SXksHN7KtveP4uauWqsnfdSHa4PoEkzN8qqN6"), isSigner: false, isWritable: true},
        {pubkey: new PublicKey("BUdDS4AUMSsvQ1QyHe4LLagvkFfUU4TW17udvxaDJaxR"), isSigner: false, isWritable: true},
        {pubkey: new PublicKey("2Ab9oAp9XcarKgdthdAtTitAHctuEkafKHh2GtzSJRyt"), isSigner: false, isWritable: true},
        {pubkey: new PublicKey("BUdDS4AUMSsvQ1QyHe4LLagvkFfUU4TW17udvxaDJaxR"), isSigner: false, isWritable: true},
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

    }
    return (
        <div>
          <br />
          fund pda :: {' '}
          <input type="text"
            onChange={(e) => setFundPDA(e.target.value)}
            value={fundPDA}
          />
          <br />
          router pda :: {' '}
          <input type="text"
            onChange={(e) => setRouterPDA(e.target.value)}
            value={routerPDA}
          />
          <button onClick={handleTransfer}>Transfer</button>
        </div>
      )
}