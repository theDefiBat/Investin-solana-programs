import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import React, { useEffect, useState } from 'react'
import { GlobalState } from '../store/globalState';

import { adminAccount, connection, FUND_ACCOUNT_KEY, platformStateAccount, priceStateAccount, programId } from '../utils/constants';
import { blob, nu64, struct, u32, u8 } from 'buffer-layout';
import { FUND_DATA } from '../utils/programLayouts';

import { Card, Col, Row ,Table} from 'reactstrap';


export const DisplayInfo = (props) => {

  const [fundData, setFundData] = useState("");
  const [fundTokens, setFundTokens] = useState([]);

//   const walletProvider = GlobalState.useState(s => s.walletProvider);
const walletProvider = GlobalState.useState(s => s.walletProvider);
  
const programIdX = programId.toBase58();
const adminAccountX = adminAccount.toBase58();
const platformStateAccountX = platformStateAccount.toBase58();
const priceStateAccountX = priceStateAccount.toBase58();

const handleGetFundData = async () => {

  if(!walletProvider) {
    alert("connect wallet ")
    return;
  }

    const key = new PublicKey('ysh2B9XKTbX8DdsWM3HrzjuZ5otrFc24Y6H8naAmoYi');
  // const key = walletProvider?.publicKey;  
  if (!key ) {
    alert("connect wallet ")
    return;
  }

    const fundStateAcc = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );
    console.log("FUND fundStateAcc:: ", fundStateAcc.toBase58())
    // setFundStateAccount(fundStateAcc.toBase58())

    const fundDataAcc = await connection.getAccountInfo(fundStateAcc);
    console.log("fundDataAcc::",fundDataAcc);
    if (fundDataAcc == null)
    {
       alert("fundDataAcc info not found")
      return;
    }
    const fundData = FUND_DATA.decode(fundDataAcc.data)
    console.error("fundData::",fundData);
    setFundData(fundData);

    // display fundState
    let fundStateTokens = fundData?.tokens.map((i)=> {
      return {
        balance : i.balance.toString(),
        debt : i.debt.toString(),
        vault : i.vault.toBase58(),
        index : i.index,
        is_initialized : i.is_initialized,
        mux : i.mux,
        padding : i.padding
      }
    });
    setFundTokens(fundStateTokens);
    console.error("parsed fundState tokens: ",fundStateTokens);
}

// useEffect(() => {

// }, [walletProvider])

  

  return (
    <div className="form-div">
    <h4>Accounts</h4>
      <p> programID : {programIdX}</p>
      <p> adminAccount : {adminAccountX}</p>
      <p> platformStateAccount : {platformStateAccountX}</p>
      <p> priceStateAccount : {priceStateAccountX}</p>
      <button onClick={handleGetFundData}>GET FUND STATE</button>
   

      {
        fundData &&
          <>
            <h4>FUND STATE</h4>
            <p> number_of_active_investments : {fundData.number_of_active_investments}</p>
            <p> no_of_investments : {fundData.no_of_investments}</p>

            <p> no_of_margin_positions : {fundData.no_of_margin_positions}</p>
            <p> no_of_assets (active tokens) : {fundData.no_of_assets}</p>
            <p> position_count  : {fundData.position_count}</p>
            <p> version  : {fundData.version}</p>
            <p> padding   : u8,7</p>

            <p> min_amount  : {fundData.min_amount.toString()}</p>
            <p> min_return  : {fundData.min_return.toString()}</p>
            <p> performance_fee_percentage  : {fundData.performance_fee_percentage}</p>
            <p> total_amount in fund USDC  : {fundData.total_amount.toString()}</p>
            <p> prev_performance  : {fundData.prev_performance}</p>
            <p> amount_in_router  : {fundData.amount_in_router.toString()}</p>
            <p> performance_fee  : {fundData.performance_fee}</p>

            <p> manager_account  : {fundData.manager_account.toBase58()}</p>
            <p> fund_pda  : {fundData.fund_pda.toBase58()}</p>
            <p> signer_nonce  : {fundData.signer_nonce}</p>

            <p> mango_positions  state== 0: inactive, 1: deposited, 2: position_open, </p>
            <p> 3: settled_open, 4: position_closed, 5: settled_close, 6: stale </p>

            {
                 fundData.mango_positions.length &&
                 <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}
                    >
                        <thead className="text-primary">
                                        <tr>
                                        <th style={{ width: "15%" }}>margin_account</th>
                                          <th style={{ width: "15%" }}>state</th>
                                          <th style={{ width: "15%" }}>margin_index</th>
                                          <th style={{ width: "15%" }}>position_side</th>
                                          <th style={{ width: "15%" }}>debtors</th>
                                          <th style={{ width: "15%" }}>padding</th>
                                          <th style={{ width: "15%" }}>position_id</th>
                                          <th style={{ width: "15%" }}>trade_amount</th>
                                          <th style={{ width: "15%" }}>fund_share</th>
                                          <th style={{ width: "15%" }}>share_ratio</th>

                                        </tr>
                        </thead>
                        <tbody>
                          {
                            fundData.mango_positions && 

                            fundData.mango_positions.map((i,x)=>{
                              return <tr key={x}>
                                <td >{i?.margin_account.toBase58()}</td>
                                <td >{i?.state}</td>
                                <td >{i?.margin_index}</td>
                                <td >{i?.position_side===0 ? 'LONG' : 'SHORT'}</td>
                                <td >{i?.debtors}</td>
                                <td >{'padding: u8 2'}</td>
                                <td >{i?.position_id}</td>
                                <td >{i?.trade_amount.toString()}</td>
                                <td >{i?.fund_share}</td>
                                <td >{i?.share_ratio}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>
            }

            <span> investors in transfer queue [10]  : </span>
            {
              fundData.investors.length &&
              <select id="cars">
                {
                  fundData.investors.map( (i,index) => 
                      <option key={index} value={i.toBase58()}> {index} - {i.toBase58()}</option>
                  )
                }
            </select>
            }
            <p> xpadding: [u8; 32]</p>

          </>
      }


     
     <p> FUND TOKENS </p>
    { true && 
    
        <Table className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }} >
                <thead className="text-primary">
                                <tr>
                                <th style={{ width: "15%" }}>index</th>
                                  <th style={{ width: "15%" }}>vault</th>
                                  <th style={{ width: "15%" }}>balance</th>
                                  <th style={{ width: "15%" }}>debt</th>
                                  <th style={{ width: "15%" }}>index</th>
                                  <th style={{ width: "15%" }}>mux</th>
                                </tr>
                </thead>
                <tbody>
                  {
                    fundTokens && 

                    fundTokens.map((i,x)=>{
                      return <tr key={x}>
                        <td >{x}</td>
                        <td >{i?.vault}</td>
                        <td >{i?.balance}</td>
                        <td >{i?.debt}</td>
                        <td >{i?.index}</td>
                        <td >{i?.mux}</td>
                      </tr>
                    })
                  }
                </tbody>
        </Table>
     }

  </div>
  )
}

