import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'
import { GreetingAccount } from '../programData/GreetingAccount';
import { GlobalState } from '../store/globalState';
import * as borsh from 'borsh';
import { GreetingSchema } from '../programData/GreetingSchema';
import {
  Account,
  Connection,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import { connection } from '../utils/constants';
import { blob, nu64, struct, u32, u8 } from 'buffer-layout';
const exampleText = "0b00000000000000ua";

const fillString = (someString) => {
  const charDiff = exampleText.length - someString.length;
  if (charDiff !== 0) {
    return someString + Array(charDiff).join('x');
  }
}

export const SendGreating = (props) => {

  const [inputText, setInputText] = useState("");
  const walletProvider = GlobalState.useState(s => s.walletProvider);

  const handleSubmit = async () => {

    if (walletProvider?.publicKey) {

      console.log(`GreetingAccount ::: `, GreetingAccount)

      const k = struct([u32('counter')])
      const data = Buffer.alloc(k.span)

      console.log("here 1")

      k.encode(
        {
          txt: fillString(inputText)
        },
        data
      )
      console.log("here 2")

      console.log(`GreetingSchema ::: `, GreetingSchema)

      // const x = borsh.serialize(GreetingSchema, messageAccount)
      console.log("here 3")

      const instruction = new TransactionInstruction({
        keys: [{ pubkey: walletProvider?.publicKey, isSigner: false, isWritable: true }],
        programId: new PublicKey("7Y1avRKxTRtBW4rH4dMoauYDVpfoya29on96azbmoFDx"),
        data: k,
      });
      console.log("here 4")

      const walletAccount = await connection.getAccountInfo(
        walletProvider?.publicKey
      );

      // console.log(`walletAccount ::: `, walletAccount.tokenAccounts)
      

      console.log("sending data started >>>> ")

      // await sendAndConfirmTransaction(
      //   connection,
      //   new Transaction().add(instruction),
      //   [walletAccount],
      // );

      console.log("done sending data >>>> ")
      const someKey = new PublicKey("EDmcyqMkrvWh8vQXXCF6VigTe4bTyxoAGjPYA9UCQr8u");
      // const x = connection._buildArgs([], "confirmed", "jsonParsed");
      const accountInfo = await connection.getParsedAccountInfo(someKey);
      console.log(`accountInfo ::: `, accountInfo)
      if (accountInfo === null) {
        throw 'Error: cannot find the greeted account';
      }
      console.log(`accountInfo.data:::`, accountInfo.data)
      // const greeting = k.decode(accountInfo.data)
      // console.log(`greeting ::: `, greeting)
      // borsh.deserialize(
      //   GreetingSchema,
      //   GreetingAccount,
      //   accountInfo.data,
      // );
      // console.log(
      //   someKey.toBase58(),
      //   'has been greeted',
      //   greeting.counter
      // );

    }
  }

  return (
    <div>
      <input
        type="text"
        onChange={(event) => setInputText(event.target.value)}
        value={inputText}
        maxLength={exampleText.length}
      />
      <button onClick={handleSubmit}>Submit</button>
    </div>
  )
}

