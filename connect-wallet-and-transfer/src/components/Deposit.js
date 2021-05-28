import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId, platformStateAccount, FUND_ACCOUNT_KEY, INVESTOR_ACCOUNT_KEY, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { INVESTOR_DATA, PLATFORM_DATA } from '../utils/programLayouts';
import { TEST_TOKENS } from '../utils/tokens'

export const Deposit = () => {

  const [amount, setAmount] = useState(0);
  const [fundPDA, setFundPDA] = useState('');
  const [fundStateAccount, setFundStateAccount] = useState('');
  const [funds, setFunds] = useState([]);


  const walletProvider = GlobalState.useState(s => s.walletProvider);


  const handleDeposit = async () => {

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };
  
    const baseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey(TEST_TOKENS['USDP'].mintAddress));

    const transaction = new Transaction()

    const RPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const FPDA = new PublicKey(fundPDA);

    const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(TEST_TOKENS['USDP'].mintAddress), RPDA[0], transaction);    

    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, "investorAcc", INVESTOR_DATA.span)
    

    console.log("RPDA:", RPDA[0].toBase58())
    console.log("FPDA: ", FPDA.toBase58())
    console.log("fundStateAccountRead:: ", fundStateAccount)
    console.log("associatedTokenaccount:: ", associatedTokenAddress1.toBase58())

    const dataLayout = struct([u8('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1,
        amount: amount * 1000000000
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
        { pubkey: FPDA, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
      ],
      programId,
      data
    });

    // let trans = await setWalletTransaction(instruction, key);


    const investorDataAcc = await connection.getAccountInfo(investerStateAccount);
    const investorData = INVESTOR_DATA.decode(investorDataAcc.data);
    if(!investorData.is_initialized) {
    
      const transaction2 = await setWalletTransaction(instruction, walletProvider?.publicKey);
      const signature = await signAndSendTransaction(walletProvider, transaction2);
      let result = await connection.confirmTransaction(signature, "confirmed");

      console.log("Tx confirmed:: ", signature);
    }
  }

  const handleFunds = async () => {
    let managers = []
    const platformDataAcc = await connection.getAccountInfo(platformStateAccount)
    const platformData = PLATFORM_DATA.decode(platformDataAcc.data)
    console.log("platformData :: ", platformData)
  
    for(let i=0; i<platformData.no_of_active_funds; i++) {
      let manager = platformData.fund_managers[i];
      let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
      let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);
      
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


