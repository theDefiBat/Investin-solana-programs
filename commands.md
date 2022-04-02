$ cargo --version
cargo 1.55.0 (32da73ab1 2021-08-23)

$ rustc --version
rustc 1.55.0 (c8dfcfe04 2021-09-06)

$solana --version
solana-cli 1.8.0 (src:4a8ff62a; feat:1813598585)

# config
$ solana config get
Config File: /Users/aw/.config/solana/cli/config.yml
RPC URL: https://api.devnet.solana.com 
WebSocket URL: wss://api.devnet.solana.com/ (computed)
Keypair Path: /Users/aw/.config/solana/id.json 
Commitment: confirmed 

# set to localhost
solana config set --url localhost

# set to devnet
solana config set --url devnet

# get address 
solana address

# get full details about a account
solana account <address from above>

# Next, start the local network. This is going to be a local Solana node that we can deploy to for testing:
solana-test-validator

# airdrop
solana airdrop 2

# build
npm run build:program-fund
# deploy 
solana program deploy /Users/aw/Desktop/Solana-Apps/Solana-IVN-Code/dist/program-fund/fund.so

# deploy to exsiting addresss
solana program deploy /Users/aw/Desktop/Solana-Apps/Solana-IVN-Code/dist/program-fund/fund.so --program-id  EYCioTzTEMFwcYKbYn8HMNdY32KkpEnRnKuirv7GZPmL

# program size 
ls -al /Users/aw/Desktop/test-solana-playground/mango-cpi/target/deploy/spl_example_transfer_lamports.so


# address

Xoheb Devnet deployer : E3Zhv46FWGLDKFM24Ft2tgoqX5NCU49CT8NwH3rDHbsA
(adminAccount) : E3Zhv46FWGLDKFM24Ft2tgoqX5NCU49CT8NwH3rDHbsA
Program Id: EYCioTzTEMFwcYKbYn8HMNdY32KkpEnRnKuirv7GZPmL
PLATFORM_STATE_ACCOUNT: 8VNYUXwcvv5zJbj6ZzeDA2kV63CkmZqc39gg9VS1fgus

OLD devnet deployer: B1J3ttZ2PLCG4KUQHLtkgYSZUmD6f8KNLjGSWzz8eE1Y
(adminAccount) : B1J3ttZ2PLCG4KUQHLtkgYSZUmD6f8KNLjGSWzz8eE1Y
program Id : 6HwgHw4QdgKR6kjHpdFdwGoP8MBy1gHGtde4JkS1GQmE
PLATFORM_STATE_ACCOUNT: 

main mainnet 
deployer : Fepyuf4vy7mKZVgpzS52UoUeSLmVvGnoMDyraCsjYUqn
(adminAccount) : owZmWQkqtY3Kqnxfua1KTHtR2S6DgBTP75JKbh15VWG
program id : 8dbbmZXbLsUirEsgaBVcPBEdciESza6L2zkEuer4crR 
PLATFORM_STATE_ACCOUNT : Cpf6kq7w4iR2hWdWTkWeoxvyRrrduke5XhA7QM5SkGNo


# tokens 
USDC  mint 8FRFC6MoGGkMFQwngccyu69VnYbzykGeez7ignHVAFSN
pools 
----BTC mint 3UNBZ6o52WTWwjac2kPUb4FyodhU1vFkRJheu1Sh2TvU
pool created: 88CJ7Zqe6RdBSHetEkruSNRpSGibexoGWKo2Jf8pPPf6
amm.poolCoinTokenAccount : Fq8nURZFto3bTPaKqmDqbBAFfArwYvfVuFKVoFyZZVD6
amm.poolPcTokenAccount : BvuamxaDy5VKnoCshtFmbFNviWbu2VzW2Gecuf12meUA

----MANGO mint  Bb9bsTQa1bGEtQ5KagGkvSHyuLqDWumFUcRqFusFNJWC
pool created: 7C9ehd2nj6PZcjix4DLhsKQYVDjYRmjLrn3nFgpQTSqZ
amm.poolCoinTokenAccount : 3Rg73PnXckaMr715Jvc3dWEt2tEBX1cjQ88SYmpqqnpw
amm.poolPcTokenAccount : H2DfYoN52NTs1hMkyZiK8XGJKUqyRgCWhxwC5MC48KPk

----ETH mint Cu84KB3tDL6SbFgToHMLYVDJJXdJjenNzSKikeAvzmkA
pool created: 7DpTVfnPK8X4oQt215doZ23evnxUXRXBDX1g5sHBXyDr
amm.poolCoinTokenAccount : 2RF2cAAACsvft5g6SZU6yPidBQ8RptExAt3DXAhnyUvq
amm.poolPcTokenAccount : 7D35qDSvfgcmVaMgZt4HH8R1ruDog5d2ZTS79Jgpa7yQ

----WSOL mint So11111111111111111111111111111111111111112
pool created: 384zMi9MbUKVUfkUdrnuMfWBwJR9gadSxYimuXeJ9DaJ
amm.poolCoinTokenAccount : v77vV7yh5LuEKabKdNWhM8X5wpDhuPidjp7eucbHGfy
amm.poolPcTokenAccount : CYv6PtaGV2D2gj6pD9io1PjJbUrcE7gyBmDethawZmMm



# Error processing Instruction 1: custom program error: 0x1
If you receive the error 'failed to send transaction: Transaction simulation failed: Error processing Instruction 1: custom program error: 0x1' that means you do not have enough Solana (SOL) in your wallet to cover the transaction fee.
