
import React, { useState } from 'react';
import { Button, Modal, ModalHeader, ModalBody, ModalFooter } from 'reactstrap';
import PropTypes from 'prop-types';
import { GlobalState } from '../store/globalState'
import SolanaWalletAdapter from '@project-serum/sol-wallet-adapter'
import { cluster } from '../utils/constants';
import { PhantomWalletAdapter } from '../utils/wallets/PhantomWallet';
import { PublicKey } from '@solana/web3.js'

const connectWalletButtonLabel = 'Connect Wallet'

export const ConnectWalletModal = (props) => {

  const [modal, setModal] = useState(false);
  const [buttonLabel, setButtonLabel] = useState(connectWalletButtonLabel);

  const toggle = () => setModal(!modal);

  const providers = ['Sollet Extension', 'Sollet', 'Phantom'];

  const handleSelectProvider = async (provider) => {
    let walletProvider = {};
    switch (provider) {
      case providers[0]: {
        walletProvider = new SolanaWalletAdapter(window.sollet, cluster)
        break;
      }
      case providers[1]: {
        walletProvider = new SolanaWalletAdapter("https://www.sollet.io", cluster);
        break;
      }
      case providers[2]: {
        walletProvider = new PhantomWalletAdapter();
        break;
      }
      default:
        break;
    }
    await walletProvider.connect();
    console.log(`walletProvider ::: `, walletProvider)
    walletProvider.on('connect', publicKey => {
      console.log('Connected to ' + publicKey.toBase58())
      console.log(`walletProvider :: `, walletProvider)
    
      let walletProviderTest = walletProvider;
      // walletProvider = {...walletProvider, publicKey : new PublicKey('5Jgocz6kyqU3fjoC6tBp55UiYoxNeDwtme9jKr4SKUUD')}
      
      GlobalState.update(s => {
        s.walletProvider = walletProviderTest;
        s.address = publicKey.toBase58();
      })
      setButtonLabel(provider);
      toggle()
    });
  }

  return (
    <div>
      <Button color="dark"
        onClick={toggle}>
        {buttonLabel == connectWalletButtonLabel ? connectWalletButtonLabel : `Connected to ${buttonLabel}`}
      </Button>
      <Modal isOpen={modal} toggle={toggle}>
        <ModalHeader toggle={toggle}>Choose Provider</ModalHeader>
        <ModalBody>
          {
            providers.map((x) => {
              return (
                <div key={x} className="provider" onClick={() => handleSelectProvider(x)}>{x}</div>
              )
            })
          }
        </ModalBody>
        {/* <ModalFooter>
          <Button color="primary" onClick={toggle}>Do Something</Button>{' '}
          <Button color="secondary" onClick={toggle}>Cancel</Button>
        </ModalFooter> */}
      </Modal>
    </div>
  );
}

ConnectWalletModal.propTypes = {
  buttonLabel: PropTypes.string,
  className: PropTypes.string
};
