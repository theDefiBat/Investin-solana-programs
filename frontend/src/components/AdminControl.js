import React, { useEffect, useState } from 'react'
import { createAssociatedTokenAccount, createAssociatedTokenAccountIfNotExist, createKeyIfNotExists, createTokenAccountIfNotExist, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction } from '../utils/web3'
import { connection, FUND_ACCOUNT_KEY, platformStateAccount, PLATFORM_ACCOUNT_KEY, programId } from '../utils/constants'
import { GlobalState } from '../store/globalState';
import { nu64, struct, u8 } from 'buffer-layout';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@project-serum/serum/lib/token-instructions';
import { FUND_DATA, PLATFORM_DATA, U64F64 } from '../utils/programLayouts';
import { Badge } from 'reactstrap';
import { MANGO_TOKENS } from "../utils/tokens";
import BN from 'bn.js';

export const AdminControl = () => {

  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleAdminControl = async () => {
    console.log("handle initalise fund clicked")

    const transaction = new Transaction()

    // ***what should be in the place of wallet provider in platformAccount
    const platformAccount = await createKeyIfNotExists(walletProvider, "", programId, PLATFORM_ACCOUNT_KEY, PLATFORM_DATA.span, transaction)

    console.log(`PLATFORM_DATA.span :::: `, PLATFORM_DATA.span)


    if (1) {
      const dataLayout = struct([
        u8('instruction'),
        u8('intialize_platform'),
        u8('freeze_platform'),
        u8('unfreeze_platform'),
        u8('change_vault'),
        u8('freeze_fund'),
        u8('unfreeze_fund'),
        nu64('change_min_amount'),
        nu64('change_min_return'),
        nu64('change_perf_fee')
      ])

      const data = Buffer.alloc(dataLayout.span)
      dataLayout.encode(
        {
          instruction: 7,
          intialize_platform: v0,
          freeze_platform: v1,
          unfreeze_platform: v2,
          change_vault: v3,
          freeze_fund: v4,
          unfreeze_fund: v5,
          min_amount: min_amount * (10 ** MANGO_TOKENS['USDC'].decimals),
          min_return: new BN(min_return),
          performance_fee_percentage: new BN(platform_fee_percentage),
        },
        data
      )
      const associatedTokenAddress1 = await createAssociatedTokenAccountIfNotExist(walletProvider, new PublicKey(MANGO_TOKENS['USDC'].mintAddress), walletProvider?.publicKey, transaction);

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: platformAccount, isSigner: false, isWritable: true },
          { pubkey: walletProvider?.publicKey, isSigner: true, isWritable: true },
          { pubkey: associatedTokenAddress1, isSigner: false, isWritable: true },
          { pubkey: new PublicKey(MANGO_TOKENS.USDC.mintAddress), isSigner: false, isWritable: true },

          // { pubkey: fundAccount, isSigner: false, isWritable: true },
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

  }

  const [v0, setv0] = useState(0);
  const [v1, setv1] = useState(0);
  const [v2, setv2] = useState(0);
  const [v3, setv3] = useState(0);
  const [v4, setv4] = useState(0);
  const [v5, setv5] = useState(0);

  const [min_amount, setMin_amount] = useState(0);
  const [min_return, setMin_return] = useState(0);
  const [platform_fee_percentage, setPlatform_fee_percentage] = useState(0);

  return (
    <div className="form-div">
      <h4>Admin Controls</h4>
      init_platform ::: {' '}
      <input type="number" value={v0} onChange={(event) => setv0(event.target.value)} />
      <br />
      freeze_platform ::: {' '}
      <input type="number" value={v1} onChange={(event) => setv1(event.target.value)} />
      <br />
      unfreeze_platform ::: {' '}
      <input type="number" value={v2} onChange={(event) => setv2(event.target.value)} />
      <br />
      change_vault ::: {' '}
      <input type="number" value={v3} onChange={(event) => setv3(event.target.value)} />
      <br />
      freeze_fund ::: {' '}
      <input type="number" value={v4} onChange={(event) => setv4(event.target.value)} />
      <br />
      unfreeze_fund ::: {' '}
      <input type="number" value={v5} onChange={(event) => setv5(event.target.value)} />
      <br />

      min_amount ::: {' '}
      <input type="number" value={min_amount} onChange={(event) => setMin_amount(event.target.value)} />
      <br />
      min_return ::: {' '}
      <input type="number" value={min_return} onChange={(event) => setMin_return(event.target.value)} />
      <br />
      platform_fee_percentage ::: {' '}
      <input type="number" value={platform_fee_percentage} onChange={(event) => setPlatform_fee_percentage(event.target.value)} />
      <br />
      <button onClick={handleAdminControl}>Admin Control</button>
    </div>
  )
}

