import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, idsIndex, MANGO_GROUP_ACCOUNT, MANGO_PROGRAM_ID, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId, SERUM_PROGRAM_ID_V3, SYSTEM_PROGRAM_ID } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, FUND_PDA_DATA, INVESTOR_DATA, PLATFORM_DATA, SPL_TOKEN_MINT_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import BN from 'bn.js';
import { Card, Col, Row ,Table} from 'reactstrap';
import { Blob, seq, struct, u32, u8, u16, ns64 ,nu64} from 'buffer-layout';
import { IDS, MangoClient, sleep } from '@blockworks-foundation/mango-client';
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
    const client = new MangoClient(connection, MANGO_PROGRAM_ID)
    let mangoGroup = await client.getMangoGroup(MANGO_GROUP_ACCOUNT)
    const mangoCache = await mangoGroup.loadCache(connection);
    const rootBanks = await mangoGroup.loadRootBanks(connection) 

    for (const data of allFunds) {
        const decodedData = FUND_PDA_DATA.decode(data.account.data);
        // const PDA_balance  = await connection.getBalance(decodedData.fund_pda, "max");
        // console.log("PDA_balance:",PDA_balance)

        console.log("mangoAccount:",decodedData.mango_positions.mango_account.toBase58())
        if(decodedData.mango_positions.mango_account.toBase58() !=='11111111111111111111111111111111') {
          let mangoAccount = await client.getMangoAccount(decodedData.mango_positions.mango_account, SERUM_PROGRAM_ID_V3)
          console.log("mm mangoAccount:",mangoAccount)
               managers.push({
                   fund_v3_index : decodedData.fund_v3_index,
                   fundState : decodedData,
                   fundPDA: decodedData.fund_pda.toBase58(),
                   fundManager: decodedData.manager_account.toBase58(),
                   fundStateAccount: data.pubkey.toBase58(),
                   // PDA_balance : PDA_balance,   
                   // fundName: decodedData.fund_pda.toBase58(),
                   // totalAmount: (new TokenAmount(decodedData.total_amount, ids.tokens[0].decimals)).toEther().toNumber(),
                   mango_account: decodedData.mango_positions.mango_account.toBase58(),
         
                   index0: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[0], mangoGroup, 0)).toFixed(6),
                   index1: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[1], mangoGroup, 1)).toFixed(6),
                   index2: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[2], mangoGroup, 2)).toFixed(6),
                   index3: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[3], mangoGroup, 3)).toFixed(6),
                   index4: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[4], mangoGroup, 4)).toFixed(6),
                   index5: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[5], mangoGroup, 5)).toFixed(6),
                   index6: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[6], mangoGroup, 6)).toFixed(6),
                   // index7: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[7], mangoGroup, 7)).toFixed(6),
                   index8: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[8], mangoGroup, 8)).toFixed(6),
                   // index9: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[9], mangoGroup, 9)).toFixed(6),
                   index10: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[10], mangoGroup, 10)).toFixed(6),
                   index11: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[11], mangoGroup, 11)).toFixed(6),
                   index12: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[12], mangoGroup, 12)).toFixed(6),
                   // index13: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[13], mangoGroup, 13)).toFixed(6),
                   index14: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[14], mangoGroup, 14)).toFixed(6),
                   index15: (mangoAccount.getUiDeposit(mangoCache.rootBankCache[15], mangoGroup, 15)).toFixed(6),
               });
        } else {
         console.log("NO mangoAccount:",decodedData.mango_positions.mango_account.toBase58(),decodedData.fund_pda.toBase58())
        }
     
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
                               <th style={{ width: "15%" }}>mango_account</th>

                                <th style={{ width: "15%" }}>0</th>
                                <th style={{ width: "15%" }}>1</th>
                                <th style={{ width: "15%" }}>2</th>
                                <th style={{ width: "15%" }}>3</th>
                                <th style={{ width: "15%" }}>4</th>
                                <th style={{ width: "15%" }}>5</th>
                                <th style={{ width: "15%" }}>6</th>
                                {/* <th style={{ width: "15%" }}>7</th> */}
                                <th style={{ width: "15%" }}>8</th>
                                {/* <th style={{ width: "15%" }}>9</th> */}
                                <th style={{ width: "15%" }}>10</th>
                                <th style={{ width: "15%" }}>11</th>
                                <th style={{ width: "15%" }}>12</th>
                                {/* <th style={{ width: "15%" }}>13</th> */}
                                <th style={{ width: "15%" }}>14</th>
                                <th style={{ width: "15%" }}>15</th>

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

                  <td >{i?.mango_account}</td>

                  <td >{i?.index0}</td>
                  <td >{i?.index1}</td>
                  <td >{i?.index2}</td>
                  <td >{i?.index3}</td>
                  <td >{i?.index4}</td>
                  <td >{i?.index5}</td>
                  <td >{i?.index6}</td>

                  <td >{i?.index8}</td>

                  <td >{i?.index10}</td>
                  <td >{i?.index11}</td>
                  <td >{i?.index12}</td>

                  <td >{i?.index14}</td>
                  <td >{i?.index15}</td>

               
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

