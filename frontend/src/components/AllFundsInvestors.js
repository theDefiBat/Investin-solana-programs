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

    //  const userkey = new PublicKey('zRzdC1b2zJte4rMjfaSFZwbnBfL1kNYaTAF4UC4bqpx');
    //  const key = walletProvider?.publicKey;
    let investments = await connection.getProgramAccounts(programId, { filters: [
      { dataSize: INVESTOR_DATA.span },
      // {
      //   memcmp: { offset: INVESTOR_DATA.offsetOf('owner'), bytes: userkey.toBase58() }
      // }
    ] });
    // console.log("investments::",investments)
    const newInvestors = []
    for (const investment of investments) {
      const invStateData = INVESTOR_DATA.decode(investment.account.data)
      invStateData['ivnStatePubKey'] = investment.pubkey;
      if (invStateData.is_initialized) {
        newInvestors.push(invStateData)
      }
    }
    console.log("newInvestors::",newInvestors)
    setInvestments(newInvestors);
  }

  const handleGetAllFunds = async () => {
    const managers = []
    const allFunds = await connection.getProgramAccounts(programId, { filters: [{ dataSize: FUND_DATA.span }] });

    for (const data of allFunds) {
        const decodedData = FUND_DATA.decode(data.account.data);
        
        if (decodedData.is_initialized) {
            managers.push({
                fundState : decodedData,
                fundPDA: decodedData.fund_pda.toBase58(),
                fundManager: decodedData.manager_account.toBase58(),
                fundStateAccount: data.pubkey.toBase58(),
                vault_key: decodedData.vault_key.toBase58(),
                mngo_vault_key: decodedData.mngo_vault_key.toBase58(),
                mango_account: decodedData.mango_account.toBase58(),
               
                mngo_per_share: decodedData.mngo_per_share.toString(),
                mngo_manager: decodedData.mngo_manager.toString(),
                mngo_accrued: decodedData.mngo_accrued.toString(),
                total_mngo_accrued: decodedData.total_mngo_accrued.toString(),

                // minAmount: (new TokenAmount(decodedData.min_amount.toNumber(), TOKENS.USDC.decimals)).toEther().toNumber()
            });
        }
    }
    console.log("managers:",managers);
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
                              <th style={{ width: "15%" }}>mngo_debt</th>
                              <th style={{ width: "15%" }}>start_index</th>
                              <th style={{ width: "15%" }}>has_withdrawn</th>
                              <th style={{ width: "15%" }}>withdrawn_from_margin</th>
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
                 <td>{i?.mngo_debt?.toString()}</td>
                 <td>{i?.start_index?.toString()}</td>

                 <td>{i?.has_withdrawn}</td>
                 <td>{i?.withdrawn_from_margin}</td>
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

                              <th style={{ width: "15%" }}>vault_key</th>
                              <th style={{ width: "15%" }}>mngo_vault_key</th>
                              <th style={{ width: "15%" }}>mango_account</th>

                              <th style={{ width: "15%" }}>mngo_per_share</th>
                              <th style={{ width: "15%" }}>mngo_manager</th>
                              <th style={{ width: "15%" }}>mngo_accrued</th>
                              <th style={{ width: "15%" }}>total_mngo_accrued</th>

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
                
                 <td >{i?.vault_key}</td>
                 <td >{i?.mngo_vault_key}</td>
                 <td >{i?.mango_account}</td>
                 <td >{i?.mngo_per_share}</td>
                 <td >{i?.mngo_manager}</td>
                 <td >{i?.mngo_accrued}</td>
                 <td >{i?.total_mngo_accrued}</td>
               
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

