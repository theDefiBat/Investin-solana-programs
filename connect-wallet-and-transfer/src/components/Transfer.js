import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, platformStateAccount, priceStateAccount, programId, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA, PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools'
import { TEST_TOKENS } from '../utils/tokens'
import { updatePoolPrices } from './updatePrices';

export const Transfer = () => {

  const [fundPDA, setFundPDA] = useState('')
  const [fundStateAccount, setFundStateAccount] = useState('')
  const [amountInRouter, setAmountInRouter] = useState(0);
  const [fundPerf, setFundPerf] = useState(0);
  const [fundAUM, setFundAUM] = useState(0);
  const [fundBalances, setFundBalances] = useState([])
  const [fundInvestorAccs, setFundInvestorAccs] = useState([])


  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleTransfer = async () => {
    
    const key = walletProvider?.publicKey;

      if (!key ) {
        alert("connect wallet")
        return;
      };

      const routerPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
      const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(TEST_TOKENS['USDR'].mintAddress));
      const routerBaseTokenAccount = await findAssociatedTokenAddress(routerPDA[0], new PublicKey(TEST_TOKENS['USDR'].mintAddress));

      const managerBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['USDR'].mintAddress), key);
      const investinBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['USDR'].mintAddress), adminAccount);    

      if (fundStateAccount == ''){
        alert("get info first!")
        return
      }
      
      const transaction = new Transaction()

      updatePoolPrices(transaction, devnet_pools)
      // transaction1.feePayer = walletProvider?.publicKey;
      // let hash1 = await connection.getRecentBlockhash();
      // console.log("blockhash", hash1);
      // transaction1.recentBlockhash = hash1.blockhash;

      // const sign1 = await signAndSendTransaction(walletProvider, transaction1);
      // console.log("signature tx:: ", sign1)

      const dataLayout = struct([u8('instruction')])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 2,
        },
        data
      )
      const transfer_instruction = new TransactionInstruction({
      keys: [
        {pubkey: platformStateAccount, isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true},
        {pubkey: priceStateAccount, isSigner: false, isWritable:true},
        {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable:true},

        {pubkey: key, isSigner: true, isWritable: true },
        
        {pubkey: routerBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: fundBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: managerBaseTokenAccount, isSigner: false, isWritable:true},
        {pubkey: investinBaseTokenAccount, isSigner: false, isWritable:true},

        {pubkey: routerPDA[0], isSigner: false, isWritable:true},

        {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true},
        
        //investor state accounts
        {pubkey: new PublicKey(fundInvestorAccs[0]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[1]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[2]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[3]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[4]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[5]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[6]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[7]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[8]), isSigner: false, isWritable:true},
        {pubkey: new PublicKey(fundInvestorAccs[9]), isSigner: false, isWritable:true},

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
    console.log("signature tx:: ", sign)

  }

  const handleGetFunds = async () => {

    console.log("size of plat data:: ", PLATFORM_DATA.span)
    console.log("size of fund dta : ", FUND_DATA.span)
    console.log('size of inv data:: ', INVESTOR_DATA.span)

    console.log('size of price acc:: ', PRICE_DATA.span)
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
    
    setAmountInRouter(parseInt(fundState.amount_in_router)/(10 ** fundState.tokens[0].decimals));
    setFundPerf(parseInt(fundState.prev_performance) / (10000))
    setFundAUM(parseInt(fundState.total_amount) / (10 ** fundState.tokens[0].decimals))
    
    let bal = []
    bal.push((parseInt(fundState.tokens[0].balance)/ (10**fundState.tokens[0].decimals)))
    bal.push((parseInt(fundState.tokens[1].balance)/ (10**fundState.tokens[1].decimals)))
    bal.push((parseInt(fundState.tokens[2].balance)/ (10**fundState.tokens[2].decimals)))
    setFundBalances(bal)
    console.log(bal)

    let investors = []
    for(let i=0; i<10; i++) {
      let acc =  await PublicKey.createWithSeed(
        new PublicKey(fundState.investors[i].toString()),
        fundPDA[0].toBase58().substr(0, 32),
        programId
      );
      console.log(acc.toBase58())
      investors.push(acc.toBase58())
    }
    setFundInvestorAccs(investors);
  }
  return (
    <div className="form-div">
    <h4>Transfer</h4>
      
      <button onClick={handleTransfer}>Transfer</button>
      <button onClick={handleGetFunds}>GetFundInfo</button>
      <br />
      Info for FUND: {fundPDA}
      <br />
      amount in router:: {amountInRouter}
      <br />
      Total AUM:: {fundAUM}
      <br />
      fund performance:: {fundPerf}
      <br />
      USDR balance: {fundBalances[0]}
      <br />
      RAYT balance: {fundBalances[1]}
      <br />
      ALPHA balance: {fundBalances[2]}

    </div>
  )
}
