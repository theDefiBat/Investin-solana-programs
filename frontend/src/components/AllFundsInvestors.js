import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, INVESTOR_DATA, PLATFORM_DATA, SPL_TOKEN_MINT_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import BN from 'bn.js';
import { Card, Col, Row ,Table} from 'reactstrap';
import { Blob, seq, struct, u32, u8, u16, ns64 ,nu64} from 'buffer-layout';

export const AllFundsInvestors = () => {

  const [investments, setInvestments] = useState([])
  const [funds, setFunds] = useState([])

  const handleGetAllInvestments = async () => {

     const userkey = new PublicKey('zRzdC1b2zJte4rMjfaSFZwbnBfL1kNYaTAF4UC4bqpx');
    let investments = await connection.getProgramAccounts(programId, { filters: [
      { dataSize: INVESTOR_DATA.span },
      {
        memcmp: { offset: INVESTOR_DATA.offsetOf('owner'), bytes: userkey.toBase58() }
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

  const handleGetAllFunds = async () => {
    const managers = []
    const allFunds = await connection.getProgramAccounts(programId, { filters: [{ dataSize: FUND_DATA.span }] });

    // let fundsWithIVNAmt = [];

    for (const data of allFunds) {
        const decodedData = FUND_DATA.decode(data.account.data);
        
        //to get funds with non-zero IVN holdings
        // for (let j =0 ; j<decodedData?.tokens.length; j++){
        //   let i = decodedData?.tokens[j];
        //   if(i.vault.toBase58() === '11111111111111111111111111111111')
        //    continue;
        //   const vault_info = await connection.getAccountInfo(i.vault);
        //   if(!vault_info)
        //    continue;
        //   const token_data = SPL_TOKEN_MINT_DATA.decode(vault_info?.data);
         
        //   if(token_data?.mint_authority?.toBase58()==='iVNcrNE9BRZBC9Aqf753iZiZfbszeAVUoikgT9yvr2a'){
        //     const ivnBalance = await connection.getTokenAccountBalance(i.vault, "max");
        //     if(Number(ivnBalance.value.uiAmount) >0){
        //         console.log("balance::",(ivnBalance.value.uiAmount));
        //         fundsWithIVNAmt.push({  
        //           //  fundState : decodedData,
        //             fundPDA: decodedData.fund_pda.toBase58(),
        //             fundManager: decodedData.manager_account.toBase58(),
        //             fundStateAccount: data.pubkey.toBase58(),
        //             ivnBalance : (ivnBalance.value.uiAmount), 
        //             ivnVault : i.vault.toBase58()
        //         })
        //      }
        //   }
        // }
       

        // if (decodedData.is_initialized) {
            // const { updatedPerformance, currentAum } = await getPerformance(mapTokens(platformData.token_list, decodedData.tokens), prices, (decodedData.prev_performance), decodedData.total_amount, (await fundMarginData(decodedData))?.balance ?? 0)
            managers.push({
                fundState : decodedData,
                fundPDA: decodedData.fund_pda.toBase58(),
                fundManager: decodedData.manager_account.toBase58(),
                fundStateAccount: data.pubkey.toBase58(),
                // fundName: decodedData.fund_pda.toBase58(),
                // totalAmount: (new TokenAmount(decodedData.total_amount, TOKENS.USDC.decimals)).toEther().toNumber(),
                // currentPerformance: decodedData.number_of_active_investments == 0 ?
                //     (decodedData.prev_performance - 1) * 100
                //     : updatedPerformance,
                // currentAum,
                // minAmount: (new TokenAmount(decodedData.min_amount.toNumber(), TOKENS.USDC.decimals)).toEther().toNumber()
            });
        // }
    }
    console.log("managers:",managers);

    // console.error("fundsWithIVNAmt:",fundsWithIVNAmt);

    setFunds(managers);
  }

  return (
    <div className="form-div">
       <Card className="justify-content-center">
      <Row className="justify-content-between">
       <Col lg="12" xs="12">
      <h4>Investments</h4>
      <button onClick={handleGetAllInvestments}> get All Investments</button>

      <Table 
        className="tablesorter"
        responsive
        width="100%"
        style={{ overflow: 'hidden !important', textAlign: 'center' }}
        >
            <thead className="text-primary">
                            <tr>
                              <th style={{ width: "15%" }}>index</th>
                              <th style={{ width: "15%" }}>ivnStatePubKey</th>
                              <th style={{ width: "15%" }}>manager</th>
                              <th style={{ width: "15%" }}>owner</th>
                              <th style={{ width: "15%" }}>amount</th>
                              <th style={{ width: "15%" }}>amount_in_router</th>
                              <th style={{ width: "15%" }}>start_performance</th>
                              <th style={{ width: "15%" }}>is_initialized</th>
                              <th style={{ width: "15%" }}>has_withdrawn</th>
                              <th style={{ width: "15%" }}>withdrawn_from_margin</th>

                              <th style={{ width: "15%" }}>margin_debt</th>
                              <th style={{ width: "15%" }}>margin_position_id</th>
                              <th style={{ width: "15%" }}>8TokensIndexsAndDebts</th>

                            </tr>
                          </thead>


        <tbody>
          {
            investments && 

            investments.map((i,x)=>{
               return <tr key={i?.ivnStatePubKey?.toBase58()}>
                 <td >{x}</td>
                 <td >{i?.ivnStatePubKey?.toBase58()}</td>
                 <td >{i?.manager?.toBase58()}</td>
                 <td >{i?.owner?.toBase58()}</td>
                 <td>{i?.amount?.toString()/10**6}</td>
                 <td>{i?.amount_in_router?.toString()/10**6}</td>
                 <td>{i?.start_performance?.toString()}</td>

                 <td>{i?.is_initialized}</td>
                 <td>{i?.has_withdrawn}</td>
                 <td>{i?.withdrawn_from_margin}</td>

                 <td>{`${i?.margin_debt[0]} <==>  ${i?.margin_debt[1]}`}</td>
                 <td>{`${i?.margin_position_id[0]} <==>  ${i?.margin_position_id[1]}`}</td>

                 <td>8TokensIndexsAndDebts</td>
               </tr>
            })
          }
            </tbody>
          </Table>

     
      </Col>
      </Row>
      <Row className="justify-content-between">
      <Col lg="10" xs="10">
      <h4>Funds</h4>
      
      <button onClick={handleGetAllFunds}> get All Funds</button>

      <Table 
        className="tablesorter"
        responsive
        width="100%"
        style={{ overflow: 'hidden !important', textAlign: 'center' }}
        >
            <thead className="text-primary">
                            <tr>
                              <th style={{ width: "15%" }}>index</th>
                              <th style={{ width: "15%" }}>fundManager</th>
                              <th style={{ width: "15%" }}>fundPDA</th>
                              <th style={{ width: "15%" }}>fundStateAccount</th>
                              {/* <th style={{ width: "15%" }}>amount</th>
                              <th style={{ width: "15%" }}>amount_in_router</th> */}
                            </tr>
                          </thead>


        <tbody>
          {
            funds && 

            funds.map((i,x)=>{
               return <tr key={x}>
                 <td >{x}</td>
                 <td >{i?.fundManager}</td>
                 <td >{i?.fundPDA}</td>
                 <td >{i?.fundStateAccount}</td>
                 {/* <td>{i?.amount?.toString()/10**6}</td>
                 <td>{i?.amount_in_router?.toString()/10**6}</td> */}
               
               </tr>
            })
          }
            </tbody>
          </Table> 

      </Col>
            </Row>
      </Card>
    </div>
  )
}

