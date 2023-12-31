import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId, platformStateAccount, FUND_ACCOUNT_KEY, TOKEN_PROGRAM_ID, idsIndex } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist, createAccountInstruction } from '../utils/web3';
import { FUND_DATA, FUND_PDA_DATA, INVESTOR_DATA, PLATFORM_DATA } from '../utils/programLayouts';
import { TOKENS } from '../utils/tokens'
import { devnet_pools } from '../utils/pools'
import { updatePoolPrices } from './updatePrices';
import {  createTokenAccountInstructions, IDS } from '@blockworks-foundation/mango-client';

export const Deposit = () => {

  const ids= IDS['groups'][idsIndex];

  const [amount, setAmount] = useState(0);
  const [fundPDA, setFundPDA] = useState('');
  const [fundStateAccount, setFundStateAccount] = useState('');
  const [funds, setFunds] = useState([]);
  const [newFunds, setNewFunds] = useState([])


  const walletProvider = GlobalState.useState(s => s.walletProvider);


  const handleDeposit = async () => {

    const key = walletProvider?.publicKey;
    console.log("**fundPDA,fundStateAccount:: ",fundPDA,fundStateAccount)
    if (!key) {
      alert("connect wallet")
      return;
    };
  

    const transaction = new Transaction()

    const RPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const FPDA = new PublicKey(fundPDA);

    const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), RPDA[0], transaction);    
    const baseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(ids.tokens[0].mintKey));

    // const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, FPDA.toBase58().substr(0, 31), INVESTOR_DATA.span, transaction)
    
    const openOrdersLamports =
    await connection.getMinimumBalanceForRentExemption(
      INVESTOR_DATA.span,
      'singleGossip'
    )
    let signers = []
    const investerStateAccount = await createAccountInstruction(connection, key, INVESTOR_DATA.span, programId, openOrdersLamports, transaction, signers);
    
    
    console.log("RPDA:", RPDA[0].toBase58())
    console.log("FPDA: ", FPDA.toBase58())
    // console.log("fundStateAccountRead:: ", fundStateAccount)
    console.log("baseTokenAccount:: ", baseTokenAccount)
    console.log("investorStateAccountRead:: ", investerStateAccount.toBase58())
    console.log("account size::: ", INVESTOR_DATA.span)
    console.log("associatedTokenaccount:: ", associatedTokenAddress1.toBase58())

    // const fundStateDataAcc = await connection.getAccountInfo(new PublicKey(fundStateAccount))
    // const fundState = FUND_DATA.decode(fundStateDataAcc.data);

   const fundPDAStateDataAcc = await connection.getAccountInfo(FPDA)
    const fundPDAState = FUND_PDA_DATA.decode(fundPDAStateDataAcc.data);

    const investors = fundPDAState.investors;
      let index=-1;
        for(let i=0; i< investors.length; i++){
            const y = investors[i].toBase58();
              if( y == PublicKey.default.toBase58()) {
                  index=i; i=1111;
              }
        }
        if(index==-1){
          throw {message : "MAX INVESTOR ACCOUNTS REACHED"};
        }
  

    const dataLayout = struct([u8('instruction'), nu64('amount'), u8('index')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1,
        amount: amount * ( 10 ** TOKENS.USDC.decimals),
        index : index
      },
      data
    )

    // DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w
    // HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o
    // HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: FPDA, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: investerStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: baseTokenAccount, isSigner: false, isWritable: true }, // Investor Base Token Account
        { pubkey: associatedTokenAddress1, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
      ],
      programId,
      data
    });
    transaction.add(instruction)
    transaction.feePayer = walletProvider?.publicKey;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;
    transaction.setSigners(key, investerStateAccount)
    transaction.partialSign(...signers)
    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

   
  }
    
  const handleFunds = async () => {
    let managers = []
    const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
    const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
    console.log("platformData :: ", platformData)

    let funds = await connection.getProgramAccounts(programId, { filters: [{ dataSize: FUND_DATA.span }] });
    console.log(`funds :::: `, funds)
    const fundData = funds.map(f => FUND_DATA.decode(f.account.data))

    console.log(`decodedFunds ::: `, fundData)
    
    for(let i=0; i<fundData.length; i++) {
      let manager = fundData[i].manager_account;
      let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
      let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);
      console.log(`PDA[0]`, PDA)
      managers.push({
        fundPDA: PDA[0].toBase58(),
        fundManager: manager.toBase58(),
        fundStateAccount: fundState.toBase58()
      });
    }
    console.log(managers)
    setFunds(managers);
    //  =============================
    const newManagers = [];
    let newFunds = await connection.getProgramAccounts(programId, { filters: [{ dataSize: FUND_PDA_DATA.span }] });
    console.log(`newFunds :::: `, newFunds)
    const fundPDAData = newFunds.map(f => FUND_PDA_DATA.decode(f.account.data))

    console.log(`fundPDAData ::: `, fundPDAData)
    
    for(let i=0; i<fundPDAData.length; i++) {
      let manager = fundPDAData[i].manager_account;
      let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
      let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);
      console.log(`PDA[0]`, PDA)
      newManagers.push({
        fundPDA: PDA[0].toBase58(),
        fundManager: manager.toBase58(),
        fundStateAccount: fundState.toBase58()
      });
    }
    console.log("newManagers ::",newManagers)
    setNewFunds(newManagers);
  }

  const handleFundSelect = async(event) => {
  
    console.log("seleecting fund ",event.target.value ,event.target ,event)

    console.log("seleecting fund ",event.target.value )
    setFundPDA(event.target.value);
    alert("seleecting fund ",event.target.value )
    console.log(`setting fundPDA :::: `, fundPDA)


    // funds.forEach(fund => {
    //   if (fund.fundPDA == event.target.value) 
    //   {
    //     setFundStateAccount(fund.fundStateAccount)
    //    console.log("set fundStateAcoount:",fund.fundStateAccount)
    //   }
    // });
    
    // console.log(`setting fundStateAccount :::: `, fundStateAccount)
  }

  return (
    <div className="form-div">
      <h4>Investor Fund Deposit</h4>
      amount ::: {' '}
      <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
      <br />
      <label htmlFor="funds">Select Fund Address:</label>


      {/* <select name="funds" width = "100px"  onChange={handleFundSelect}>
      <option key={-1} value={0}>NONE</option>
            { funds && 
                funds.map((fund,index) => {
                    return (<option key={index} value={fund.fundPDA}>{fund.fundPDA}</option>)
                })
            }
      </select>
      <br/> */}
      <select name="funds" width = "100px"  onChange={handleFundSelect}>
      <option key={-1} value={0}>NONE</option>
            { newFunds && 
                newFunds.map((fund,index) => {
                    return (<option key={index} value={fund.fundPDA}>{fund.fundPDA}</option>)
                })
            }
      </select>
      <button onClick={handleDeposit}>Deposit</button>
      <button onClick={handleFunds}>Load  Funds</button>
      
    </div>
  )
}
