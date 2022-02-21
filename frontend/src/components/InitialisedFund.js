import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, idsIndex, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8 } from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, PLATFORM_DATA, u64, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import BN from 'bn.js';
import { TOKENS } from '../utils/tokens';
import { IDS } from '@blockworks-foundation/mango-client';

export const InitialisedFund = () => {

  const ids= IDS['groups'][idsIndex];

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleInitialFund = async () => {

    const transaction = new Transaction()


    // ***what should be in the place of wallet provider in platformAccount
    const platformAccount = platformStateAccount;
    // const platformAccount = await createKeyIfNotExists(walletProvider, "", programId, PLATFORM_ACCOUNT_KEY, PLATFORM_DATA.span, transaction)

    console.log(`PLATFORM_DATA.span :::: `, PLATFORM_DATA.span)
    console.log(`**FUND_DATA.span :::: `, FUND_DATA.span) 

    const fundPDA = await PublicKey.findProgramAddress([walletProvider?.publicKey.toBuffer()], programId);
    // const routerPDA = await PublicKey.findProgramAddress([Buffer.from('router')], programId);
    console.log(`fundPDA :::: `, fundPDA, fundPDA[0].toBase58()) 


    // const fundAccount = await createKeyIfNotExists(
    //   walletProvider,
    //   "",
    //   programId,
    //   FUND_ACCOUNT_KEY,
    //   FUND_DATA.span,
    //   transaction,
    // );

    if (1) {
      const dataLayout = struct([u8('instruction'), nu64('min_amount'),
       nu64('min_return'), nu64('performance_fee_percentage'), u8('no_of_tokens')])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 0,
          min_amount: min_amount * (10 ** ids.tokens[0].decimals),
          min_return: min_return * 100,
          performance_fee_percentage: platform_fee_percentage * 100,
          no_of_tokens: 1
        },
        data
      )

      console.log("params data passed :",
           min_amount * (10 ** ids.tokens[0].decimals),
          min_return * 100,
           platform_fee_percentage * 100,
      )

      const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(
        walletProvider,
        new PublicKey(ids.tokens[0].mintKey),
        fundPDA[0],
        transaction,
      );
      // const associatedTokenAddress2 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(ids.tokens[1].mintKey), fundPDA[0], transaction);

      console.log("associatedTokenAddress1:",associatedTokenAddress1.toBase58())
      console.log("ids.tokens[0].mintKey:",ids.tokens[0].mintKey)

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformAccount, isSigner: false, isWritable: true },
          { pubkey: fundPDA[0], isSigner: false, isWritable: true },
          { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },

          { pubkey: new PublicKey(ids.tokens[0].mintKey), isSigner: false, isWritable: true },
          { pubkey: associatedTokenAddress1, isSigner: false, isWritable: true },

          // { pubkey: new PublicKey(ids.tokens[1].mintKey), isSigner: false, isWritable: true }, 
          // { pubkey: associatedTokenAddress2, isSigner: false, isWritable: true },

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
      console.log("signature tx url:: ", `https://solscan.io/tx/${sign}`) 

    }

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

