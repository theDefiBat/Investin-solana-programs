import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import React, { useState } from 'react'

import { adminAccount, connection, platformStateAccount, priceStateAccount, programId } from '../utils/constants';
import { blob, nu64, struct, u32, u8 } from 'buffer-layout';



export const DisplayInfo = (props) => {

//   const [inputText, setInputText] = useState("");
//   const walletProvider = GlobalState.useState(s => s.walletProvider);
  
const programIdX = programId.toBase58();
const adminAccountX = adminAccount.toBase58();
const platformStateAccountX = platformStateAccount.toBase58();
const priceStateAccountX = priceStateAccount.toBase58();

  

  return (
    <div className="form-div">
    <h4>Accounts</h4>
      <p> programID : {programIdX}</p>
      <p> adminAccount : {adminAccountX}</p>
      <p> platformStateAccount : {platformStateAccountX}</p>
      <p> priceStateAccount : {priceStateAccountX}</p>
  </div>
  )
}

