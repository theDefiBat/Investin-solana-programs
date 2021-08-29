import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, MARGIN_ACCOUNT_KEY_1, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8, u32} from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, PLATFORM_DATA, u64, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import { IDS, MangoAccountLayout } from '@blockworks-foundation/mango-client'
import BN from 'bn.js';

export const InitialisedFund = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleInitialFund = async () => {

    const transaction = new Transaction()
    let ids = IDS['groups'][0]


    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);

    const fundAccount = await createKeyIfNotExists(walletProvider, "", programId, FUND_ACCOUNT_KEY, FUND_DATA.span, transaction)
    const mangoAccount = await createKeyIfNotExists(walletProvider, "", new PublicKey(ids.mangoProgramId), MARGIN_ACCOUNT_KEY_1, MangoAccountLayout.span, transaction)

    console.log(`FUND_DATA.span :::: `, FUND_DATA.span) 


    console.log(`fundPDA::: `, fundPDA[0].toBase58())
    console.log(`fundAccount ::: `, fundAccount.toBase58())

    if (1) {
      const dataLayout = struct([u32('instruction'), nu64('min_amount'), nu64('min_return'), nu64('performance_fee_percentage'), u8('perp_market_index')])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 0,
          min_amount: min_amount * (10 ** ids.tokens[0].decimals),
          min_return: min_return * 100,
          performance_fee_percentage: platform_fee_percentage * 100,
          perp_market_index: 1
        },
        data
      )


      const fundBaseVault = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[0].mintKey), fundPDA[0], transaction);
      const fundMngoVault = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[1].mintKey), fundPDA[0], transaction);

      console.log("IDS:: ", ids)
      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: fundAccount, isSigner: false, isWritable: true },
          { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
          { pubkey: fundPDA[0], isSigner: false, isWritable: true },

          { pubkey: fundBaseVault, isSigner: false, isWritable: true },
          { pubkey: fundMngoVault, isSigner: false, isWritable: true },
    
          { pubkey: new PublicKey(ids.publicKey), isSigner: false, isWritable: true },
          { pubkey: mangoAccount, isSigner: false, isWritable: true },
          { pubkey: new PublicKey(ids.mangoProgramId), isSigner: false, isWritable: true },

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

    }

    GlobalState.update(s => {
      s.createFundPublicKey = fundAccount;
    })
  }

  const [min_amount, setMin_amount] = useState(0);
  const [min_return, setMin_return] = useState(0);
  const [platform_fee_percentage, setPlatform_fee_percentage] = useState(0);

  return (
    <div className="form-div">
      <h4>Initialise Fund</h4>
      min_amount ::: {' '}
      <input type="number" value={min_amount} onChange={(event) => setMin_amount(event.target.value)} />
      <br />
      min_return ::: {' '}
      <input type="number" value={min_return} onChange={(event) => setMin_return(event.target.value)} />
      <br />
      platform_fee_percentage ::: {' '}
      <input type="number" value={platform_fee_percentage} onChange={(event) => setPlatform_fee_percentage(event.target.value)} />
      <br />
      <button onClick={handleInitialFund}>initialise fund</button>
    </div>
  )
}

