import { PublicKey, Transaction, TransactionInstruction, create} from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId, platformStateAccount, FUND_ACCOUNT_KEY, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u32 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, signAndSendTransaction, createAssociatedTokenAccountIfNotExist, createAccountInstruction } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA } from '../utils/programLayouts';
import { awaitTransactionSignatureConfirmation, IDS, MangoClient } from '@blockworks-foundation/mango-client';
import { sendSignedTransactionAndNotify } from '../utils/solanaWeb3';

export const ProcessDeposits = () => {

  const [selectedInvestmentStateAcc, setSelectedInvestmentStateAcc] = useState('');
  const [investments, setInvestments] = useState([]);

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const ids = IDS['groups'][0]

  
  const handleprocesDeposit = async () => {

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId)[0];

    console.log('selected investment::', selectedInvestmentStateAcc)
  
    let fundStateInfo = await connection.getAccountInfo(new PublicKey(fundPDA))
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)

    const transaction = new Transaction()
  
    const openOrdersLamports = await connection.getMinimumBalanceForRentExemption(
          INVESTOR_DATA.span,
          'singleGossip'
        )
    let signers = [];
    
    const investorBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), key, transaction);

    console.log("account size::: ", INVESTOR_DATA.span)

    const dataLayout = struct([u32('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 2,
      },
      data
    )
    const keys =  [
      { pubkey: new PublicKey(fundPDA), isSigner: false, isWritable: true }, //fund State Account
      // { pubkey: investerStateAccount, isSigner: false, isWritable: true },
      { pubkey: key, isSigner: true, isWritable: true },
      { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true }, // Investor Base Token Account
      { pubkey: fundState.usdc_vault_key, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    ];

    for(let i = 0; i<keys.length; i++){
      console.log('>>',i, keys[i].pubkey.toBase58())
    }


    const instruction = new TransactionInstruction({
      keys,
      programId,
      data
    });

   
    transaction.add(instruction)
    transaction.feePayer = walletProvider?.publicKey;
    let hash = await connection.getRecentBlockhash();
    console.log("tx", transaction);
    transaction.recentBlockhash = hash.blockhash;
    transaction.setSigners(key);
    transaction.partialSign(...signers)

    // const sign = await signAndSendTransaction(walletProvider, transaction);
    // console.log("signature tx:: ", sign)
    // await awaitTransactionSignatureConfirmation(sign, 120000, connection, 'finalized')
   

      try {
          await sendSignedTransactionAndNotify({
              connection,
              transaction: transaction,
              successMessage: "Investment successful",
              failMessage: "Investment unsuccessful",
              wallet: walletProvider
          })
      } catch (error) {
          console.error('handleMakeInvestment: ', error);
      }

  }
    
  const handleGetInvestors = async () => {

    const fundPDA = (await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId))[0];
    console.log("fundPDA::",fundPDA.toBase58())

    let investments = await connection.getProgramAccounts(programId, { 
      filters: [
        {
          memcmp : { offset : INVESTOR_DATA.offsetOf('fund') , bytes : fundPDA.toString()},
          // memcmp : { offset : INVESTOR_DATA.offsetOf('investment_status') , bytes : 0}
        },
        { dataSize: INVESTOR_DATA.span }
      ]
     });
    console.log(`found investments :::: `, investments)

    const investmentStateAccs = investments.map(f => f.pubkey.toBase58())

    const investmentsData = investments.map(f => INVESTOR_DATA.decode(f.account.data))
    console.log(`decodedFunds ::: `, investmentsData)
    
    // for(let i=0; i<investments.length; i++) {
    //   let fund = investmentsData[i].fund;
    //   let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);
    //   console.log(`PDA[0]`, PDA)
    //   managers.push({
    //     fundPDA: PDA[0].toBase58(),
    //     fundManager: manager.toBase58(),
    //   });
    // }
    // console.log(managers)
    setInvestments(investmentStateAccs);
  }

  const handleSelectInvestment = async(event) => {
    setSelectedInvestmentStateAcc(event.target.value);
    console.log(`setting selectedInvestmentStateAcc :::: `,event.target.value, selectedInvestmentStateAcc)
  }

  return (
    <div className="form-div">
      <h4>Process Deposit</h4>
      
      <br />
      <label htmlFor="funds">Select Investment Address:</label>

      <select name="funds" width = "100px" onClick={handleSelectInvestment}>
        {
          investments.map((i) => {
            return (<option key={i} value={i}>{i}</option>)
          })
        }
      </select>
      <button onClick={handleprocesDeposit}> Process Deposit</button>
      <button onClick={handleGetInvestors}>Load Investments of my fund</button>
    </div>
  )
}
