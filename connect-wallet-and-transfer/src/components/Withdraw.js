import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GlobalState } from '../store/globalState';
import { connection, programId } from '../utils/constants';
import { nu64, struct, u8 } from 'buffer-layout';
import { createKeyIfNotExists, findAssociatedTokenAddress, setWalletTransaction, signAndSendTransaction, createAssociatedTokenAccount } from '../utils/web3';
import { pools } from '../utils/pools';

const getPoolAccounts = () => {
  return pools.map((p) => {
    return [
      { pubkey: new PublicKey(p.poolCoinTokenAccount), isSigner: false, isWritable: true },
      { pubkey: new PublicKey(p.poolPcTokenAccount), isSigner: false, isWritable: true }
    ]
  })
}

export const Withdraw = () => {

  const [amount, setAmount] = useState(0);
  const [fundPDA, setFundPDA] = useState('');

  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const fundAccount = GlobalState.useState(s => s.createFundPublicKey);

  const handleWithdraw = async () => {

    console.log(`[...getPoolAccounts()] ::: `, [...getPoolAccounts()])

    const key = walletProvider?.publicKey;

    if (!key) {
      alert("connect wallet")
      return;
    };

    const clientAccount = await connection.getAccountInfo(key);
    const investerStateAccount = await createKeyIfNotExists(walletProvider, clientAccount, programId, (64 + 16 + 2))
    const transaction = new Transaction()

    const routerAssociatedTokenAddress = await createAssociatedTokenAccount(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), PDA[0], transaction);
    // TODO: Manager Base Token Account
    const managerAssociatedTokenAccount = await createAssociatedTokenAccount(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), PDA[0], transaction);
    // TODO: Investin Base Token Account
    const investinAssociatedTokenAddress = await createAssociatedTokenAccount(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), PDA[0], transaction);

    const PDA = await PublicKey.findProgramAddress([key.toBuffer()], programId);
    const MPDA = new PublicKey('FqV8b3zWwLLT8SgkP3jQoZntbv5WAuxpNYmVk9HtR2yQ')

    const investorBaseTokenAccount = await findAssociatedTokenAddress(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'));
    const investorTokenAccount2 = await findAssociatedTokenAddress(key, new PublicKey('HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o'));
    const investorTokenAccount3 = await findAssociatedTokenAddress(key, new PublicKey('HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty'));

    const fundAssociatedTokenAddress1 = await createAssociatedTokenAccount(key, new PublicKey('DdzREMVFg6pa5825HBKVzeCrEi8EJiREfb8UrxSZB64w'), MPDA[0], transaction);
    const fundAssociatedTokenAddress2 = await createAssociatedTokenAccount(key, new PublicKey('HUHuQCZUvxCiuFg54vRStrXSbCFeBhmXRqSuR5eEVB6o'), MPDA[0], transaction);
    const fundAssociatedTokenAddress3 = await createAssociatedTokenAccount(key, new PublicKey('HW18fiAHKzs7ZSaT5ibAhnSWVde25sazTSbMzss4Fcty'), MPDA[0], transaction);

    const dataLayout = struct([u8('instruction'), nu64('amount')])
    const data = Buffer.alloc(dataLayout.span)
    dataLayout.encode(
      {
        instruction: 1,
        amount: amount * 1000000000
      },
      data
    )

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: new PublicKey('A6R1siU7eCLiTFTswtofYeUtaRCPWn65yT4VhfY1vPEK'), isSigner: false, isWritable: true }, //fund State Account
        { pubkey: investerStateAccount, isSigner: false, isWritable: true },
        { pubkey: key, isSigner: true, isWritable: true },
        { pubkey: routerAssociatedTokenAddress, isSigner: false, isWritable: true }, // Router Base Token Account
        { pubkey: managerAssociatedTokenAccount, isSigner: false, isWritable: true }, // Manager Base Token Account
        { pubkey: investinAssociatedTokenAddress, isSigner: false, isWritable: true }, // Investin Base Token Account
        { pubkey: MPDA, isSigner: false, isWritable: false },
        { pubkey: new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), isSigner: false, isWritable: true },

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
  }

  return (
    <div className="form-div">
      <h4>Withdraw</h4>
      amount ::: {' '}
      <input type="number" value={amount} onChange={(event) => setAmount(event.target.value)} />
      <br />
      fund pda :: {' '}
      <input type="text"
        onChange={(e) => setFundPDA(e.target.value)}
        value={fundPDA}
      />
      <button onClick={handleWithdraw}>Deposit</button>
    </div>
  )

}