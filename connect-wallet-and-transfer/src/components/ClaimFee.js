import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, programId, TOKEN_PROGRAM_ID, FUND_ACCOUNT_KEY } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { findAssociatedTokenAddress, signAndSendTransaction, createAssociatedTokenAccountIfNotExist, commitment } from '../utils/web3';
import { TOKENS } from '../utils/tokens'
import { FUND_DATA } from '../utils/programLayouts';
import { pools } from '../utils/pools'


export const Claim = () => {
    const [fundPDA, setFundPDA] = useState('');
    const [fundStateAccount, setFundStateAccount] = useState('')
    const [performanceFee, setPerformanceFee] = useState(0)

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const handleClaim = async () => {
    
        const key = walletProvider?.publicKey;
        
        const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
        

        console.log("FUND STTE:: ", fundStateAccount)
        setFundStateAccount(fundStateAccount)

        let x = await connection.getAccountInfo(new PublicKey(fundStateAccount))
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

        const fundBaseTokenAccount = await findAssociatedTokenAddress(fundPDA[0], new PublicKey(TOKENS['USDC'].mintAddress));
        const managerBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(TOKENS['USDC'].mintAddress));
        const investinBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TOKENS['USDC'].mintAddress), adminAccount); 
        
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
        {pubkey: fundPDA[0], isSigner: false, isWritable:true},
        {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},

         // Pool Token accounts
         {pubkey: new PublicKey(pools[0].poolCoinTokenAccount), isSigner: false, isWritable: true},
         {pubkey: new PublicKey(pools[0].poolPcTokenAccount), isSigner: false, isWritable: true},
         {pubkey: new PublicKey(pools[1].poolCoinTokenAccount), isSigner: false, isWritable: true},
         {pubkey: new PublicKey(pools[1].poolPcTokenAccount), isSigner: false, isWritable: true},

    ],
    programId,
    data
    });
    
    transaction.add(claim_instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash('finalized');
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx perf: ", sign)
  }
    
  const handleGetFee = async () => {

    const key = walletProvider?.publicKey;  
    if (!key ) {
      alert("connect wallet")
      return;
    }

    const fundStateAcc = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    console.log("FUND STTE:: ", fundStateAcc.toBase58())
    setFundStateAccount(fundStateAcc.toBase58())

    let x = await connection.getAccountInfo(fundStateAcc)
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
    setPerformanceFee(parseInt(fundState.performance_fee) / (10**fundState.tokens[0].decimals))
  }
    

    return (
        <div className="form-div">
            <h4>Claim Performance Fee</h4>
         
          <br />
          <button onClick={handleClaim}>Claim Performance Fee</button>
          <button onClick={handleGetFee}>Get Claimable Fee</button>
          <br />
          Fees to claim:: {performanceFee}

        </div>
      )
}

    