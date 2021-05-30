import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId, INVESTOR_ACCOUNT_KEY, FUND_ACCOUNT_KEY, platformStateAccount, adminAccount, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { devnet_pools } from '../utils/pools';
import { keyBy } from 'lodash';
import { INVESTOR_DATA, PLATFORM_DATA, FUND_DATA } from '../utils/programLayouts';


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


  const [fundBalances, setFundBalances] = useState([])
  const [withdrawableAmount, setWithdrawableAmount] = useState(0)

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const fundAccount = GlobalState.useState(s => s.createFundPublicKey);

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
    const MPDA = new PublicKey(fundPDA)

    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, INVESTOR_ACCOUNT_KEY, INVESTOR_DATA.span)
    
    const transaction = new Transaction()

    const routerAssociatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), RPDA[0]);
    // TODO: Manager Base Token Account
    const managerAssociatedTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), RPDA[0]);
    // TODO: Investin Base Token Account
    const investinAssociatedTokenAddress = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), adminAccount);



    const investorBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), key);
    const investorTokenAccount2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o'), key);
    const investorTokenAccount3 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty'), key);

    const fundAssociatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), MPDA);
    const fundAssociatedTokenAddress2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o'), MPDA);
    const fundAssociatedTokenAddress3 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey('HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty'), MPDA);

    const dataLayout = struct([u8('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 3,
        amount: amount * 1000000000
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
        { pubkey: managerAssociatedTokenAccount, isSigner: false, isWritable: true }, // Manager Base Token Account
        { pubkey: investinAssociatedTokenAddress, isSigner: false, isWritable: true }, // Investin Base Token Account
        
        { pubkey: MPDA, isSigner: false, isWritable: false },
        { pubkey: RPDA[0], isSigner: false, isWritable: false },

        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },

        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true }, // Investor Token Accounts
        { pubkey: investorTokenAccount2, isSigner: false, isWritable: true },
        { pubkey: investorTokenAccount3, isSigner: false, isWritable: true },

        { pubkey: fundAssociatedTokenAddress1, isSigner: false, isWritable: true }, // Fund Token Accounts
        { pubkey: fundAssociatedTokenAddress2, isSigner: false, isWritable: true },
        { pubkey: fundAssociatedTokenAddress3, isSigner: false, isWritable: true },

        // TODO : send pool token accounts 
        ...getPoolAccounts().flat()
      ],
      programId,
      data
    });

    transaction.add(instruction);
    console.log(`transaction ::: `, transaction)
    console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey.toBase58())
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("tx::: ", sign)
  }

  const handleGetInvestments = async () => {
    const key = walletProvider?.publicKey;  
    if (!key ) {
      alert("connect wallet")
      return;
    }
    const investorStateAccount = await PublicKey.createWithSeed(
      key,
      INVESTOR_ACCOUNT_KEY,
      programId,
    );

    let x = await connection.getAccountInfo(investorStateAccount)
    if (x == null)
    {
      alert("investor account not found")
      return
    }
    let invState = INVESTOR_DATA.decode(x.data)
    if (!invState.is_initialized) {
      alert("investor data not initialized!")
      return
    }
    console.log(invState);

    let fundAddress = invState.manager.toString()
    console.log("fund address:: ", fundAddress)
    setFundPDA(fundAddress)
    
    const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
    const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
    console.log("platformData :: ", platformData)
  
    let fundStateAcc = null
    let fundManager = null
    for(let i=0; i<platformData.no_of_active_funds; i++) {
      let manager = platformData.fund_managers[i];
      let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
      
      if (fundAddress == PDA[0].toBase58()) {
        fundStateAcc = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);
        fundManager = manager;
        setFundStateAccount(fundStateAcc)
        console.log("PDA has matched")
        console.log("fundState:: ", fundStateAcc.toBase58())
        console.log("fundManager:: ", fundManager.toBase58())
      }
    }
    if(!fundStateAcc) {
      alert("no funds found")
      return
    }
    let y = await connection.getAccountInfo(fundStateAcc)
    if (y == null)
    {
      alert("investor account not found")
      return
    }
    let fundState = FUND_DATA.decode(y.data);
    console.log(fundState)
    setFundPerf(parseInt(fundState.prev_performance) / (10000));
    setStartPerf(parseInt(invState.start_performance) / (10000));
    setWithdrawableAmount(((parseInt(invState.amount) * 0.98) / (10 ** fundState.tokens[0].decimals)) 
      *(parseInt(fundState.prev_performance) / parseInt(invState.start_performance))
    );
    let share_ratio = ((parseInt(invState.amount) * 0.98) / parseInt(fundState.total_amount)) * 
      (parseInt(fundState.prev_performance) / parseInt(invState.start_performance))
    
    setInvShare(share_ratio)

    let bal = []
    bal.push((parseInt(fundState.tokens[0].balance)/ (10**fundState.tokens[0].decimals)) * share_ratio)
    bal.push((parseInt(fundState.tokens[1].balance)/ (10**fundState.tokens[1].decimals)) * share_ratio)
    bal.push((parseInt(fundState.tokens[1].balance)/ (10**fundState.tokens[2].decimals)) * share_ratio)
    setFundBalances(bal)
  }

  return (
    <div className="form-div">
      <h4>Withdraw</h4>
      amount ::: {' '}
      <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
      <br />
      
      <button onClick={handleWithdraw}>Withdraw All</button>
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
      USDP balance: {fundBalances[0]}
      <br />
      ALPHA balance: {fundBalances[1]}
      <br />
      BETA balance: {fundBalances[2]}
    </div>
  )

}