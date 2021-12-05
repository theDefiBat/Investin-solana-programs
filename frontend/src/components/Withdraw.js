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

  const [amount, setAmount] = useState(0);

  const [fundPDA, setFundPDA] = useState('')
  const [fundStateAccount, setFundStateAccount] = useState('')
  const [fundPerf, setFundPerf] = useState(0);
  const [startPerf, setStartPerf] = useState(0);
  const [invShare, setInvShare] = useState(0);
  const [funds, setFunds] = useState([]);


  const [fundBalances, setFundBalances] = useState([])
  const [withdrawableAmount, setWithdrawableAmount] = useState(0)

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const fundAccount = GlobalState.useState(s => s.createFundPublicKey);

  const handleWithdrawSettle = async () => {
    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    };

    if(!fundStateAccount) {
      alert("no funds found")
      return
    }
    console.log("fundStateAcc::: ", fundStateAccount)
    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, fundPDA.substr(0, 31), INVESTOR_DATA.span)

    const accountInfo = await connection.getAccountInfo(new PublicKey(fundStateAccount));
    const fund_data = FUND_DATA.decode(accountInfo.data);

    let margin_account_1 = fund_data.mango_positions[0].margin_account;
    let margin_account_2 = fund_data.mango_positions[1].margin_account;

    let open_orders_1 = PublicKey.default
    let oracle_acc_1 = PublicKey.default
    let is_active = false
    if (margin_account_1 != PublicKey.default && fund_data.mango_positions[0].state != 0) {
      let margin_info = await connection.getAccountInfo(margin_account_1)
      let margin_data = MarginAccountLayout.decode(margin_info.data)
      let mango_info = await connection.getAccountInfo(MANGO_GROUP_ACCOUNT)
      let mango_data = MangoGroupLayout.decode(mango_info.data)

      let index = fund_data.mango_positions[0].margin_index
      open_orders_1 = margin_data.openOrders[index]
      oracle_acc_1 = mango_data.oracles[index]
    }
    let open_orders_2 = PublicKey.default
    let oracle_acc_2 = PublicKey.default
    if (margin_account_2 != PublicKey.default && fund_data.mango_positions[1].state != 0) {
      let margin_info = await connection.getAccountInfo(margin_account_2)
      let margin_data = MarginAccountLayout.decode(margin_info.data)
      let mango_info = await connection.getAccountInfo(MANGO_GROUP_ACCOUNT)
      let mango_data = MangoGroupLayout.decode(mango_info.data)

      let index = fund_data.mango_positions[1].margin_index
      open_orders_2 = margin_data.openOrders[index]
      oracle_acc_2 = mango_data.oracles[index]
    }
  
    const transaction = new Transaction()
    
    updatePoolPrices(transaction, devnet_pools)

    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 4
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: platformStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true },
        { pubkey: investerStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: key, isSigner: true, isWritable: true },

        { pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable: true },
        {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable:true},

        { pubkey: margin_account_1, isSigner: false, isWritable: true },
        { pubkey: margin_account_2, isSigner: false, isWritable: true },
        { pubkey: open_orders_1, isSigner: false, isWritable: true },
        { pubkey: open_orders_2, isSigner: false, isWritable: true },
        { pubkey: oracle_acc_1, isSigner: false, isWritable: true },
        { pubkey: oracle_acc_2, isSigner: false, isWritable: true },

      ],
      programId,
      data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

  }
  const handleWithdraw = async () => {

    console.log(`[...getPoolAccounts()] ::: `, [...getPoolAccounts()])

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };

    if(!fundStateAccount) {
      alert("no funds found")
      return
    }
    console.log("fundStateAcc::: ", fundStateAccount)

    const RPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);

    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, fundPDA.substr(0, 31), INVESTOR_DATA.span)
    
    const transaction = new Transaction()

    // updatePoolPrices(transaction, devnet_pools)

    const routerAssociatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), RPDA[0], transaction);

    const investorBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), key, transaction);
    const investorTokenAccount2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[4].mintKey), key, transaction);
    // const investorTokenAccount3 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[6].mintKey), key, transaction);

    const fundAssociatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), new PublicKey(fundPDA), transaction);
    const fundAssociatedTokenAddress2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[4].mintKey), new PublicKey(fundPDA), transaction);
    // const fundAssociatedTokenAddress3 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[6].mintKey), MPDA, transaction);

    console.log("USDC vault:: ", fundAssociatedTokenAddress1)
    console.log("SRM vault:: ", fundAssociatedTokenAddress2)

    const dataLayout = struct([u8('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 3,
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: platformStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: fundStateAccount, isSigner: false, isWritable: true },
        { pubkey: investerStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: key, isSigner: true, isWritable: true },

        { pubkey: routerAssociatedTokenAddress, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: new PublicKey(fundPDA), isSigner: false, isWritable: false },
        { pubkey: RPDA[0], isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },

        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true }, // Investor Token Accounts
        { pubkey: investorTokenAccount2, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },


        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true }, // Fund Token Accounts
        { pubkey: fundAssociatedTokenAddress2, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },

      ],
      programId,
      data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash("finalized");
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

  }
  
  const handleFunds = async () => {
    const key = walletProvider?.publicKey;
    let invFunds = []
    const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
    const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
    console.log("platformData :: ", platformData)
  
    let funds = await connection.getProgramAccounts(programId, { filters: [{ dataSize: FUND_DATA.span }] });
    console.log(`funds :::: `, funds)
    const fundData = funds.map(f => FUND_DATA.decode(f.account.data))

    console.log(`decodedFunds ::: `, fundData)
    
    for(let i=0; i<fundData.length; i++) {
      let manager = fundData[i].manager_account;

      let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
      let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);

      let invStateAccount = await PublicKey.createWithSeed(key, PDA[0].toBase58().substr(0, 31), programId);
      let invState = await connection.getAccountInfo(invStateAccount);

      if (invState == null) {
        continue
      }

      let invStateData = INVESTOR_DATA.decode(invState.data)
      console.log(invStateData)

      // if (!invStateData.is_initialized) {
      //   continue
      // }
      invFunds.push({
        fundPDA: PDA[0].toBase58(),
        fundManager: manager.toBase58(),
        fundStateAccount: fundState.toBase58()
      });
    }
    console.log(invFunds)
    setFunds(invFunds);
  }

  const handleFundSelect = async(event) => {
  
    setFundPDA(event.target.value);
    funds.forEach(fund => {
      if (fund.fundPDA == event.target.value) 
      {setFundStateAccount(fund.fundStateAccount)
       console.log("set fundStateAcoount")}
    });
    console.log(`setting fundPDA :::: `, fundPDA)
    console.log(`setting fundStateAccount :::: `, fundStateAccount)
  }
  
  const handleGetInvestments = async () => {
    const key = walletProvider?.publicKey;  
    if (!key ) {
      alert("connect wallet")
      return;
    }
    if (!fundPDA) {
      alert('no fund found')
      return;
    }
    const investorStateAccount = await PublicKey.createWithSeed(
      key,
      fundPDA.substr(0, 31),
      programId,
    );

    let x = await connection.getAccountInfo(investorStateAccount)
    if (x == null)
    {
      alert("investor account not found for selected fund")
      return
    }
    let invState = INVESTOR_DATA.decode(x.data)
    // if (!invState.is_initialized) {
    //   alert("investor data not initialized!")
    //   return
    // }
    console.log(invState);
    
    let y = await connection.getAccountInfo(new PublicKey(fundStateAccount))
    if (y == null)
    {
      alert("investor account not found")
      return
    }
    let fundState = FUND_DATA.decode(y.data);
    console.log(fundState)
    setFundPerf(parseInt(fundState.prev_performance));
    setStartPerf(parseInt(invState.start_performance));
    setWithdrawableAmount(((parseInt(invState.amount) * 0.98) / (10 ** fundState.tokens[0].decimals)) 
      *(parseInt(fundState.prev_performance) / parseInt(invState.start_performance))
    );
    let share_ratio = ((parseInt(invState.amount) * 0.98) / parseInt(fundState.total_amount)) * 
      (parseInt(fundState.prev_performance) / parseInt(invState.start_performance))
    
    setInvShare(share_ratio)

    let bal = []
    bal.push((parseInt(fundState.tokens[0].balance)/ (10**fundState.tokens[0].decimals)) * share_ratio)
    bal.push((parseInt(fundState.tokens[1].balance)/ (10**fundState.tokens[1].decimals)) * share_ratio)
    bal.push((parseInt(fundState.tokens[2].balance)/ (10**fundState.tokens[2].decimals)) * share_ratio)
    setFundBalances(bal)
  }

  const handleMangoWithdrawInvestor = async () => {
    
      const key = walletProvider?.publicKey;

      if (!key) {
        alert("connect wallet")
        return;
      };
  
      if(!fundStateAccount) {
        alert("no funds found")
        return
      }
      console.log("fundStateAcc::: ", fundStateAccount)
    
      const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, fundPDA.substr(0, 31), INVESTOR_DATA.span)
      

    if (fundStateAccount == ''){
      alert("get fund info first!")
      return
    }

    let fund_data = FUND_DATA.decode((await connection.getAccountInfo(new PublicKey(fundStateAccount))).data)
    let inv_data = INVESTOR_DATA.decode((await connection.getAccountInfo(investerStateAccount)).data)


    console.log("fund_data:: ", fund_data)
    console.log("inv_data:: ", inv_data)

    for(let i = 0; i<2; i++) {
      const transaction = new Transaction()

      if (inv_data.margin_debt[i] == 0) {continue}

      let index = inv_data.margin_position_id[i] == fund_data.mango_positions[0].position_id ? 0 : 1
      let side = fund_data.mango_positions[index].position_side == 0 ? 'sell' : 'buy'
      let market_index = fund_data.mango_positions[index].margin_index
      let margin_account_acc = fund_data.mango_positions[index].margin_account
      console.log("market index:: ", market_index)
      await mangoWithdrawInvestor(connection, margin_account_acc, new PublicKey(fundStateAccount), investerStateAccount, new PublicKey(fundPDA), walletProvider, market_index, side, 10, null, transaction, side == 'buy' ? 0 : NUM_MARKETS)

      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      const sign = await signAndSendTransaction(walletProvider, transaction);
      console.log("signature tx:: ", sign)
      console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

    }

      
      
}

  return (
    <div className="form-div">
      <h4>Withdraw</h4>
      
      <select name="funds" width = "100px" onClick={handleFundSelect}>
        {
          funds.map((fund) => {
            return (<option key={fund.fundPDA} value={fund.fundPDA}>{fund.fundPDA}</option>)
          })
        }
      </select>
      <button onClick={handleFunds}>Load Investments</button>
      <br />
      <br />
     
      <button onClick={handleWithdrawSettle}>Settle Withdraws</button>
      <button onClick={handleGetInvestments}>GetInvestments</button>
  
      <br />
      Assets Info::
      <br />
      withdrawableAmount:: {withdrawableAmount}
      <br />
      fund Address :: {fundPDA}
      <br />
      fund performance:: {fundPerf}
      <br />
      inv start performance :: {startPerf}
      <br/>
      inv share ratio:: {invShare}
      <br />
      USDR balance: {fundBalances[0]}
      <br />
      RAYT balance: {fundBalances[1]}
      <br />
      ALPHA balance: {fundBalances[2]}
      <br />
      <button onClick={handleWithdraw}>Withdraw from Fund</button>
      <button onClick={handleMangoWithdrawInvestor}>Withdraw from Margin Account</button>
    </div>
  )

}