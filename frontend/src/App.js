import { Container } from 'reactstrap';
import { CustomNavbar } from './components/CustomNavbar';
import './App.css';
import { useEffect, useState } from 'react';

import { GlobalState } from './store/globalState'
import { InitialisedFund } from './components/InitialisedFund';
import { Deposit } from './components/Deposit';
import { Withdraw } from './components/Withdraw';
// import { Claim } from './components/ClaimFee';
import { DisplayInfo } from './components/DisplayInfo';
import { AllFundsInvestors } from './components/AllFundsInvestors';


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
         <DisplayInfo/>
        <InitialisedFund />
        <Deposit />
        <Withdraw />
        {/* <Claim /> */}
        <AllFundsInvestors/>
      </Container>
    </div>
  );
}

export default App;
