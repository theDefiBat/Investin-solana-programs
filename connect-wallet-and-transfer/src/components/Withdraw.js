import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection,  FUND_ACCOUNT_KEY, programId, TOKEN_PROGRAM_ID} from '../utils/constants';

import { struct, u32 } from 'buffer-layout';
import { createKeyIfNotExists, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { INVESTOR_DATA, FUND_DATA } from '../utils/programLayouts';
import { IDS, MangoClient, NodeBankLayout, PerpMarketLayout } from '@blockworks-foundation/mango-client';

export const Withdraw = () => {

  const [fundPDA, setFundPDA] = useState('')
  const [fundStateAccount, setFundStateAccount] = useState('')
  const [funds, setFunds] = useState([]);


  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const fundAccount = GlobalState.useState(s => s.createFundPublicKey);
  const ids = IDS['groups'][0]


  const handleWithdraw = async () => {

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
    
    const transaction = new Transaction()

    const investerStateAccount = await createKeyIfNotExists(walletProvider, null, programId, fundPDA.substr(0, 31), INVESTOR_DATA.span)
    const investorBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), key, transaction);

    let fundStateInfo = await connection.getAccountInfo(new PublicKey(fundStateAccount))
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)

    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    console.log("mango group:: ", mangoGroup)

    let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[0].nodeKeys[0]))
    let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
    console.log("nodebank:: ", nodeBank)


    const dataLayout = struct([u32('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 2,
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: fundStateAccount, isSigner: false, isWritable: true },
        { pubkey: investerStateAccount, isSigner: false, isWritable: true }, //fund State Account
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: fundState.vault_key, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },

        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
        { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundPDA), isSigner: false, isWritable: false },
        { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.perpMarkets[0].publicKey), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.perpMarkets[0].bidsKey), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.perpMarkets[0].asksKey), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.perpMarkets[0].eventsKey), isSigner: false, isWritable: true },

        { pubkey: new PublicKey(ids.tokens[0].rootKey), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.tokens[0].nodeKeys[0]), isSigner: false, isWritable: true },
        { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
        { pubkey: investorBaseTokenAccount, isSigner: false, isWritable: true }, // Investor Token Accounts
        { pubkey: mangoGroup.signerKey, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
        { pubkey: PublicKey.default, isSigner: false, isWritable: true },
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
  
    let funds = await connection.getProgramAccounts(programId, { filters: [{ dataSize: FUND_DATA.span }] });
    console.log(`funds :::: `, funds)
    const fundData = funds.map(f => FUND_DATA.decode(f.account.data))

    console.log(`decodedFunds ::: `, fundData)
    let invFunds = []
    for(let i=0; i<fundData.length; i++) {
      let manager = fundData[i].manager_account;

      let PDA = await PublicKey.findProgramAddress([manager.toBuffer()], programId);
      let fundState = await PublicKey.createWithSeed(manager, FUND_ACCOUNT_KEY, programId);

      let invStateAccount = await PublicKey.createWithSeed(walletProvider?.publicKey, PDA[0].toBase58().substr(0, 31), programId);
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
  }

  const handleHarvestMngo = async () => {
    
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

    if (fundStateAccount == ''){
      alert("get fund info first!")
      return
    }
    let fundStateInfo = await connection.getAccountInfo((new PublicKey(fundStateAccount)))
    let fundState = FUND_DATA.decode(fundStateInfo.data)
    console.log("fundState:: ", fundState)
  
    let client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(new PublicKey(ids.publicKey))
    console.log("mango group:: ", mangoGroup)
  
    let perpAcc = await connection.getAccountInfo(new PublicKey(ids.perpMarkets[0].publicKey))
    let perpMkt = PerpMarketLayout.decode(perpAcc.data)
    console.log("perpmkt:: ", perpMkt)
  
    let nodeBankInfo = await connection.getAccountInfo(new PublicKey(ids.tokens[1].nodeKeys[0]))
    let nodeBank = NodeBankLayout.decode(nodeBankInfo.data)
    console.log("nodebank:: ", nodeBank)

    const investorStateAccount = await PublicKey.createWithSeed(
      key,
      fundPDA.substr(0, 31),
      programId,
    );
  
    const transaction = new Transaction()
  
    const investorMngoAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[1].mintKey), key, transaction);
  
    const dataLayout = struct([u32('instruction')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 3
      },
      data
    )
  
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true },
        { pubkey: investorStateAccount, isSigner: false, isWritable: true },
        { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
        { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: false },
        { pubkey: fundState.mngo_vault_key, isSigner: false, isWritable: true },
        { pubkey: investorMngoAccount , isSigner: false, isWritable: true },
  
        { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
        { pubkey: mangoGroup.mangoCache , isSigner: false, isWritable: false },
        { pubkey: fundState.mango_account, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundPDA), isSigner: false, isWritable: false },
        { pubkey: new PublicKey(ids.perpMarkets[0].publicKey), isSigner: false, isWritable: false },
        { pubkey: perpMkt.mngoVault, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.tokens[1].rootKey), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(ids.tokens[1].nodeKeys[0]), isSigner: false, isWritable: true },
        { pubkey: nodeBank.vault, isSigner: false, isWritable: true },
        { pubkey: mangoGroup.signerKey, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },
        { pubkey: PublicKey.default, isSigner: false, isWritable: true },
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
      <button onClick={handleGetInvestments}>Get Investments</button>

      <br />
      <br />
     
      <button onClick={handleWithdraw}>Withdraw from Fund</button>
      <button onClick={handleHarvestMngo}>Harvest Mngo</button>
  
      <br />
    </div>
  )

}