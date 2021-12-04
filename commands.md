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

# deploy 
solana program deploy /Users/aw/Desktop/Solana-Apps/Solana-IVN-Code/dist/program-fund/fund.so

# deploy to exsiting addresss
solana program deploy /Users/aw/Desktop/Solana-Apps/Solana-IVN-Code/dist/program-fund/fund.so --program-id  EYCioTzTEMFwcYKbYn8HMNdY32KkpEnRnKuirv7GZPmL


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
BTC mint 
pool created: 88CJ7Zqe6RdBSHetEkruSNRpSGibexoGWKo2Jf8pPPf6
amm.poolCoinTokenAccount : Fq8nURZFto3bTPaKqmDqbBAFfArwYvfVuFKVoFyZZVD6
amm.poolPcTokenAccount : BvuamxaDy5VKnoCshtFmbFNviWbu2VzW2Gecuf12meUA
