import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, SOL_USDC_MARKET, connection,  platformStateAccount, priceStateAccount, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MARGIN_ACCOUNT_KEY, ORACLE_BTC_DEVNET, ORACLE_ETH_DEVNET, ORACLE_SOL_DEVNET, ORACLE_SRM_DEVNET, idsIndex} from '../utils/constants';

import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { devnet_pools, DEV_TOKENS } from '../utils/pools';
import { keyBy } from 'lodash';
import { INVESTOR_DATA, PLATFORM_DATA, FUND_DATA } from '../utils/programLayouts';

import { updatePoolPrices } from './updatePrices';
import { MarginAccountLayout, NUM_MARKETS, MangoGroupLayout } from '../utils/MangoLayout';

import { mangoWithdrawInvestor, placeOrder, placeOrder2 } from '../utils/mango';
import { TOKENS } from '../utils/tokens';
import { IDS, MangoClient } from '@blockworks-foundation/mango-client';

import { closeAccount } from '@project-serum/serum/lib/token-instructions'

const getPoolAccounts = () => {
  return devnet_pools.map((p) => {
    return [
      { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
      { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
    ]
  })
}

export const Withdraw = () => {
  
  const ids= IDS['groups'][idsIndex];

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  
  const [amount, setAmount] = useState(0);
  const [investments, setInvestments] = useState([])
  const [investmentIndex, setInvestmentIndex] = useState(0)
  const [investorAddr, setInvestorAddr] = useState('')
  const [investorStateAcc, setInvestorStateAcc] = useState('')

  
  const handleGetAllInvestments = async () => {

    //  const userkey = new PublicKey('zRzdC1b2zJte4rMjfaSFZwbnBfL1kNYaTAF4UC4bqpx');
    let investments = await connection.getProgramAccounts(programId, { filters: [
      { dataSize: INVESTOR_DATA.span },
      {
        memcmp: { offset: INVESTOR_DATA.offsetOf('owner'), bytes: walletProvider?.publicKey.toBase58() }
      }
    ] });
    // console.log("investments::",investments)
    const newInvestors = []
    for (const investment of investments) {
      const invStateData = INVESTOR_DATA.decode(investment.account.data)
      invStateData['ivnStatePubKey'] = investment.pubkey;
    //   if (invStateData.is_initialized && invStateData.owner.toBase58() == key.toBase58()) {
        newInvestors.push(invStateData)
    //   }
    }
    console.log("newInvestors::",newInvestors)
    setInvestments(newInvestors);
  }

  const handleWithdrawSettle = async () => {

    console.log("**----handleWithdrawSettle investorStateAcc,investorAddr::",investorStateAcc,investorAddr)

    const key = walletProvider?.publicKey;
    if (!key) {
      console.log("connect wallet")
      return;
    };
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
      const fundStateAccount = await PublicKey.createWithSeed(
          key,
          FUND_ACCOUNT_KEY,
          programId,
      );
    if (!fundStateAccount) {
      console.log("no funds found")
      return
    }
  
    //getting fresh fund_state of invested fund
    const accountInfo = await connection.getAccountInfo(new PublicKey(fundStateAccount));
    const fund_data = FUND_DATA.decode(accountInfo.data);
  
      // let margin_account_1 = fund_data.mango_positions[0].margin_account;
      // let mango_info = await connection.getAccountInfo(MANGO_GROUP_ACCOUNT)
      // let mango_data = MANGO_GROUP_LAYOUT.decode(mango_info.data)
  
    const transaction = new Transaction()
  
    // let filt_pools = []
    // let WSOLWhitelisted = false;
    // for (let i = 1; i < NUM_TOKENS; i++) {
    //   if (fund_data.tokens[i].balance > 0) {
    //     let mint = platformState.token_list[fund_data.tokens[i].index[fund_data.tokens[i].mux]].mint
    //     if(mint.toBase58() === TOKENS.WSOL.mintAddress){
    //       WSOLWhitelisted=true;
    //     }
    //     if(fund_data.tokens[i].mux === 0){
    //       let x = pools.find(p => p.coin.mintAddress == mint.toBase58())
    //       filt_pools.push(x)
    //     } else {
    //       let x = orca_pools.find(p => p.coin.mintAddress == mint.toBase58())
    //       filt_pools.push(x)
    //     }
    //   }
    // }
     //send WSOL everytime 
    //  const wsol_usdc_pool = pools.find(p => p.name == 'WSOL-USDC');
    //  if(!WSOLWhitelisted){
    //    filt_pools.push(wsol_usdc_pool)
    //  }
    updatePoolPrices(transaction, devnet_pools)

    // console.log("mangoAccount:: ", mangoAccount)
    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    // let mangoAcc = await client.getMangoAccount(new PublicKey(mangoAccount), ids.serumProgramId)
    // console.log("mangoAcc DATA:: ", mangoAcc)
    let mangoGroup = await client.getMangoGroup(connection, MANGO_GROUP_ACCOUNT)


    // [      perp_market_ai,     // write default_ai if no perp market for i^th index
    //        bids_ai,            // write default_ai if no perp market for i^th index
    //        asks_ai,            // write default_ai if no perp market for i^th index
    //        event_queue_ai,]   //write default_ai if no perp market for i^th index
    let perpKeys = []
    for(let i=0; i<4;i++){
          const marketIndex = fund_data.mango_positions.perp_markets[0];
          if(marketIndex!=255){
            perpKeys.push(new PublicKey(ids.perpMarkets[marketIndex].publicKey))
            perpKeys.push(new PublicKey(ids.perpMarkets[marketIndex].bidsKey))
            perpKeys.push(new PublicKey(ids.perpMarkets[marketIndex].asksKey))
            perpKeys.push(new PublicKey(ids.perpMarkets[marketIndex].eventsKey))
          } else {
            perpKeys.push(PublicKey.default)
            perpKeys.push(PublicKey.default)
            perpKeys.push(PublicKey.default)
            perpKeys.push(PublicKey.default)
          }
    }
  
    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 4
      },
      data
    )

    const keys = [
      { pubkey: platformStateAccount, isSigner: false, isWritable: true }, //fund State Account
      { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true },
      { pubkey: fundPDA[0], isSigner: false, isWritable: true },

      { pubkey: new PublicKey(investorStateAcc), isSigner: false, isWritable: true }, //fund State Account
      { pubkey: key, isSigner: true, isWritable: true },

      { pubkey: fund_data.mango_positions.mango_account , isSigner: false, isWritable: true },
      { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
      { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: false },
      { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
       
      { pubkey: PublicKey.default, isSigner: false, isWritable: false },
       ...perpKeys, // 16 accs 
    ];

    for(let i=0; i<keys.length;i++) {
      console.log("key:",i, keys[i].pubkey.toBase58())
    }
  
    const instruction = new TransactionInstruction({
      keys: keys,
      programId,
      data
    });
    transaction.add(instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;
  
    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign);
    console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

  }

  const handleWithdrawFromMargin = async () => {
    
  }

  const handleWithdrawFromFund =  async () => {

    console.log("**----handleWithdrawFromFund investorStateAcc::",investorStateAcc)

    const key = walletProvider?.publicKey;
    if (!key) {
      console.log("connect wallet")
      return;
    };
     const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
     const fundStateAccount = await PublicKey.createWithSeed(
            key,
            FUND_ACCOUNT_KEY,
            programId,
        );
    if (!fundStateAccount) {
      console.log("no funds found")
      return
    }
    const transaction = new Transaction()

  
    // const accountInfo = await connection.getAccountInfo(new PublicKey(fundStateAccount));
    // const fund_data = FUND_DATA.decode(accountInfo.data);
  
    const RPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const investerStateAccount = new PublicKey(investorStateAcc);
    const routerAssociatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(DEV_TOKENS['USDC'].mintKey), RPDA[0], transaction);
  
    let investorStateData = await connection.getAccountInfo(investerStateAccount);
    investorStateData = INVESTOR_DATA.decode(investorStateData.data)
  
    const investorBaseTokenAccounts = [];
    const fundAssociatedTokenAddresses = []
  
    const investorTokens = investorStateData.token_indexes;
    const investorTokenDebts = investorStateData.token_debts;
  
      const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
      if(!platformDataAcc){
        alert('platform state not initilaized');
        return;
      }
      const platformState = PLATFORM_DATA.decode(platformDataAcc.data)
      let wsolTokenAccount;
  
    for (let i = 0; i < investorStateData.token_indexes.length; i++) {
  
      // if wsol then save to close account
      if (platformState.token_list[investorTokens[i]]?.mint.toBase58() === DEV_TOKENS.SOL.mintKey && investorTokenDebts[i] > 0) {
        wsolTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(DEV_TOKENS.SOL.mintKey))
      }
  
      // const invAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, platformState.token_list[investorTokens[i]].mint, key, transaction);
      investorBaseTokenAccounts.push({
        pubkey: investorTokenDebts[i] > 0 || i === 0 ? await createAssociatedTokenAccountIfNotExist(walletProvider, platformState.token_list[investorTokens[i]].mint, key, transaction) : PublicKey.default,
        isSigner: false,
        isWritable: true
      })
      // const fundAssToken = await createAssociatedTokenAccountIfNotExist(walletProvider, platformState.token_list[investorTokens[i]].mint, new PublicKey(fundPDA), transaction) ;
      fundAssociatedTokenAddresses.push({
        pubkey: investorTokenDebts[i] > 0 ? await createAssociatedTokenAccountIfNotExist(walletProvider, platformState.token_list[investorTokens[i]].mint, new PublicKey(fundPDA), transaction) : PublicKey.default,
        isSigner: false,
        isWritable: true
      })
    }
  
    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 3,
      },
      data
    )
  
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: platformStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true },
        { pubkey: investerStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: key, isSigner: true, isWritable: true },
  
        { pubkey: routerAssociatedTokenAddress, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: new PublicKey(fundPDA), isSigner: false, isWritable: false },
        { pubkey: RPDA[0], isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        ...investorBaseTokenAccounts,
        ...fundAssociatedTokenAddresses
      ],
      programId,
      data
    });
  
    transaction.add(instruction);
  
    if (wsolTokenAccount) {
      transaction.add(
        closeAccount({
          source: wsolTokenAccount,
          destination: key,
          owner: key
        })
      )
    }
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;
  
    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign);
    console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
  }

  return (
    <div className="form-div">
      <h4>WITHDRAW INVESTOR</h4>
      
      <select name="funds" width = "100px"  onChange={ (e) => setInvestmentIndex(e.target.value)}>
        {
          investments.map((i, index) => {
            return (<option key={index} value={index}>
                     {i?.ivnStatePubKey?.toBase58()} 
                      || {i?.amount?.toString()/10**6}
                    </option>)
          })
        }
      </select>
      <button onClick={handleGetAllInvestments}>Load Investments</button>
      <br />
      {
        investments && investments.length &&
        <>
                 <p > ivnStatePubKey:  {investments[investmentIndex]?.ivnStatePubKey?.toBase58()}</p>
                 <p > manager : {investments[investmentIndex]?.manager?.toBase58()}</p>
                 <p > owner : {investments[investmentIndex]?.owner?.toBase58()}</p>
                 <p> amount : {investments[investmentIndex]?.amount?.toString()/10**6}</p>
                 <p>amount_in_router : {investments[investmentIndex]?.amount_in_router?.toString()/10**6}</p>
                 <p>start_performance : {investments[investmentIndex]?.start_performance?.toString()}</p>
                 <p>is_initialized : {investments[investmentIndex]?.is_initialized}</p>
                 <p>has_withdrawn :{investments[investmentIndex]?.has_withdrawn}</p>
                 <p>withdrawn_from_margin : {investments[investmentIndex]?.withdrawn_from_margin}</p>
                 <p>margin_debt :{`${investments[investmentIndex]?.margin_debt[0]} <==>  ${investments[investmentIndex]?.margin_debt[1]}`}</p>
                 <p>margin_position_id:{`${investments[investmentIndex]?.margin_position_id[0]} <==>  ${investments[investmentIndex]?.margin_position_id[1]}`}</p>
        </>
      }

               
     
      <button onClick={handleWithdrawSettle}>withdraw_settle_1</button>
      <button onClick={handleWithdrawFromMargin}>withdraw_from_margin_2</button>
      <button onClick={handleWithdrawFromFund}>withdraw_from_fund_3</button>
      
      
    </div>
  )

}