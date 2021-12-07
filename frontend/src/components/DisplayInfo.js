import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import React, { useEffect, useState } from 'react'
import { GlobalState } from '../store/globalState';

import { adminAccount, connection, FUND_ACCOUNT_KEY, idsIndex, platformStateAccount, priceStateAccount, programId } from '../utils/constants';
import { blob, nu64, struct, u32, u8 } from 'buffer-layout';
import { AMM_INFO_LAYOUT_V4, FUND_DATA, SPL_TOKEN_MINT_DATA } from '../utils/programLayouts';
import { IDS, MangoClient, I80F48, NodeBankLayout, PerpAccountLayout, PerpMarketLayout ,RootBankCacheLayout, RootBankLayout} from '@blockworks-foundation/mango-client';
import { Card, Col, Row ,Table} from 'reactstrap';
import { DEV_TOKENS } from '../utils/pools';


export const DisplayInfo = (props) => {
  const ids= IDS['groups'][idsIndex];

  const [fundData, setFundData] = useState("");
  const [fundTokens, setFundTokens] = useState([]);
  const [mangoGroup, setMangoGroup] = useState({})
  const [mangoAccount, setMangoAccount] = useState('7BLzTNvjNjaCnZ2Nnpu1aFYqTBsL8Lz2FUxknSAZ8tDX')
  const [mangoAccountData, setMangoAccountData] = useState({})
  const [nodeBank, setNodeBank] = useState({})
  const [rootBank, setRootBank] = useState({})
  const walletProvider = GlobalState.useState(s => s.walletProvider);
  
const programIdX = programId.toBase58();
const adminAccountX = adminAccount.toBase58();
const platformStateAccountX = platformStateAccount.toBase58();
const priceStateAccountX = priceStateAccount.toBase58();

const handleGetFundData = async () => {

  
  // let ammInfo = await connection.getAccountInfo(new PublicKey('384zMi9MbUKVUfkUdrnuMfWBwJR9gadSxYimuXeJ9DaJ'))
  // let amm = AMM_INFO_LAYOUT_V4.decode(ammInfo.data)
  // console.log("amm :: ", amm)

  // console.log("amm.poolCoinTokenAccount :: ", amm.poolCoinTokenAccount.toBase58())
  // console.log("amm.poolPcTokenAccount :: ", amm.poolPcTokenAccount.toBase58())

  // console.log("amm.ammId :: ", amm.ammId)
  // console.log("amm.ammAuthority :: ", amm.ammAuthority.toBase58())
  // console.log("amm.ammOpenOrders :: ", amm.ammOpenOrders.toBase58())
  // console.log("amm.ammTargetOrders :: ", amm.ammTargetOrders.toBase58())

  // console.log("amm.serumProgramId :: ", amm.serumProgramId.toBase58())
  // console.log("amm.serumMarket :: ", amm.serumMarket.toBase58())
  // console.log("amm.serumBids :: ", amm.serumBids.toBase58())
  // console.log("amm.serumAsks :: ", amm.serumAsks.toBase58())
  // console.log("amm.serumEventQueue :: ", amm.serumEventQueue.toBase58())
  // console.log("amm.serumCoinVaultAccount :: ", amm.serumCoinVaultAccount.toBase58())
  // console.log("amm.serumPcVaultAccount :: ", amm.serumPcVaultAccount.toBase58())
  // console.log("amm.serumVaultSigner :: ", amm.serumVaultSigner.toBase58())

  // const market = marketToLayout[ammInfo.serumMarket]

  if(!walletProvider) {
    alert("connect wallet ")
    return;
  }
  // chiro addr : zRzdC1b2zJte4rMjfaSFZwbnBfL1kNYaTAF4UC4bqpx
  // darkness :  ysh2B9XKTbX8DdsWM3HrzjuZ5otrFc24Y6H8naAmoYi
  // double mango : CS22VrmEuNH7Jb1sJVLMubr4aBFiXTkxB1iv5SiZGiA4
  // baigan : B9YVBghroTdohKoTQb7SofHh2U6FxAybuF6UwZEw7c1x
  // aak : 5Arakn7JSt3sPkXdWvy1887Bjd2d755b57BTEwBR7cW3
  // the Moon (lucio) :FRaWwEyKTwFgcU7tZa3xbSCxkEH61rdpCWqVV7z1Zj7S
  
  // const key = new PublicKey('zRzdC1b2zJte4rMjfaSFZwbnBfL1kNYaTAF4UC4bqpx');
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
    
    let fundStateTokens = [];
    for (let j =0; j<fundData?.tokens.length; j++) {
       const i = fundData?.tokens[j];
       console.log("vault vault_info token::",i);
       if(!i.is_active)
        continue;

       const vault_info = await connection.getAccountInfo(i.vault);
       console.log("vault vault_info ::",vault_info);
       if(!vault_info)
         {
           console.log("vault error ::");
           continue;
         }
       const data = SPL_TOKEN_MINT_DATA.decode(vault_info.data)
      //  console.log("tokenData ::",data);
      //  if(data?.mint_authority?.toBase58()==='iVNcrNE9BRZBC9Aqf753iZiZfbszeAVUoikgT9yvr2a'){
      //     const ivnBalance = await connection.getTokenAccountBalance(i.vault, "max");
      //     console.log("balance::",(ivnBalance.value.uiAmount))
      //   }
      
      const obj =  {
        balance : i.balance.toString(),
        debt : i.debt.toString(),
        vault : i.vault.toBase58(),
        mint_authority: data?.mint_authority?.toBase58(),
        index : i.index,
        is_on_mango : i.is_on_mango,
        is_initialized : i.is_initialized,
        mux : i.mux,
        padding : i.padding
      }
      fundStateTokens.push(obj);
    }
   
    setFundTokens(fundStateTokens);
    console.error("parsed fundState tokens: ",fundStateTokens);
}

const getAllDecodeMangoData = async () => {

  let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
  let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
  console.log("mango group:: ", mangoGroup)
  let mangoGroupDecoded = {};
  mangoGroupDecoded.admin = mangoGroup.admin.toBase58();
  mangoGroupDecoded.dexProgramId = mangoGroup.dexProgramId.toBase58();
  mangoGroupDecoded.insuranceVault = mangoGroup.insuranceVault.toBase58();
  mangoGroupDecoded.mangoCache = mangoGroup.mangoCache.toBase58();

  mangoGroupDecoded.msrmVault = mangoGroup.msrmVault.toBase58();
  mangoGroupDecoded.numOracles = mangoGroup.numOracles.toString();

  mangoGroupDecoded.oracles =  mangoGroup.oracles.map( i => i.toBase58());

  mangoGroupDecoded.perpMarkets =  mangoGroup.perpMarkets.map( i => {
    return {
      baseLotSize: i.baseLotSize.toString(),
      initAssetWeight: i.initAssetWeight.toString(),
      initLiabWeight: i.initLiabWeight.toString(),
      liquidationFee: i.liquidationFee.toString(),
      maintAssetWeight: i.maintAssetWeight.toString(),
      maintLiabWeight: i.maintLiabWeight.toString(),
      makerFee: i.makerFee.toString(),
      perpMarket:  i.perpMarket.toBase58(),
      quoteLotSize: i.quoteLotSize.toString(),
      takerFee:  i.takerFee.toString(),
    }
  });

  mangoGroupDecoded.spotMarkets =  mangoGroup.spotMarkets.map( i => {
    return {
      initAssetWeight: i.initAssetWeight.toString(),
      initLiabWeight: i.initLiabWeight.toString(),
      liquidationFee: i.liquidationFee.toString(),
      maintAssetWeight: i.maintAssetWeight.toString(),
      maintLiabWeight: i.maintLiabWeight.toString(),
      spotMarket:  i.spotMarket.toBase58(),
    }
  });

  mangoGroupDecoded.tokens =  mangoGroup.tokens.map( i => {
    return {
      decimals: i.decimals,
      mint: i.mint.toBase58(),
      rootBank: i.rootBank.toBase58(),
    }
  });
  setMangoGroup(mangoGroupDecoded)
  console.error("mango group DECODED**:: ", mangoGroupDecoded)


  let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].nodeKeys[0]))
  let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
  console.log("nodebank:: ", nodeBank)
  let nodeBankDecode = {
    borrows:  nodeBank.borrows.toString(),
    deposits: nodeBank.deposits.toString(),
    vault: nodeBank.vault.toBase58()
  }
  setNodeBank(nodeBankDecode)
  console.error("nodeBankDecode:: ", nodeBankDecode)

  let rootBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].rootKey))
  let rootBank = RootBankLayout.decode(rootBankInfo.data)
  console.log("rootBank:: ", rootBank)
  let rootBankDecode = {
    optimalUtil:  rootBank.optimalUtil.toString(),
    optimalRate: rootBank.optimalRate.toString(),
    maxRate: rootBank.maxRate.toString(),
    depositIndex:  rootBank.depositIndex.toString(),
    borrowIndex: rootBank.borrowIndex.toString(),
    lastUpdated: rootBank.lastUpdated.toString(),
  }
  setRootBank(rootBankDecode)
  console.error("rootBankDecode:: ", rootBankDecode)

  return;
}

const getMangoAccountData = async () => {
    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))

    let mangoAcc = await client.getMangoAccount(new PublicKey(mangoAccount), ids.serumProgramId)
    console.log("mangoAccount:: ", mangoAccount)

    let mangoAccountDecoded = {};
    mangoAccountDecoded.mangoGroup = mangoAcc.mangoGroup.toBase58();
    mangoAccountDecoded.borrows = mangoAcc.borrows.map( i => i.toString());
    mangoAccountDecoded.clientOrderIds = mangoAcc.clientOrderIds.map( i => i.toString());
    mangoAccountDecoded.deposits = mangoAcc.deposits.map( i => i.toString());
    mangoAccountDecoded.orders = mangoAcc.orders.map( i => i.toString());

    mangoAccountDecoded.perpAccounts =  mangoAcc.perpAccounts.map( i => {
      return {
        asksQuantity: i.asksQuantity.toString(),
        basePosition: i.basePosition.toString(),
        bidsQuantity: i.bidsQuantity.toString(),
        longSettledFunding: i.longSettledFunding.toString(),
        mngoAccrued: i.mngoAccrued.toString(),
        quotePosition: i.quotePosition.toString(),
        shortSettledFunding: i.shortSettledFunding.toString(),
        takerBase: i.takerBase.toString(),
        takerQuote: i.takerQuote.toString(),
      }
    });
    mangoAccountDecoded.spotOpenOrders = mangoAcc.spotOpenOrders.map( i => i.toBase58());
    setMangoAccountData(mangoAccountDecoded)
    console.error("mangoAccountDecoded DECODED**:: ", mangoAccountDecoded)

}
  

  return (
    <div className="form-div">
      <h4>{ process.env.REACT_APP_NETWORK }</h4>
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
            <hr/>

            <h5>----Mango-positions</h5>
            <p> mango_account  : {fundData.mango_positions.mango_account.toBase58()}</p>
            <p> deposit_index  : {fundData.mango_positions.deposit_index}</p>
            <p> markets_active  : {fundData.mango_positions.markets_active}</p>
            <p> deposits_active  : {fundData.mango_positions.deposits_active}</p>
            <br/>
            <p> perp_markets[]  : {fundData.mango_positions.perp_markets[0]} {' || '}
            {fundData.mango_positions.perp_markets[1]} {' || '}
            {fundData.mango_positions.perp_markets[2]}{' || '}
            {fundData.mango_positions.perp_markets[3]}
            </p>
            

            <p> investor_debts[]  : {fundData.mango_positions.investor_debts[0].toString()} {' || '}
                   {fundData.mango_positions.investor_debts[1].toString()} 
            </p>
           

            
            {/* <p> investor_debts  : 
                            {fundData.mango_positions.investor_debts[0].toString()} ||
                            {fundData.mango_positions.investor_debts[1].toString()} </p>
            <p> perp_markets  : 
                            {fundData.mango_positions.perp_markets[0]} ||
                            {fundData.mango_positions.perp_markets[1]} || 
                            {fundData.mango_positions.perp_markets[2]} || 
                            {fundData.mango_positions.perp_markets[3]} 
                </p> */}

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
                                  <th style={{ width: "15%" }}>mint_authority</th>
                                  <th style={{ width: "15%" }}>is_on_mango</th>
                                  <th style={{ width: "15%" }}>balance</th>
                                  <th style={{ width: "15%" }}>debt</th>
                                  <th style={{ width: "25%" }}>index</th>
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
                        <td >{i?.mint_authority} || { ((ids.tokens).find(j => j.mintKey === i?.mint_authority))?.symbol}</td>
                        <td >{i?.is_on_mango}</td>
                        <td >{i?.balance}</td>
                        <td >{i?.debt}</td>
                        <td >{i?.index[0]} ||  {i?.index[1]} || {i?.index[2]}</td>
                        <td >{i?.mux}</td>
                      </tr>
                    })
                  }
                </tbody>
        </Table>
     }

<hr/>
        Mango account  ::: {' '}
        <input type="text" value={mangoAccount} style={{width :"500px"}} onChange={(event) => setMangoAccount(event.target.value)} />
        <button onClick={getMangoAccountData}>GET  MANGO ACC DATA </button>
        <br />
      {
        mangoAccount && mangoAccountData &&
        <>
         <h4>MANGO ACC :{mangoAccount} </h4>
            <p> mangoGroup : {mangoAccountData.admin}</p>
            <b> borrows :</b>
            <p>
             { 
             mangoAccountData?.borrows?.length &&
             mangoAccountData.borrows.map((i,x)=> <> {x} = {i} <b>_||_</b></>  )
                }
            </p>
           <b> clientOrderIds :</b>
           { mangoAccountData?.clientOrderIds && mangoAccountData?.clientOrderIds.map((i,x)=> <>{x} = {i} <b>_||_</b></>  )}
           <br/>
           <b> deposits :</b>
           { mangoAccountData?.deposits && mangoAccountData?.deposits.map((i,x)=> <>{x} = {i} <b>_||_</b></>  )}
           <br/>
  
           <b> orders :</b>
           { mangoAccountData?.orders && mangoAccountData?.orders.map((i,x)=> <>{x} = {i} <b>_||_</b></>  )}
           <br/>
          
           <b> spotOpenOrders :</b>
           { mangoAccountData?.spotOpenOrders && mangoAccountData?.spotOpenOrders.map((i,x)=> <>{x} = {i} <b>_||_</b></>  )}
           <br/>

           <b>perpAccounts</b>
            {
                 mangoAccountData.perpAccounts?.length &&
                 <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}
                    >
                        <thead className="text-primary">
                                        <tr>
                                        <th style={{ width: "15%" }}>index</th>
                                          <th style={{ width: "15%" }}>asksQuantity</th>
                                          <th style={{ width: "15%" }}>basePosition</th>
                                          <th style={{ width: "15%" }}>bidsQuantity</th>
                                          <th style={{ width: "15%" }}>longSettledFunding</th>
                                          <th style={{ width: "15%" }}>mngoAccrued</th>
                                          <th style={{ width: "15%" }}>quotePosition</th>
                                          <th style={{ width: "15%" }}>shortSettledFunding</th>
                                          <th style={{ width: "15%" }}>takerBase</th>
                                          <th style={{ width: "15%" }}>takerQuote</th>
                                        </tr>
                        </thead>
                        <tbody>
                          {
                            mangoAccountData.perpAccounts && 
                            mangoAccountData.perpAccounts.map((i,x)=>{
                              return <tr key={x}>
                                <td >{x}</td>
                                <td >{i?.asksQuantity}</td>
                                <td >{i?.basePosition}</td>
                                <td >{i?.bidsQuantity}</td>
                                <td >{i?.longSettledFunding}</td>
                                <td >{i?.mngoAccrued}</td>
                                <td >{i?.quotePosition}</td>
                                <td >{i?.shortSettledFunding}</td>
                                <td >{i?.takerBase}</td>
                                <td >{i?.takerQuote}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>
            }
  
        
        </>
      }
      <hr/>
      <h4>MANGO GRP </h4>
      <button onClick={getAllDecodeMangoData}>GET ALL MANGO DATA </button>
      {
        mangoGroup && mangoGroup?.admin && 
          <>
            <p> admin : {mangoGroup.admin}</p>
            <p> dexProgramId : {mangoGroup.dexProgramId}</p>
            <p> insuranceVault : {mangoGroup.insuranceVault}</p>
            <p> mangoCache : {mangoGroup.mangoCache}</p>
            <p> msrmVault : {mangoGroup.msrmVault}</p>
            <p> numOracles : {mangoGroup.numOracles}</p>
            <b>mangoGroup-oracles</b>
            <ul>
            {
              mangoGroup.oracles &&
              mangoGroup.oracles.map((i,x)=> <li key={x}> <b>{x}</b> {i}</li> )
            }
            </ul>
            <b>spotMarkets</b>
            {
                 mangoGroup.spotMarkets?.length &&
                 <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}
                    >
                        <thead className="text-primary">
                                        <tr>
                                        <th style={{ width: "15%" }}>initAssetWeight</th>
                                          <th style={{ width: "15%" }}>initLiabWeight</th>
                                          <th style={{ width: "15%" }}>liquidationFee</th>
                                          <th style={{ width: "15%" }}>maintAssetWeight</th>
                                          <th style={{ width: "15%" }}>maintLiabWeight</th>
                                          <th style={{ width: "15%" }}>spotMarket</th>
                                        </tr>
                        </thead>
                        <tbody>
                          {
                            mangoGroup.spotMarkets && 
                            mangoGroup.spotMarkets.map((i,x)=>{
                              return <tr key={x}>
                                <td >{i?.initAssetWeight}</td>
                                <td >{i?.initLiabWeight}</td>
                                <td >{i?.liquidationFee}</td>
                                <td >{i?.maintAssetWeight}</td>
                                <td >{i?.maintLiabWeight}</td>
                                <td >{i?.spotMarket}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>
            }
            <b>ALL perp Markets</b>
            {
                 mangoGroup.perpMarkets?.length &&
                 <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}
                    >
                        <thead className="text-primary">
                                        <tr>
                                        <th style={{ width: "15%" }}>index</th>
                                        <th style={{ width: "15%" }}>perpMarket</th>
                                        <th style={{ width: "15%" }}>baseLotSize</th>
                                          <th style={{ width: "15%" }}>initAssetWeight</th>
                                          <th style={{ width: "15%" }}>initLiabWeight</th>
                                          <th style={{ width: "15%" }}>liquidationFee</th>
                                          <th style={{ width: "15%" }}>maintAssetWeight</th>
                                          <th style={{ width: "15%" }}>maintLiabWeight</th>
                                          <th style={{ width: "15%" }}>makerFee</th>
                                          <th style={{ width: "15%" }}>quoteLotSize</th>
                                          <th style={{ width: "15%" }}>takerQuote</th>
                                          <th style={{ width: "15%" }}>takerFee</th>
                                        </tr>
                        </thead>
                        <tbody>
                          {
                            mangoGroup.perpMarkets && 
                            mangoGroup.perpMarkets.map((i,x)=>{
                              return <tr key={x}>
                                <td >{x}</td>
                                <td >{i?.perpMarket}</td>
                                <td >{i?.baseLotSize}</td>
                                <td >{i?.initAssetWeight}</td>
                                <td >{i?.initLiabWeight}</td>
                                <td >{i?.liquidationFee}</td>
                                <td >{i?.maintAssetWeight}</td>
                                <td >{i?.maintLiabWeight}</td>
                                <td >{i?.makerFee}</td>
                                <td >{i?.quoteLotSize}</td>
                                <td >{i?.takerFee}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>
            }

            <b>TOKENS</b>
             {
                 mangoGroup.tokens?.length &&
                 <Table  className="tablesorter" responsive width="100%" style={{ overflow: 'hidden !important', textAlign: 'center' }}
                    >
                        <thead className="text-primary">
                                        <tr>
                                          <th style={{ width: "15%" }}>mint</th>
                                          <th style={{ width: "15%" }}>rootBank</th>
                                          <th style={{ width: "15%" }}>decimals</th>
                                        </tr>
                        </thead>
                        <tbody>
                          {
                            mangoGroup.tokens && 
                            mangoGroup.tokens.map((i,x)=>{
                              return <tr key={x}>
                                <td >{i?.mint}</td>
                                <td >{i?.rootBank}</td>
                                <td >{i?.decimals}</td>
                              </tr>
                            })
                          }
                        </tbody>
                </Table>
            }
           </> 
      }
      <h5>USDC NODE BANK {ids.tokens[0].nodeKeys[0]}</h5>
      {
      nodeBank && nodeBank?.borrows &&
        <>
            <p> borrows : {nodeBank.borrows}</p>
            <p> deposits : {nodeBank.deposits}</p>
            <p> vault : {nodeBank.vault}</p>
        </>
      }
      <h5>USDC ROOT BANK {ids.tokens[0].rootKey}</h5>
      {
       rootBank && rootBank?.depositIndex &&
        <>
            <p> depositIndex : {rootBank.depositIndex}</p>
            <p> borrowIndex : {rootBank.borrowIndex}</p>
            <p> lastUpdated : {rootBank.lastUpdated}</p>
            <p> maxRate : {rootBank.maxRate}</p>
            <p> optimalUtil : {rootBank.optimalUtil}</p>
            <p> optimalRate : {rootBank.optimalRate}</p>
        </>
      }

  </div>
  )
}

