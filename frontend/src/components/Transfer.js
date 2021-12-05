import { PublicKey, SYSVAR_CLOCK_PUBKEY, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { adminAccount, connection, FUND_ACCOUNT_KEY, idsIndex, MANGO_GROUP_ACCOUNT, MANGO_PROGRAM_ID, platformStateAccount, priceStateAccount, programId, TOKEN_PROGRAM_ID } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccountIfNotExist } from '../utils/web3';
import { FUND_DATA, INVESTOR_DATA, PLATFORM_DATA, PRICE_DATA } from '../utils/programLayouts';
import { devnet_pools, pools } from '../utils/pools'

import { updatePoolPrices } from './updatePrices';
import {
  IDS,
  MangoClient, MangoGroupLayout, MarginAccountLayout
} from '@blockworks-foundation/mango-client'

export const Transfer = () => {

  const ids= IDS['groups'][idsIndex];

  const [fundPDA, setFundPDA] = useState('')
  const [fundStateAccount, setFundStateAccount] = useState('')
  const [amountInRouter, setAmountInRouter] = useState(0);
  const [fundPerf, setFundPerf] = useState(0);
  const [fundAUM, setFundAUM] = useState(0);
  const [fundBalances, setFundBalances] = useState([])
  const [fundInvestorAccs, setFundInvestorAccs] = useState([])


  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleTransfer = async () => {

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };
    const transaction = new Transaction()

    const routerPDA = await PublicKey.findProgramAddress([Buffer.from("router")], programId);
    const fundBaseTokenAccount = await findAssociatedTokenAddress(new PublicKey(fundPDA), new PublicKey(ids.tokens[0].mintKey));
    const routerBaseTokenAccount = await findAssociatedTokenAddress(routerPDA[0], new PublicKey(ids.tokens[0].mintKey));

    const managerBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), key, transaction);
    const investinBaseTokenAccount = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), adminAccount, transaction);

    if (fundStateAccount == '') {
      alert("get info first!")
      return
    }

    const accountInfo = await connection.getAccountInfo(new PublicKey(fundStateAccount));
    const fund_data = FUND_DATA.decode(accountInfo.data);

    let fund_mango_account = fund_data.mango_positions.margin_account;

    let platData = await connection.getAccountInfo(platformStateAccount)
    let plat_info = PLATFORM_DATA.decode(platData.data)
    console.log("plat info:: ", plat_info)

    const client = new MangoClient(connection, new PublicKey(ids.mangoProgramId))
    let mangoGroup = await client.getMangoGroup(connection, ids.publicKey)
    let mangoCache = await mangoGroup.loadCache(connection)
    console.log("mangoCache:",mangoCache)

    updatePoolPrices(transaction, devnet_pools)


    const dataLayout = struct([u8('instruction')])

    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 2,
      },
      data
    )
    const transfer_instruction = new TransactionInstruction({
      keys: [
        { pubkey: platformStateAccount, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundStateAccount), isSigner: false, isWritable: true },

        { pubkey: fund_mango_account, isSigner: false, isWritable: true },
        { pubkey: MANGO_GROUP_ACCOUNT, isSigner: false, isWritable: true },
        { pubkey: new PublicKey(mangoCache), isSigner: false, isWritable: true },
        { pubkey: MANGO_PROGRAM_ID, isSigner: false, isWritable: true },

        { pubkey: key, isSigner: true, isWritable: true },

        { pubkey: routerBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: fundBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: managerBaseTokenAccount, isSigner: false, isWritable: true },
        { pubkey: plat_info.investin_vault, isSigner: false, isWritable: true },

        { pubkey: routerPDA[0], isSigner: false, isWritable: true },

        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: true },


        //investor state accounts
        { pubkey: new PublicKey(fundInvestorAccs[0]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[1]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[2]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[3]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[4]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[5]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[6]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[7]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[8]), isSigner: false, isWritable: true },
        { pubkey: new PublicKey(fundInvestorAccs[9]), isSigner: false, isWritable: true },

      ],
      programId,
      data
    });

    transaction.add(transfer_instruction);
    transaction.feePayer = key;
    let hash = await connection.getRecentBlockhash();
    console.log("blockhash", hash);
    transaction.recentBlockhash = hash.blockhash;

    const sign = await signAndSendTransaction(walletProvider, transaction);
    console.log("signature tx:: ", sign)
    console.log("signature tx url:: ", `https://solscan.io/tx/{sign}`) 

  }

  const handleGetFunds = async () => {

    console.log("size of plat data:: ", PLATFORM_DATA.span)
    console.log("size of fund dta : ", FUND_DATA.span)
    console.log('size of inv data:: ', INVESTOR_DATA.span)

    console.log('size of price acc:: ', PRICE_DATA.span)
    const key = walletProvider?.publicKey;
    if (!key) {
      alert("connect wallet")
      return;
    }
    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    setFundPDA(fundPDA[0].toBase58())

    const fundStateAccount = await PublicKey.createWithSeed(
      key,
      FUND_ACCOUNT_KEY,
      programId,
    );

    console.log("FUND STTE:: ", fundStateAccount.toBase58())
    setFundStateAccount(fundStateAccount.toBase58())

    let x = await connection.getAccountInfo(fundStateAccount)
    if (x == null) {
      alert("fund account not found")
      return
    }
    console.log(x)
    let fundState = FUND_DATA.decode(x.data)
    if (!fundState.is_initialized) {
      alert("fund not initialized!")
      return
    }
    console.log(fundState)

    setAmountInRouter(parseInt(fundState.amount_in_router) / (10 ** 9));
    setFundPerf(fundState.prev_performance)
    setFundAUM(parseInt(fundState.total_amount) / (10 ** 9))

    let bal = []
    bal.push((parseInt(fundState.tokens[0].balance) / (10 ** 9)))
    bal.push((parseInt(fundState.tokens[1].balance) / (10 ** 6)))
    bal.push((parseInt(fundState.tokens[2].balance) / (10 ** fundState.tokens[2].decimals)))
    setFundBalances(bal)
    console.log(bal)

    let investors = []
    for (let i = 0; i < 10; i++) {
      let acc = await PublicKey.createWithSeed(
        new PublicKey(fundState.investors[i].toString()),
        fundPDA[0].toBase58().substr(0, 31),
        programId
      );
      console.log(fundState.investors[i].toBase58())
      investors.push(fundState.investors[i].toBase58())
    }
    setFundInvestorAccs(investors);
  }
  return (
    <div className="form-div">
      <h4>Transfer</h4>

      <button onClick={handleTransfer}>Transfer</button>
      <button onClick={handleGetFunds}>GetFundInfo</button>
      <br />
      Info for FUND: {fundPDA}
      <br />
      amount in router:: {amountInRouter}
      <br />
      Total AUM:: {fundAUM}
      <br />
      fund performance:: {fundPerf}
      <br />
      USDR balance: {fundBalances[0]}
      <br />
      RAYT balance: {fundBalances[1]}
      <br />
      ALPHA balance: {fundBalances[2]}

    </div>
  )
}
