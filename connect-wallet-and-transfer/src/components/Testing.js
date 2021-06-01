import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, platformStateAccount, programId, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA } from '../utils/programLayouts';
import { devnet_pools } from '../utils/pools'
import { TEST_TOKENS } from '../utils/tokens'
import { u64 } from '@project-serum/borsh';

export const Testing = () => {

    const [amount, setAmount] = useState(0);

    const walletProvider = GlobalState.useState(s => s.walletProvider);
    
    const handleDeposit = async () => {

        const key = walletProvider?.publicKey;
    
        if (!key) {
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

        const fundBaseTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(TEST_TOKENS['USDR'].mintAddress));
        const managerBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(TEST_TOKENS['USDR'].mintAddress));
        console.log("amount deposit: ", amount)

        const dataLayout = struct([u8('instruction'), nu64('amount')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
            instruction: 6,
            amount: amount * (10 ** TEST_TOKENS['USDR'].decimals),
            },
            data
        )
        const instruction = new TransactionInstruction({
        keys: [
            {pubkey: fundStateAccount, isSigner: false, isWritable:true},
            {pubkey: key, isSigner: true, isWritable: true},
        
            {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
            {pubkey: managerBaseTokenAccount, isSigner: false, isWritable:true},
 
            {pubkey: fundPDA[0], isSigner: false, isWritable:true},

            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},
            
            // Pool Token accounts
            {pubkey: new PublicKey(devnet_pools[0].poolCoinTokenAccount), isSigner: false, isWritable: true},
            {pubkey: new PublicKey(devnet_pools[0].poolPcTokenAccount), isSigner: false, isWritable: true},
            {pubkey: new PublicKey(devnet_pools[1].poolCoinTokenAccount), isSigner: false, isWritable: true},
            {pubkey: new PublicKey(devnet_pools[1].poolPcTokenAccount), isSigner: false, isWritable: true},

        ],
        programId,
        data
        });
        
        const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
        const signature = await signAndSendTransaction(walletProvider, transaction2);
        let result = await connection.confirmTransaction(signature, "confirmed");
        console.log("tx:: ", signature)
    }

    const handleWithdraw = async () => {

        const key = walletProvider?.publicKey;
    
        if (!key) {
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

        const fundBaseTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(TEST_TOKENS['USDR'].mintAddress));
        const managerBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(TEST_TOKENS['USDR'].mintAddress));
        
        console.log("amount withdraww: ", amount)
        const dataLayout = struct([u8('instruction'), nu64('amount')])

        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
            instruction: 7,
            amount: amount * (10 ** TEST_TOKENS['USDR'].decimals),
            },
            data
        )
        const instruction = new TransactionInstruction({
        keys: [
            {pubkey: fundStateAccount, isSigner: false, isWritable:true},
            {pubkey: key, isSigner: true, isWritable: true},
        
            {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
            {pubkey: managerBaseTokenAccount, isSigner: false, isWritable:true},
 
            {pubkey: fundPDA[0], isSigner: false, isWritable:true},

            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},
            
            // Pool Token accounts
            {pubkey: new PublicKey(devnet_pools[0].poolCoinTokenAccount), isSigner: false, isWritable: true},
            {pubkey: new PublicKey(devnet_pools[0].poolPcTokenAccount), isSigner: false, isWritable: true},
            {pubkey: new PublicKey(devnet_pools[1].poolCoinTokenAccount), isSigner: false, isWritable: true},
            {pubkey: new PublicKey(devnet_pools[1].poolPcTokenAccount), isSigner: false, isWritable: true},

        ],
        programId,
        data
        });
        
        const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
        const signature = await signAndSendTransaction(walletProvider, transaction2);
        let result = await connection.confirmTransaction(signature, "confirmed");
        console.log("tx:: ", signature)

    }
    return (
        <div className="form-div">
        <h4>Testing</h4>
            amount ::: {' '}
            <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
            <br />
            <button onClick={handleDeposit}>Deposit</button>
            <button onClick={handleWithdraw}>Withdraw</button>
    
        </div>
      )
}