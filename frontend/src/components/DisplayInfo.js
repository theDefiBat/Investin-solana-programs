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

  const key = walletProvider?.publicKey;  
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
       

//   (async () => {


//     })()

// }, [walletProvider])

  

  return (
    <div className="form-div">
    <h4>Accounts</h4>
      <p> programID : {programIdX}</p>
      <p> adminAccount : {adminAccountX}</p>
      <p> platformStateAccount : {platformStateAccountX}</p>
      <p> priceStateAccount : {priceStateAccountX}</p>
      <button onClick={handleGetFundData}>GET FUND STATE</button>

    { true && <Table 
        className="tablesorter"
        responsive
        width="100%"
        style={{ overflow: 'hidden !important', textAlign: 'center' }}
        >
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
    </Table> }

  </div>
  )
}

