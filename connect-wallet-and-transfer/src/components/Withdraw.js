import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, SOL_USDC_MARKET, connection,  platformStateAccount, priceStateAccount, FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID , CLOCK_PROGRAM_ID, MANGO_PROGRAM_ID_V2, MANGO_GROUP_ACCOUNT, MANGO_VAULT_ACCOUNT_USDC, MARGIN_ACCOUNT_KEY, ORACLE_BTC_DEVNET, ORACLE_ETH_DEVNET, ORACLE_SOL_DEVNET, ORACLE_SRM_DEVNET} from '../utils/constants';

import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { devnet_pools } from '../utils/pools';
import { keyBy } from 'lodash';
import { INVESTOR_DATA, PLATFORM_DATA, FUND_DATA } from '../utils/programLayouts';
import { TEST_TOKENS } from '../utils/tokens';
import { updatePoolPrices } from './updatePrices';
import { MarginAccountLayout, NUM_MARKETS, MangoGroupLayout } from '../utils/MangoLayout';
import { MANGO_TOKENS } from '../utils/tokens'
import { placeOrder, placeOrder2 } from '../utils/mango';


const getPoolAccounts = () => {
  return devnet_pools.map((p) => {
    return [
      { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
      { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
    ]
  })
}

export const Withdraw = () => {

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

    let margin_account = fund_data.mango_positions[0].margin_account;
    let open_orders = PublicKey.default
    let oracle_acc = PublicKey.default
    let is_active = false
    if (margin_account != PublicKey.default && fund_data.mango_positions[0].state != 0) {
      let margin_info = await connection.getAccountInfo(margin_account)
      let margin_data = MarginAccountLayout.decode(margin_info.data)
      let mango_info = await connection.getAccountInfo(MANGO_GROUP_ACCOUNT)
      let mango_data = MangoGroupLayout.decode(mango_info.data)
      console.log("margin_data:: ", margin_data)

      let index = fund_data.mango_positions[0].margin_index
      open_orders = margin_data.openOrders[index]
      oracle_acc = mango_data.oracles[index]
    }
    const transaction = new Transaction()

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
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true },
        { pubkey: investerStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: key, isSigner: true, isWritable: true },

        {pubkey: priceStateAccount, isSigner: false, isWritable:true},

        { pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable: true },
        { pubkey: margin_account, isSigner: false, isWritable: true },
        { pubkey: open_orders, isSigner: false, isWritable: true },
        { pubkey: oracle_acc, isSigner: false, isWritable: true },

        {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable:true}
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

    const routerAssociatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(MANGO_TOKENS['USDC'].mintAddress), RPDA[0], transaction);

    const investorBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(MANGO_TOKENS['USDC'].mintAddress), key, transaction);
    // const investorTokenAccount2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['RAYT'].mintAddress), key, transaction);
    // const investorTokenAccount3 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['ALPHA'].mintAddress), key, transaction);

    const fundAssociatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(MANGO_TOKENS['USDC'].mintAddress), new PublicKey(fundPDA), transaction);
    // const fundAssociatedTokenAddress2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['RAYT'].mintAddress), MPDA, transaction);
    // const fundAssociatedTokenAddress3 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['ALPHA'].mintAddress), MPDA, transaction);

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
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true },


        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true }, // Fund Token Accounts
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true },
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

      if (!invStateData.is_initialized) {
        continue
      }
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
    if (!invState.is_initialized) {
      alert("investor data not initialized!")
      return
    }
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

    if (!key ) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    const  fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    let mango_group_acc = await connection.getAccountInfo(MANGO_GROUP_ACCOUNT)
    let mango_data = MangoGroupLayout.decode(mango_group_acc.data)

    const signer_acc = mango_data.signerKey;

    const invBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(MANGO_TOKENS['USDC'].mintAddress));
    const margin_account_acc = await createKeyIfNotExists(walletProvider, "", MANGO_PROGRAM_ID_V2, MARGIN_ACCOUNT_KEY, MarginAccountLayout.span, transaction)
    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, fundPDA[0].toBase58().substr(0, 32), INVESTOR_DATA.span)

    console.log("margin_acc::", margin_account_acc)
    let margin_data = await connection.getAccountInfo(margin_account_acc);
    let margin_dec = MarginAccountLayout.decode(margin_data.data)
    console.log("margin data", margin_dec)

    if (fundStateAccount == ''){
      alert("get fund info first!")
      return
    }

    let x = await connection.getAccountInfo(margin_account_acc)
      if (x == null)
      {
          alert("margin account not found")
          return
      }
      console.log(x)
      let marginStateAccount = MarginAccountLayout.decode(x.data)

    let open_order_acc = []
    open_order_acc.push(marginStateAccount.openOrders[0])
    open_order_acc.push(marginStateAccount.openOrders[1])
    open_order_acc.push(marginStateAccount.openOrders[2])
    open_order_acc.push(marginStateAccount.openOrders[3])

    let side = marginStateAccount.deposits[0] > 100 ? 'sell' : 'buy';
    console.log("side:: ", side)

    //await placeOrder2(connection, margin_account_acc, fundStateAccount, fundPDA[0], walletProvider, SOL_USDC_MARKET, side, 0.01, null, transaction, side == 'buy' ? 0 : NUM_MARKETS)

      transaction.feePayer = key;
      let hash = await connection.getRecentBlockhash();
      console.log("blockhash", hash);
      transaction.recentBlockhash = hash.blockhash;

      //const sign = await signAndSendTransaction(walletProvider, transaction);
      //console.log("signature tx:: ", sign)

      const transaction1 = new Transaction()
    const dataLayout = struct([u8('instruction'), nu64('token_index')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 13,
        //token_index: side == 'buy' ? 0 : NUM_MARKETS
        token_index: 4
      },
      data
    )
    const instruction = new TransactionInstruction({
      keys: [
          {pubkey: fundStateAccount, isSigner: false, isWritable: true},
          {pubkey: investerStateAccount, isSigner: false, isWritable: true },
          {pubkey: key, isSigner: true, isWritable: true },
          {pubkey: fundPDA[0], isSigner: false, isWritable: true },
          {pubkey: MANGO_PROGRAM_ID_V2, isSigner: false, isWritable:true},

          {pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable:true},
          {pubkey: margin_account_acc, isSigner: false, isWritable:true},
          {pubkey: invBaseTokenAccount, isSigner: false, isWritable:true},
          {pubkey: MANGO_VAULT_ACCOUNT_USDC, isSigner: false, isWritable:true},
          // TODO: signer_acc
          {pubkey: signer_acc, isSigner: false, isWritable:true},
          {pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable:true},
          {pubkey: CLOCK_PROGRAM_ID, isSigner: false, isWritable:true},
          // open order account
          {pubkey: open_order_acc[0], isSigner: false, isWritable:true},
          {pubkey: open_order_acc[1], isSigner: false, isWritable:true},
          {pubkey: open_order_acc[2], isSigner: false, isWritable:true},
          {pubkey: open_order_acc[3], isSigner: false, isWritable:true},
          // oracle accounts
          {pubkey: ORACLE_BTC_DEVNET, isSigner: false, isWritable:true},
          {pubkey: ORACLE_ETH_DEVNET, isSigner: false, isWritable:true},
          {pubkey: ORACLE_SOL_DEVNET, isSigner: false, isWritable:true},
          {pubkey: ORACLE_SRM_DEVNET, isSigner: false, isWritable:true},
      ],
    programId,
    data
    });
  
      transaction1.add(instruction);
      transaction1.feePayer = key;
      let hash1 = await connection.getRecentBlockhash();
      console.log("blockhash", hash1);
      transaction1.recentBlockhash = hash1.blockhash;

      const sign1 = await signAndSendTransaction(walletProvider, transaction1);
      console.log("signature tx:: ", sign1)
      
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