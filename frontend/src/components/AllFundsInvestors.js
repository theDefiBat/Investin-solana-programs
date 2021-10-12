import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, INVESTOR_DATA, PLATFORM_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import { MANGO_TOKENS } from "../utils/tokens";
import BN from 'bn.js';
import { Card, Col, Row ,Table} from 'reactstrap';
import { Blob, seq, struct, u32, u8, u16, ns64 ,nu64} from 'buffer-layout';

export const AllFundsInvestors = () => {

  const [investments, setInvestments] = useState([])
  const [funds, setFunds] = useState([])

  const handleGetAllInvestments = async () => {

   

    let investments = await connection.getProgramAccounts(programId, { filters: [{ dataSize: INVESTOR_DATA.span }] });
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
                 {/* <td>{i?.amount?.toString()}</td>
                 <td>{i?.amount?.toString()}</td> */}

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
      
        {/* <table>
          
           {
             platformData?.token_list && 

             platformData?.token_list.map((i)=>{
                return <tr key={i?.mint?.toBase58()}>
                  <td >{i?.mint?.toBase58()}</td>
                  <td>{i?.pool_price?.toString()}</td>
                </tr>
             })
           }
           </table> */}
      </Col>
            </Row>
      </Card>
    </div>
  )
}

