import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, priceStateAccount, connection, programId, TOKEN_PROGRAM_ID, FUND_ACCOUNT_KEY, idsIndex, SYSTEM_PROGRAM_ID, platformStateAccount } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { findAssociatedTokenAddress, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { TEST_TOKENS } from '../utils/tokens'
import { FUND_DATA, FUND_PDA_DATA } from '../utils/programLayouts';
import { devnet_pools } from '../utils/pools'
import { updatePoolPrices } from './updatePrices';


export const MigrateState = () => {


    const [fundPDA, setFundPDA] = useState('');
    // const [fundStateAccount, setFundStateAccount] = useState('');

    const walletProvider = GlobalState.useState(s => s.walletProvider);

    const [funds, setFunds] = useState([])
    const [oldFunds, setOldFunds] = useState([])

    const handleGetAllMigratedFunds = async () => {
      const managers = []
      const allFunds = await connection.getProgramAccounts(programId, { filters: [
        { dataSize: FUND_PDA_DATA.span },
        //  {
        //   memcmp: { offset: FUND_PDA_DATA.offsetOf('number_of_active_investments'), bytes: '3' }
        // }
      ] });
      console.log("allFunds::",allFunds)
      for (const data of allFunds) {
          const decodedData = FUND_PDA_DATA.decode(data.account.data);
          const PDA_balance  = await connection.getBalance(decodedData.fund_pda, "max");
          console.log("PDA_balance:",PDA_balance)
  
         
              managers.push({
                  fund_v3_index : decodedData.fund_v3_index,
                  fundState : decodedData,
                  fundPDA: decodedData.fund_pda.toBase58(),
                  fundManager: decodedData.manager_account.toBase58(),
                  fundStateAccount: data.pubkey.toBase58(),
                  PDA_balance : PDA_balance,
                  // fundName: decodedData.fund_pda.toBase58(),
                  // totalAmount: (new TokenAmount(decodedData.total_amount, ids.tokens[0].decimals)).toEther().toNumber(),
              });
      }
      console.log("-----PDA funds:",managers);
  
      setFunds(managers);
    }

    const handleGetAllNonMigratedFunds = async () => {
      const managers = []
      const allFunds = await connection.getProgramAccounts(programId, { filters: [
        { dataSize: FUND_DATA.span },
        //  {
        //   memcmp: { offset: FUND_PDA_DATA.offsetOf('number_of_active_investments'), bytes: '3' }
        // }
      ] });
      console.log("All OLD FUND_STATE Funds::",allFunds)
    
      for (const data of allFunds) {
           const decodedData = FUND_DATA.decode(data.account.data);
  
           const PDA_balance  = await connection.getBalance(decodedData.fund_pda, "max");
           console.log("PDA_balance:",PDA_balance)

          if (decodedData.is_initialized) {
              managers.push({
                  fund_v3_index : decodedData.fund_v3_index,
                  fundState : decodedData,
                  fundPDA: decodedData.fund_pda.toBase58(),
                  fundManager: decodedData.manager_account.toBase58(),
                  fundStateAccount: data.pubkey.toBase58(),
                  PDA_balance : PDA_balance,
                  // fundName: decodedData.fund_pda.toBase58(),
                  // totalAmount: (new TokenAmount(decodedData.total_amount, ids.tokens[0].decimals)).toEther().toNumber(),
              });
          }
      }
      console.log("OLD funds:",managers);  
      setOldFunds(managers);
    }


  const handleMigrate = async () => {

      if(funds.length==0 || oldFunds.length==0){
        alert("first get funds")
        return;
      }
    
        const key = walletProvider?.publicKey;
        // const managerPubkey = new PublicKey('Gyj8YbcA1vENNzhYdtGmJCFi4DELeUCEjhhontbmqZQk');
        // const fundPDA = await PublicKey.findProgramAddress([managerPubkey.toBuffer()], programId);
        // const fundStateAccount = await PublicKey.createWithSeed(
        //   managerPubkey,
        //   FUND_ACCOUNT_KEY,
        //   programId,
        // );
        // let x = await connection.getAccountInfo(fundStateAccount)
        // if (x == null)
        // {
        //   alert("fund account not found")
        //   return
        // }
        // if (fundStateAccount == ''){
        //   alert("get info first!")
        //   return
        // }

        if (!key) {
          alert("connect wallet")
          return;
        };

        
      
        const fundStateAndPDAS = [
          // 1
          {pubkey: new PublicKey('CRpFgJZU1edZMck4ek32CyZ3sRucgVfwCmkrvk88kHSq'), isSigner: false, isWritable: true },
          {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // 2
          {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 3
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 4
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 5
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 6
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 7
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 8
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 9
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // // 10
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
          // {pubkey: new PublicKey(''), isSigner: false, isWritable: true },
        ]

        const transaction = new Transaction()
        const dataLayout = struct([u8('instruction'),u8('count')])
        const data = Buffer.alloc(dataLayout.span)
        dataLayout.encode({
          instruction: 23,
          count : 2
        },data)

        
        
        const keys = [
        {pubkey: platformStateAccount, isSigner: false, isWritable: true },
        {pubkey: key, isSigner: true, isWritable: true },
        {pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: true},

        // {pubkey: fundPDA[0], isSigner: false, isWritable:true},
        // {pubkey: fundStateAccount, isSigner: false, isWritable: true},
        ...fundStateAndPDAS
      ]

      for(let i=0; i<keys.length;i++) {
        console.log("key:",i, keys[i].pubkey.toBase58())
      }
        
        const migrate_instruction = new TransactionInstruction({
          keys,
          programId,
          data
        });
    
        transaction.add(migrate_instruction);
        transaction.feePayer = key;
        let hash = await connection.getRecentBlockhash();
        console.log("blockhash", hash);
        transaction.recentBlockhash = hash.blockhash;

        const sign = await signAndSendTransaction(walletProvider, transaction);
        console.log("tx perf: ", sign)
        console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
  }
 


    return (
        <div className="form-div">
            <h4>Migrate State </h4>
         
          <br />
          <button onClick={handleMigrate}>Migrate</button>
          <br />
          New State :: {}

        </div>
      )
}

    