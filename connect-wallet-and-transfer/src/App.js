import { Container } from 'reactstrap';
import { CustomNavbar } from './components/CustomNavbar';
import './App.css';
import { useEffect, useState } from 'react';

import { GlobalState } from './store/globalState'
import { getTransactions } from './utils/getTransactions';
import { connection } from './utils/constants';
// import TransactionsView from './components/Transactions';
// import { SendGreating } from './components/SendGreating';
import { Deposit } from './components/Deposit';
// import { InitialisedFund } from './components/InitialisedFund';
// import { Swap } from './components/Swap';
// import { Withdraw } from './components/Withdraw';
// import { Transfer } from './components/Transfer';
// // import { Claim } from './components/ClaimFee';
import { GetPrices } from './components/GetPrices';
// import { MangoInitialize } from './components/MangoInitialize';
// import { MangoPlaceOrder } from './components/MangoPlaceOrder';
// import { AdminControl } from './components/AdminControl';
import { IVN } from './components/IVN';


function App() {
  const walletProvider = GlobalState.useState(s => s.walletProvider);
  const address = GlobalState.useState(s => s.address);
  const [transactions, setTransactions] = useState([]);

  return (
    <div>
      <Container>
        <CustomNavbar />
        {/* {
          address &&
          <p>Connected to {address}</p>
        } */}
        {/* {
          transactions && <TransactionsView transactions={transactions} />
        } */}
        {/* <SendGreating />
        
        
        <Claim />
        */}
        {/* <InitialisedFund />
        <Transfer />
        <Withdraw /> 
        <Swap />
        <MangoInitialize />
        <MangoPlaceOrder /> */}
        <GetPrices />
        {/* <AdminControl /> */}
        <Deposit />
        <IVN />

      </Container>
    </div>
  );
}

export default App;
