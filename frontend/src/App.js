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
import { DisplayInfo } from './components/DisplayInfo';

import { Swap } from './components/Swap';
import { OrcaSwap } from './components/OrcaSwap';

import { Withdraw } from './components/Withdraw';
import { Transfer } from './components/Transfer';
import { Claim } from './components/ClaimFee';
import { Testing } from './components/Testing';
import { GetPrices } from './components/GetPrices';
import { MangoInitialize } from './components/MangoInitialize';
import { MangoPlaceOrder } from './components/MangoPlaceOrder';
import { AdminControl } from './components/AdminControl';
import { AllFundsInvestors } from './components/AllFundsInvestors';
import { MigrateState } from './components/MigrateFundState';
import { UpdateAllTokenPrices } from './components/UpdateAllFundTokenPrices';





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
        <DisplayInfo/>
        <MigrateState/>
        <InitialisedFund />
        <Deposit />
	      <Transfer />
        <GetPrices />
        <Swap />
        <UpdateAllTokenPrices/>

        {/* <OrcaSwap /> */}
        {/* <Testing /> */}

        <Withdraw />
        <Claim />
        <MangoInitialize />

        <MangoPlaceOrder />
        {/* <AdminControl /> */}
        <AllFundsInvestors/>
      </Container>
    </div>
  );
}

export default App;
