import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, idsIndex, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId, SYSTEM_PROGRAM_ID } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, FUND_PDA_DATA, INVESTOR_DATA, PLATFORM_DATA, SPL_TOKEN_MINT_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import BN from 'bn.js';
import { Card, Col, Row ,Table} from 'reactstrap';
import { Blob, seq, struct, u32, u8, u16, ns64 ,nu64} from 'buffer-layout';
import { IDS, sleep } from '@blockworks-foundation/mango-client';
import { TOKENS } from '../utils/tokens';
const ids= IDS['groups'][idsIndex];

export const AllFundsInvestors = () => {
  

  const [investments, setInvestments] = useState([])
  const [funds, setFunds] = useState([])
  const [oldFunds, setOldFunds] = useState([])

  const [tokenList, setTokenList] = useState([]) 

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const tokensStatic = Object.entries(TOKENS).map( i => i[1])

  useEffect(  ()=> {
    (async () => {
      const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
      if(!platformDataAcc) {
        alert('platform state not initilaized');
        return;
      }
        const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
        // console.log("platformData::",platformData);
        // setPlatformData(platformData)
        const platformTokens = platformData?.token_list;
        console.log("platformTokens::",platformTokens);
        console.log("ids.tokens::",ids.tokens);

        //  Object.keys(TOKENS).find(mt => TOKENS[mt]?.mintKey === t.mint.toBase58())

        let t = []; 
        if(platformTokens?.length) {
          t = platformTokens.map( (i) => {
            return {
               symbol: ((tokensStatic).find( k => k.mintAddress ===i.mint.toBase58()))?.symbol ?? 'NONE',
                mintAddress: i.mint.toBase58(),
                decimals: i.decimals?.toString(),
               pool_coin_account: i.pool_coin_account.toBase58(),
                pool_pc_account: i.pool_pc_account.toBase58(),
                pool_price : i.pool_price?.toString(),
            }
          })
        } 

        setTokenList(t)
    })()
    
  },[walletProvider])

  

  const handleGetAllMigratedFunds = async () => {
    const managers = []
    const allFunds = await connection.getProgramAccounts(programId, { filters: [
      { dataSize: FUND_PDA_DATA.span },
      //  {
      //   memcmp: { offset: FUND_PDA_DATA.offsetOf('number_of_active_investments'), bytes: '3' }
      // }
    ] });
    console.log("-------1)-AllMigratedFunds nondecoded::",allFunds)
    for (const data of allFunds) {
        const decodedData = FUND_PDA_DATA.decode(data.account.data);
        // const PDA_balance  = await connection.getBalance(decodedData.fund_pda, "max");
        // console.log("PDA_balance:",PDA_balance)

       
            managers.push({
                fund_v3_index : decodedData.fund_v3_index,
                fundState : decodedData,
                fundPDA: decodedData.fund_pda.toBase58(),
                fundManager: decodedData.manager_account.toBase58(),
                fundStateAccount: data.pubkey.toBase58(),
                // PDA_balance : PDA_balance,
                // fundName: decodedData.fund_pda.toBase58(),
                // totalAmount: (new TokenAmount(decodedData.total_amount, ids.tokens[0].decimals)).toEther().toNumber(),
            });
    }
    console.log("-----2) AllMigratedFunds Decoded PDA funds:",managers);

    setFunds(managers);
  }

  const handleGetAllNonMigratedFunds = async () => {
    const managers = []
    const allFunds = await connection.getProgramAccounts(programId, { filters: [
      { dataSize: FUND_DATA.span },
      //  {
      //   memcmp: { offset: FUND_PDA_DATA.offsetOf('number_of_active_investments'), bytes: '3' }
      // }
    ] });
    console.log("-----1) All OLD FUND_STATE Funds nodecoded::",allFunds)
  
    for (const data of allFunds) {
         const decodedData = FUND_DATA.decode(data.account.data);

        //  const PDA_balance  = await connection.getBalance(decodedData.fund_pda, "max");
        //  console.log("PDA_balance:",PDA_balance)

        if (decodedData.is_initialized && decodedData.version!==3) {
            managers.push({
                fund_v3_index : decodedData.fund_v3_index,
                fundState : decodedData,
                fundPDA: decodedData.fund_pda.toBase58(),
                fundManager: decodedData.manager_account.toBase58(),
                fundStateAccount: data.pubkey.toBase58(),
                // PDA_balance : PDA_balance,
                // fundName: decodedData.fund_pda.toBase58(),
                // totalAmount: (new TokenAmount(decodedData.total_amount, ids.tokens[0].decimals)).toEther().toNumber(),
            });
        } else {
          // console.log("fund is_initialized false",decodedData?.fundPDA?.toBase58(), decodedData)
        }
    }
    console.log("------2) OLD funds decoded:",managers);  
    setOldFunds(managers);
  }

  const handleMigrate = async () => {

  

   console.log("---calling migrate")

    if( oldFunds.length==0){
      alert("first get funds")
      return;
    }
  
      const key = walletProvider?.publicKey;
      if (!key) {
        alert("connect wallet")
        return;
      };

      const fundStateAndPDAS = []
      for(let i=0;i<2;i++){
        fundStateAndPDAS.push({pubkey: new PublicKey(oldFunds[i].fundPDA), isSigner: false, isWritable: true })
        fundStateAndPDAS.push({pubkey: new PublicKey(oldFunds[i].fundStateAccount), isSigner: false, isWritable: true })
      }
    
      

      const transaction = new Transaction()
      const dataLayout = struct([u8('instruction'),u8('count')])
      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode({
        instruction: 23,
        count : 2
      },data)

      
      
      const keys = [
      {pubkey: platformStateAccount, isSigner: false, isWritable: true },
      {pubkey: key, isSigner: true, isWritable: true },
      {pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: true},

      // {pubkey: fundPDA[0], isSigner: false, isWritable:true},
      // {pubkey: fundStateAccount, isSigner: false, isWritable: true},
      ...fundStateAndPDAS
    ]

    for(let i=0; i<keys.length;i++) {
      console.log("key:",i, keys[i].pubkey.toBase58())
    }
      
      const migrate_instruction = new TransactionInstruction({
        keys,
        programId,
        data
      });
  
      transaction.add(migrate_instruction);
      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("tx perf: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

      await sleep(20000)
      await handleGetAllNonMigratedFunds()
}

  return (
    <div className="form-div">
       <Card className="justify-content-center" >
      
       <h4>Migrate State </h4>          <button onClick={handleMigrate}>Migrate</button>
          <br />
    
      <h4>Funds</h4>
      
      <button onClick={handleGetAllMigratedFunds}> get All Migrated Funds</button>

         <Table 
        className="tablesorter"
        responsive
        // width="100%"
        style={{ overflow: 'hidden !important', textAlign: 'center' }}
        >
            <thead className="text-primary">
                            <tr>
                              <th style={{ width: "15%" }}>index</th>
                              <th style={{ width: "15%" }}>fund_v3_index</th>
                              <th style={{ width: "15%" }}>fundManager</th>
                              <th style={{ width: "15%" }}>fundPDA</th>
                              <th style={{ width: "15%" }}>fundStateAccount</th>
                              {/* <th style={{ width: "15%" }}>PDA_balance</th> */}
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
                 <td >{i?.fund_v3_index}</td>
                 <td >{i?.fundManager}</td>
                 <td >{i?.fundPDA}</td>
                 <td >{i?.fundStateAccount}</td>
                 {/* <td>{i?.PDA_balance}</td> */}
                 {/* <td>{i?.amount?.toString()/10**6}</td>
                 <td>{i?.amount_in_router?.toString()/10**6}</td> */}
               
               </tr>
            })
          }
            </tbody>
          </Table>

          <button onClick={handleGetAllNonMigratedFunds}> get All OLD Funds</button>

        <Table 
        className="tablesorter"
        responsive
        // width="100%"
        style={{ overflow: 'hidden !important', textAlign: 'center' }}
        >
            <thead className="text-primary">
                            <tr>
                              <th style={{ width: "15%" }}>index</th>
                              <th style={{ width: "15%" }}>fund_v3_index</th>
                              <th style={{ width: "15%" }}>fundManager</th>
                              <th style={{ width: "15%" }}>fundPDA</th>
                              <th style={{ width: "15%" }}>fundStateAccount</th>
                              {/* <th style={{ width: "15%" }}>PDA_balance</th> */}
                              {/* <th style={{ width: "15%" }}>amount</th>
                              <th style={{ width: "15%" }}>amount_in_router</th> */}
                            </tr>
            </thead>
        <tbody>
          {
            oldFunds && 

            oldFunds.map((i,x)=>{
              return <tr key={x}>
                <td >{x}</td>
                <td >{i?.fund_v3_index}</td>
                <td >{i?.fundManager}</td>
                <td >{i?.fundPDA}</td>
                <td >{i?.fundStateAccount}</td>
                {/* <td>{i?.PDA_balance}</td> */}
                {/* <td>{i?.amount?.toString()/10**6}</td>
                <td>{i?.amount_in_router?.toString()/10**6}</td> */}
              
              </tr>
            })
          }
            </tbody>
          </Table> 

          <hr/>
         
      </Card>
    </div>
  )
}

