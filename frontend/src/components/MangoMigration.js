import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8 } from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, NEW_FUND_DATA, PLATFORM_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import {  TOKENS } from "../utils/tokens";
import BN from 'bn.js';
import { Card, Col, Row } from 'reactstrap';

export const MangoMigration = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

   const [fundStateAcc, setFundStateAcc] = useState('')
   const [managerAddress, setManagerAddress] = useState('')
   const [fundData, setFundData] = useState({})
   const [newFundData, setNewFundData] = useState({})

 
//   useEffect(  ()=> {
//     (async () => {
//     })()
//   },[walletProvider])

  const handleGetFundData = async () => {

   console.log("managerAddress::",managerAddress)
    if(!walletProvider) {
      alert("connect wallet ")
      return;
    }
    // let key;
    // if(managerAddress){
    //    key = new PublicKey(managerAddress);
    // } else {
    //    alert('enter adde');
    //    return; 
    // } 
    // if (!key ) {
    //   alert("connect wallet ")
    //   return;
    // }
  
    //   const fundStateAcc = await PublicKey.createWithSeed(
    //     key,
    //     FUND_ACCOUNT_KEY,
    //     programId,
    //   );
    //   console.log("FUND fundStateAcc:: ", fundStateAcc.toBase58())  
      const fundDataAcc = await connection.getAccountInfo(new PublicKey(fundStateAcc));
      console.log("fundDataAcc::",fundDataAcc);
      if (fundDataAcc == null)
      {
         alert("fundDataAcc info not found")
        return;
      }
      const fundData = FUND_DATA.decode(fundDataAcc.data)
      console.error("fundData::",fundData);
      setFundData(fundData);
    //   ==============

     const newfundData = NEW_FUND_DATA.decode(fundDataAcc.data)
      console.error("newfundData::",newfundData);
      setNewFundData(newfundData);
  
  }

  const handleMigrate = async () => {
    console.log("handleMigrate")
    if(walletProvider?.PublicKey.toBase58()!== 'owZmWQkqtY3Kqnxfua1KTHtR2S6DgBTP75JKbh15VWG') {
        alert('admin only');
        return;
    }
    const dataLayout = struct([
        u8('instruction'),
      ])
      const transaction = new Transaction()
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 11,
        },
        data
      )

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: new PublicKey('owZmWQkqtY3Kqnxfua1KTHtR2S6DgBTP75JKbh15VWG'), isSigner: true, isWritable: false },
          { pubkey: new PublicKey(fundStateAcc), isSigner: false, isWritable: true },
        ],
        programId,
        data
      });
      transaction.add(instruction)
      transaction.feePayer = walletProvider?.publicKey;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 
  }

  return (
    <div className="form-div">
       <Card className="justify-content-center">

       <h4>Migrate</h4>
            <p>fundStateAcc : {fundStateAcc}</p> 
            {/* managerAddress :: <input type="text" style={{width :"500px"}} value={managerAddress} onChange={(event) => setManagerAddress(event.target.value)} /> */}

            fundSateAcc :: <input type="text" style={{width :"500px"}} value={fundStateAcc} onChange={(event) => setFundStateAcc(event.target.value)} />
            <button style={{width :"300px"}} onClick={handleMigrate}>Migrate</button>
            <button style={{width :"300px"}} onClick={handleGetFundData}>get fund state</button>


    <Row className="justify-content-between">
      <Col lg="6" xs="6">
      <h4>OLD FUND STATE</h4>
           { fundData && fundData?.min_amount &&
           <>
            <p> number_of_active_investments : {fundData?.number_of_active_investments}</p>
            <p> no_of_investments : {fundData?.no_of_investments}</p>

            <p> no_of_margin_positions : {fundData?.no_of_margin_positions}</p>
            <p> no_of_assets (active tokens) : {fundData?.no_of_assets}</p>
            <p> position_count  : {fundData?.position_count}</p>
            <p> version  : {fundData?.version}</p>
            <p> padding   : u8,7</p>

            <p> min_amount  : {fundData?.min_amount?.toString()}</p>
            <p> min_return  : {fundData?.min_return?.toString()}</p>
            <p> performance_fee_percentage  : {fundData?.performance_fee_percentage}</p>
            <p> total_amount in fund USDC  : {fundData?.total_amount?.toString()}</p>
            <p> prev_performance  : {fundData?.prev_performance}</p>
            <p> amount_in_router  : {fundData?.amount_in_router?.toString()}</p>
            <p> performance_fee  : {fundData?.performance_fee}</p>

            <p> manager_account  : {fundData?.manager_account?.toBase58()}</p>
            <p> fund_pda  : {fundData?.fund_pda?.toBase58()}</p>
            <p> signer_nonce  : {fundData?.signer_nonce}</p>

            {
              fundData?.mango_positions && 
              <>
               <p>changes----</p>
                <p> **version  : {fundData?.version?.toString()}</p>

                <p> margin_account  : {fundData?.mango_positions?.margin_account?.toBase58()}</p>
                <p> perp_markets  : {fundData?.mango_positions?.perp_markets?.toString()}</p>
                <p> deposit_index  : {fundData?.mango_positions?.deposit_index?.toString()}</p>
                <p> markets_active  : {fundData?.mango_positions?.markets_active?.toString()}</p>
                <p> deposits_active  : {fundData?.mango_positions?.deposits_active?.toString()}</p>
                {/* <p> investor_debts[0]  : {fundData?.mango_positions?.investor_debts[0]?.toString()}</p>
                <p> investor_debts[1]  : {fundData?.mango_positions?.investor_debts[1]?.toString()}</p> */}
              </>
            }
           
            </>
}
      </Col>

      <Col lg="6" xs="6">
      <h4>NEW FUND STATE</h4>
      { newFundData && newFundData?.min_amount && 
      <>
            <p> number_of_active_investments : {newFundData?.number_of_active_investments}</p>
            <p> no_of_investments : {newFundData?.no_of_investments}</p>

            <p> no_of_margin_positions : {newFundData?.no_of_margin_positions}</p>
            <p> no_of_assets (active tokens) : {newFundData?.no_of_assets}</p>
            <p> position_count  : {newFundData?.position_count}</p>
            <p> version  : {newFundData?.version}</p>
            <p> padding   : u8,7</p>

            <p> min_amount  : {newFundData?.min_amount.toString()}</p>
            <p> min_return  : {newFundData?.min_return.toString()}</p>
            <p> performance_fee_percentage  : {newFundData?.performance_fee_percentage}</p>
            <p> total_amount in fund USDC  : {newFundData?.total_amount?.toString()}</p>
            <p> prev_performance  : {newFundData?.prev_performance}</p>
            <p> amount_in_router  : {newFundData?.amount_in_router?.toString()}</p>
            <p> performance_fee  : {newFundData?.performance_fee}</p>

            <p> manager_account  : {newFundData?.manager_account?.toBase58()}</p>
            <p> fund_pda  : {newFundData?.fund_pda?.toBase58()}</p>
            <p> signer_nonce  : {newFundData?.signer_nonce}</p>
           

            {
              newFundData?.mango_positions.mango_account && 
              <>
                <p>changes----</p>
                <p> **version  : {newFundData?.version.toString()}</p>

                <p> mango_account  : {newFundData?.mango_positions.mango_account.toBase58()}</p>
                <p> perp_markets  : {newFundData?.mango_positions.perp_markets.toString()}</p>
                <p> deposit_index  : {newFundData?.mango_positions.deposit_index.toString()}</p>
                <p> markets_active  : {newFundData?.mango_positions.markets_active.toString()}</p>
                <p> deposits_active  : {newFundData?.mango_positions.deposits_active.toString()}</p>
                <p> investor_debts[0]  : {newFundData?.mango_positions.investor_debts[0].toString()}</p>
                <p> investor_debts[1]  : {newFundData?.mango_positions.investor_debts[1].toString()}</p>
              </>
            }
            </>
}
      </Col>
            </Row>
      </Card>
    </div>
  )
}

