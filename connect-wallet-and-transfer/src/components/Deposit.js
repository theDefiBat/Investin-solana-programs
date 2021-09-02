import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId, platformStateAccount, FUND_ACCOUNT_KEY, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u32 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA } from '../utils/programLayouts';
import { IDS, MangoClient } from '@blockworks-foundation/mango-client';

export const Deposit = () => {

  const [amount, setAmount] = useState(0);
  const [fundPDA, setFundPDA] = useState('');
  const [fundStateAccount, setFundStateAccount] = useState('');
  const [funds, setFunds] = useState([]);


  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const ids = IDS['groups'][0]



  const handleDeposit = async () => {

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };
  
    let fundStateInfo = await connection.getAccountInfo(new PublicKey(fundStateAccount))
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))

    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    console.log("mango group:: ", mangoGroup)

    const transaction = new Transaction()
  
    const invBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(ids.tokens[0].mintKey));
    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, fundPDA.substr(0, 31), INVESTOR_DATA.span, transaction)

    console.log("account size::: ", INVESTOR_DATA.span)

    const dataLayout = struct([u32('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1,
        amount: amount * ( 10 ** ids.tokens[0].decimals)
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true }, //fund State Account
        { pubkey: investerStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: invBaseTokenAccount, isSigner: false, isWritable: true }, // Investor Base Token Account
        { pubkey: fundState.vault_key, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: new PublicKey('2C7AtpEbcdfmDzh5g4cFBzCXbgZJmxhY2bWPMi7QKqBH'), isSigner: false, isWritable: true },

        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
        { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
        { pubkey: mangoGroup.mangoCache, isSigner: false, isWritable: true },


        
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true }
      ],
      programId,
      data
    });
    transaction.add(instruction)
    transaction.feePayer = walletProvider?.publicKey;
    let hash = await connection.getRecentBlockhash();
    console.log("tx", transaction);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
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
    
  const handleFunds = async () => {
    let managers = []

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
    </div>
  )
}
