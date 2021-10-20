### Investin programs

![version](https://img.shields.io/badge/version-1.1.0-blue.svg) ![license](https://img.shields.io/badge/license-MIT-blue.svg) [![GitHub issues open](https://img.shields.io/github/issues/creativetimofficial/black-dashboard-react.svg?maxAge=2592000)]() [![GitHub issues closed](https://img.shields.io/github/issues-closed-raw/creativetimofficial/black-dashboard-react.svg?maxAge=2592000)]()  [![Chat](https://img.shields.io/badge/chat-on%20discord-7289da.svg)](https://discord.com/invite/Yf54h9B)

![logoWithHeading](https://assets.coingecko.com/coins/images/15588/small/ivn_logo.png?1621267247)


Decentralized investing and trading platform 
Enabling anyone to invest or trade digital assets in a trustless and secure environment using smart contracts

### Architectural Overview
![download](https://user-images.githubusercontent.com/20189814/120890571-43e16f00-c621-11eb-9150-a8b5f431f002.png)


### Build the on-chain program

Programs are available in the programs/ folder

```bash
$ npm install
$ npm run build:program-fund
```


### Deploy the on-chain program

```bash
$ solana program deploy dist/program-fund/fund.so
```
### Basic UI
The main UI is maintained in another repo
A basic UI for testing is inside frontend folder
### 1) create env file
create a new .env inside the frontend folder
copy the corresponding env file 
```
.env.production -> for profuction
.env.devnet -> for devnet
```
### 2) Install Deps 
run cmd the following inside frontend folder
```
$ yarn
```

### 3) Start servce 
run cmd the following frontend folder
```
$ yarn start
```
