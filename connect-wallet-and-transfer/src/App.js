import { Container } from 'reactstrap';
import { CustomNavbar } from './components/CustomNavbar';
import './App.css';
import { useEffect, useState } from 'react';

import { GlobalState } from './store/globalState'
import { getTransactions } from './utils/getTransactions';
import { connection } from './utils/constants';
import TransactionsView from './components/Transactions';
import { SendGreating } from './components/SendGreating';
import { Deposit } from './components/Deposit';
import { InitialisedFund } from './components/InitialisedFund';
import { Swap } from './components/Swap';
import { Withdraw } from './components/Withdraw';
import { Transfer } from './components/Transfer';
import { Claim } from './components/ClaimFee';
import { Testing } from './components/Testing';
import { GetPrices } from './components/GetPrices';
import { MangoInitialize } from './components/MangoInitialize';
import { MangoDeposit } from './components/MangoDeposit';
import { MangoWithdrawToFund } from './components/MangoWithdrawToFund';


function App() {
  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const address = GlobalState.useState(s => s.address);
  const [transactions, setTransactions] = useState([]);

  // useEffect(() => {
  //   if (walletProvider?.publicKey) {
  //     console.log(`walletProvider?.publicKey ::: `, walletProvider?.publicKey)
  //     getTransactions(connection, walletProvider.publicKey).then((trans) => {
  //       console.log(`trans ::: `, trans)
  //       setTransactions(trans);
  //     });
  //   }
  // }, [walletProvider])

  return (
    <div>
      <Container>
        <CustomNavbar />
        {
          address &&
          <p>Connected to {address}</p>
        }
        {/* {
          transactions && <TransactionsView transactions={transactions} />
        } */}
        <SendGreating />
        <InitialisedFund />
        <Deposit />
	      <Transfer />
        <Swap />
        <Testing />
        <Withdraw />
        <Claim />
        <GetPrices />
        <MangoInitialize />
        <MangoDeposit />
        <MangoWithdrawToFund />
      </Container>
    </div>
  );
}

export default App;
