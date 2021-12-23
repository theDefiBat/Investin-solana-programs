import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState, useEffect } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId, platformStateAccount, FUND_ACCOUNT_KEY, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA, PLATFORM_DATA } from '../utils/programLayouts';
import { TOKENS } from '../utils/tokens'
import { devnet_pools } from '../utils/pools'
import { updatePoolPrices } from './updatePrices';
import { mangoWithdrawInvestor } from '../utils/mango';
import { NUM_MARKETS } from '../utils/MangoLayout';

export const Deposit = () => {

  const [amount, setAmount] = useState(0);
  const [fundPDA, setFundPDA] = useState('');
  const [fundStateAccount, setFundStateAccount] = useState('');
  const [funds, setFunds] = useState([]);
  const [address, setAddress] = useState('')
  const [invStateData, setInvStateData] = useState(null)


  const walletProvider = GlobalState.useState(s => s.walletProvider);

  useEffect(() => {
    if(!address){
      return;
    }

    (async () => {
      const investment = await connection.getAccountInfo(new PublicKey(address))
      if(investment != null) {
        const invStateData = INVESTOR_DATA.decode(investment.data)
        console.log("invStateData::",invStateData)
        setInvStateData(invStateData)
      }
     
    })()
   
  }, [address])


  const handleDeposit = async () => {

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };
  
    const baseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(TOKENS['USDC'].mintAddress));

    const transaction = new Transaction()

    const RPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const FPDA = new PublicKey(fundPDA);

    const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TOKENS['USDC'].mintAddress), RPDA[0], transaction);    

    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, FPDA.toBase58().substr(0, 31), INVESTOR_DATA.span, transaction)
    
    // await updatePoolPrices(transaction, devnet_pools)
    
    console.log("RPDA:", RPDA[0].toBase58())
    console.log("FPDA: ", FPDA.toBase58())
    console.log("fundStateAccountRead:: ", fundStateAccount)
    console.log("baseTokenAccount:: ", baseTokenAccount)

    console.log("investorStateAccountRead:: ", investerStateAccount.toBase58())

    console.log("account size::: ", INVESTOR_DATA.span)
    console.log("associatedTokenaccount:: ", associatedTokenAddress1.toBase58())

    const dataLayout = struct([u8('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1,
        amount: amount * ( 10 ** TOKENS['USDC'].decimals)
      },
      data
    )

    // DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w
    // HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o
    // HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true }, //fund State Account
        { pubkey: investerStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: baseTokenAccount, isSigner: false, isWritable: true }, // Investor Base Token Account
        { pubkey: associatedTokenAddress1, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true }
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
    console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

    // const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
    // const signature = await signAndSendTransaction(walletProvider, transaction2);
    // let result = await connection.confirmTransaction(signature, "confirmed");
    // console.log("tx:: ", signature)
    
    // transaction.add(deposit_instruction);
    // transaction.feePayer = key;
    // let hash = await connection.getRecentBlockhash();
    // console.log("blockhash", hash);
    // transaction.recentBlockhash = hash.blockhash;

    // const sign = await signAndSendTransaction(walletProvider, transaction);
    // console.log("signature tx:: ", sign)


  // const investorDataAcc = await connection.getAccountInfo(investerStateAccount);
  // const investorData = INVESTOR_DATA.decode(investorDataAcc.data);
  }

  const handleMangoWithdrawInvestor = async () => {

    console.log("----handleMangoWithdrawInvestor---:",address)
    
    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };


    const manager = invStateData?.manager?.toBase58();
    // const fundAccount = await createKeyIfNotExists(walletProvider, "", programId, FUND_ACCOUNT_KEY, FUND_DATA.span, transaction)
    if(!manager) {
      alert("no manager")
      return
    }
    console.log("manager::",manager)
    const fundStateAccount = await PublicKey.createWithSeed(
      invStateData?.manager,
      FUND_ACCOUNT_KEY,
      programId,
    );
    console.log("fundStateAcc::: ", fundStateAccount)

    if(!fundStateAccount) {
      alert("no funds found")
      return
    }
      

  if (fundStateAccount == ''){
    alert("get fund info first!")
    return
  }

  let fund_data = FUND_DATA.decode((await connection.getAccountInfo(new PublicKey(fundStateAccount))).data)
  // let inv_data = INVESTOR_DATA.decode((await connection.getAccountInfo(address)).data)


  console.log("fund_data:: ", fund_data)
  // console.log("inv_data:: ", inv_data)

  for(let i = 0; i<2; i++) {
    const transaction = new Transaction()

    if (invStateData.margin_debt[i] == 0) {continue}

    let index = invStateData.margin_position_id[i] == fund_data.mango_positions[0].position_id ? 0 : 1
    let side = fund_data.mango_positions[index].position_side == 0 ? 'sell' : 'buy'
    let market_index = fund_data.mango_positions[index].margin_index
    let margin_account_acc = fund_data.mango_positions[index].margin_account
    console.log("market index:: ", market_index)
    await mangoWithdrawInvestor(connection, margin_account_acc, new PublicKey(fundStateAccount), address, fund_data.fund_pda, walletProvider, market_index, side, 10, null, transaction, side == 'buy' ? 0 : NUM_MARKETS)

    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

  }

}


    
  const handleFunds = async () => {
    let managers = []
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
      console.log(`PDA[0]`, PDA)
      managers.push({
        fundPDA: PDA[0].toBase58(),
        fundManager: manager.toBase58(),
        fundStateAccount: fundState.toBase58()
      });
    }
    console.log(managers)
    setFunds(managers);
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

  return (
    <div className="form-div">
      <h4>Deposit</h4>
      amount ::: {' '}
      <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
      <br />
      <label htmlFor="funds">Select Fund Address:</label>

      <select name="funds" width = "100px" onClick={handleFundSelect}>
        {
          funds.map((fund) => {
            return (<option key={fund.fundPDA} value={fund.fundPDA}>{fund.fundPDA}</option>)
          })
        }
      </select>
      <button onClick={handleDeposit}>Deposit</button>
      <button onClick={handleFunds}>Load Funds</button>
      <br/>
      <hr/>
      <br/>
       investor State Acc ::: {' '}<input type="text" style={{width :"500px"}} value={address} onChange={(event) => setAddress(event.target.value)} />
               
                 <p >manager: {invStateData?.manager?.toBase58()}</p>
                 <p >owner: {invStateData?.owner?.toBase58()}</p>
                 <p>amount: {invStateData?.amount?.toString()/10**6}</p>
                 <p>amount_in_router: {invStateData?.amount_in_router?.toString()/10**6}</p>
                 <p>start_performance :{invStateData?.start_performance?.toString()}</p>

                 <p>is_initialized:{invStateData?.is_initialized}</p>
                 <p>has_withdrawn:{invStateData?.has_withdrawn}</p>
                 <p>withdrawn_from_margin:{invStateData?.withdrawn_from_margin}</p>

                 <p>margin_debt:{`${invStateData?.margin_debt[0]} <==>  ${invStateData?.margin_debt[1]}`}</p>
                 <p>margin_position_ids: {`${invStateData?.margin_position_id[0]} <==>  ${invStateData?.margin_position_id[1]}`}</p>

                 <p>token_debts: {invStateData?.token_debts.toString()}</p>
                 <p>token_indexes:{invStateData?.token_indexes.toString()}</p>
       <button onClick={handleMangoWithdrawInvestor}>withdraw from margin</button>
      <button onClick={handleFunds}> Final Withdraw</button>
    </div>
  )
}
