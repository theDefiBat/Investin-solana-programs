import React, { useState } from 'react'
import { createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, findAssociatedTokenAddress, signAndSendTransaction } from '../utils/web3'
import { CLOCK_PROGRAM_ID, connection } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, u8, struct } from 'buffer-layout';
import * as BufferLayout from "buffer-layout";

import { PublicKey, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { publicKeyLayout, u64 } from '../utils/programLayouts';
import { TOKENS } from '../utils/tokens';


export const ESCROW_ACCOUNT_DATA_LAYOUT = BufferLayout.struct([
    BufferLayout.u8("isInitialized"),
    publicKeyLayout("initializerPubkey"),
    publicKeyLayout("initializerTempTokenAccountPubkey"),
    publicKeyLayout("initializerReceivingTokenAccountPubkey"),
    u64("expectedAmount"),
  ]);


export const ivnProgId = new PublicKey('DZDE83B6wZcCpW4ZtwaNyURuytK8JLPBmeRXn3qWjE1m')

export const IVN_TOKEN = {
    symbol: 'IVN',
    name: 'Investin Token',
    mintAddress: 'iVNcrNE9BRZBC9Aqf753iZiZfbszeAVUoikgT9yvr2a',
    decimals: 6
  }

export const Escrow = () => {
    const [amount, setAmount] = useState(0);
    const [expected, setExpected] = useState(0);


    const walletProvider = GlobalState.useState(s => s.walletProvider);
  
    const handleInitEscrow = async() => {
        const transaction = new Transaction()
        const key = walletProvider.publicKey

        let pdaAcc = await PublicKey.findProgramAddress(
            [Buffer.from('escrow')],
            ivnProgId
        )

        console.log("PDA:: ", pdaAcc[0].toBase58())
        const ivn_token_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(IVN_TOKEN.mintAddress), key, transaction)
        const usdc_token_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TOKENS.USDC.mintAddress), key, transaction)

        const ivn_vault_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(IVN_TOKEN.mintAddress), pdaAcc[0], transaction)


        const escrowAccount = await createKeyIfNotExists(walletProvider, "", ivnProgId, "ESCROWED", ESCROW_ACCOUNT_DATA_LAYOUT.span, transaction)

        const dataLayout = struct([u8('instruction'), nu64('amount'), nu64('expected_amount')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
          {
            instruction: 0,
            amount: amount * 10**IVN_TOKEN.decimals,
            expected_amount: expected * 10**TOKENS.USDC.decimals
          },
          data
        )
       
        const instruction = new TransactionInstruction({
          keys: [
            {pubkey: key, isSigner: true, isWritable: true},
            {pubkey: ivn_token_acc, isSigner: false, isWritable: true},
            {pubkey: ivn_vault_acc, isSigner: false, isWritable: true},

            {pubkey: usdc_token_acc, isSigner: false, isWritable: true},
            {pubkey: escrowAccount, isSigner: false, isWritable: true},

            {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
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

        console.log("wallet:: ", walletProvider)
        let signedTrans = await walletProvider.signTransaction(transaction);
        console.log("sign transaction");
        let sign = await connection.sendRawTransaction(signedTrans.serialize());
        // const sign = await walletProvider.signTransaction(transaction)
        console.log("signature tx:: ", sign)

    }
    const handleTrade = async () => {
        const transaction = new Transaction()
        const key = walletProvider.publicKey
        let pdaAcc = await PublicKey.findProgramAddress(
            [Buffer.from('escrow')],
            ivnProgId
        )

        console.log("PDA:: ", pdaAcc[0].toBase58())
        const usdc_token_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TOKENS.USDC.mintAddress), key, transaction)
        const ivn_token_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(IVN_TOKEN.mintAddress), key, transaction)

        
        const ivn_vault_acc = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(IVN_TOKEN.mintAddress), pdaAcc[0], transaction)

        // const escrowAccount = await createKeyIfNotExists(walletProvider, "", ivnProgId, "ESCROW", ESCROW_ACCOUNT_DATA_LAYOUT.span, transaction)

        const escrowAccount = new PublicKey('BRG2XNe5AKfEBntQoVJWRgPAYwv7877X8373tiFfTHpF')

        let escrowInfo = await connection.getAccountInfo(escrowAccount)
        let escrow_data = ESCROW_ACCOUNT_DATA_LAYOUT.decode(escrowInfo.data)

        console.log("escrow data:: ", escrow_data)

        const dataLayout = struct([u8('instruction')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
          {
            instruction: 1,
          },
          data
        )
       
        const instruction = new TransactionInstruction({
          keys: [
            {pubkey: key, isSigner: true, isWritable: true},
            {pubkey: usdc_token_acc, isSigner: false, isWritable: true},
            {pubkey: ivn_token_acc, isSigner: false, isWritable: true},

            {pubkey: escrow_data.initializerTempTokenAccountPubkey, isSigner: false, isWritable: true},
            {pubkey: escrow_data.initializerPubkey, isSigner: false, isWritable: true},
            {pubkey: escrow_data.initializerReceivingTokenAccountPubkey, isSigner: false, isWritable: true},

            {pubkey: escrowAccount, isSigner: false, isWritable: true},

            {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
            {pubkey: pdaAcc[0], isSigner: false, isWritable: false},

          ],
          programId: ivnProgId,
          data: data
        });

        transaction.add(instruction)
        transaction.feePayer = walletProvider?.publicKey;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", transaction);
        transaction.recentBlockhash = hash.blockhash;

        console.log("wallet:: ", walletProvider)
        let signedTrans = await walletProvider.signTransaction(transaction);
        console.log("sign transaction");
        let sign = await connection.sendRawTransaction(signedTrans.serialize());
        // const sign = await walletProvider.signTransaction(transaction)
        console.log("signature tx:: ", sign)

    }

    return (
        <div className="form-div">
        <h4>Sale</h4>
        amount IVN ::: {' '}
        <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
        <br />
        expectedAmount USDC ::: {' '}
        <input type="number" value={expected} onChange={(event) => setExpected(event.target.value)} />
        <br />
        <button onClick={handleInitEscrow}>Init Escrow</button>
        <br />
        <button onClick={handleTrade}>Execute trade</button>
        </div>
    )
}