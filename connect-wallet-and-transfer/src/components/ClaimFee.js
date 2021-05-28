import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, programId, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { findAssociatedTokenAddress, signAndSendTransaction } from '../utils/web3';
import { TEST_TOKENS } from '../utils/tokens'

export const Claim = () => {
    const [fundPDA, setFundPDA] = useState('');
    // const [fundStateAccount, setFundStateAccount] = useState('')

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleClaim = async () => {
    
        const key = walletProvider?.publicKey;

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

        if (!key) {
          alert("connect wallet")
          return;
        };

        if (fundStateAccount == ''){
          alert("get info first!")
          return
        }

        const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(TEST_TOKENS['USDP'].mintAddress));
        const managerBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(TEST_TOKENS['USDP'].mintAddress));
        const investinBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['USDP'].mintAddress), adminAccount); 
        
        const transaction = new Transaction()

        const dataLayout = struct([u8('instruction')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode(
            {
            instruction: 5,
            },
            data
        )
        
        const claim_instruction = new TransactionInstruction({
        keys: [
        {pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true},
        {pubkey: key, isSigner: true, isWritable: true },
        {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: managerBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: investinBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundPDA), isSigner: false, isWritable:true},
        {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},
    ],
    programId,
    data
    });
    
    transaction.add(claim_instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    }

    return (
        <div className="form-div">
            <h4>Withdraw</h4>
          fund pda :: {' '}
          <input type="text"
            onChange={(e) => setFundPDA(e.target.value)}
            value={fundPDA}
          />
          <br />
          {/* router pda :: {' '}
          <input type="text"
            onChange={(e) => setRouterPDA(e.target.value)}
            value={routerPDA}
          /> */}
          <button onClick={handleClaim}>Claim Performance Fee</button>
        </div>
      )
}

    