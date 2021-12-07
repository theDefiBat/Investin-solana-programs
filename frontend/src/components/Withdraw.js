import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, SOL_USDC_MARKET, connection,  platformStateAccount, priceStateAccount, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MARGIN_ACCOUNT_KEY, ORACLE_BTC_DEVNET, ORACLE_ETH_DEVNET, ORACLE_SOL_DEVNET, ORACLE_SRM_DEVNET, idsIndex} from '../utils/constants';

import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { devnet_pools } from '../utils/pools';
import { keyBy } from 'lodash';
import { INVESTOR_DATA, PLATFORM_DATA, FUND_DATA } from '../utils/programLayouts';

import { updatePoolPrices } from './updatePrices';
import { MarginAccountLayout, NUM_MARKETS, MangoGroupLayout } from '../utils/MangoLayout';

import { mangoWithdrawInvestor, placeOrder, placeOrder2 } from '../utils/mango';
import { TOKENS } from '../utils/tokens';
import { IDS } from '@blockworks-foundation/mango-client';


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

  }

  const handleWithdrawFromMargin = async () => {
    
  }

  const handleWithdrawFromFund = async () => {
    
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