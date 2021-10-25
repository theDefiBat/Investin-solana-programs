import { Connection, PublicKey } from "@solana/web3.js";

export const PLATFORM_ACCOUNT_KEY = "pacck"; //"platAccKey_11";
export const FUND_ACCOUNT_KEY = "facck"; //"fundAccKey_11";
export const MARGIN_ACCOUNT_KEY_1 = "macck1"; //"margin_account_key_11"
export const MARGIN_ACCOUNT_KEY_2 = "macck2";//"margin_account_key_22"
export const PRICE_ACCOUNT_KEY = "margin_account_key_22"

export const cluster = process.env.REACT_APP_CLUSTER_URL;
export const connection = new Connection(cluster, "confirmed");

export const adminAccount = new PublicKey(process.env.REACT_APP_ADMIN_ACCOUNT)
export const programId = new PublicKey(process.env.REACT_APP_PROGRAMID);
export const platformStateAccount = new PublicKey(process.env.REACT_APP_PLATFORM_STATE_ACCOUNT)
export const priceStateAccount = new PublicKey(process.env.REACT_APP_PRICE_STATE_ACCOUNT)

export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(process.env.REACT_APP_ASSOCIATED_TOKEN_PROGRAM_ID)

export const LIQUIDITY_POOL_PROGRAM_ID_V4 = new PublicKey(process.env.REACT_APP_LIQUIDITY_POOL_PROGRAM_ID_V4)
export const SERUM_PROGRAM_ID_V3 = new PublicKey(process.env.REACT_APP_SERUM_PROGRAM_ID_V3)
export const ORCA_SWAP_PROGRAM_ID = new PublicKey('9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP')

export const MANGO_PROGRAM_ID_V2 = new PublicKey(process.env.REACT_APP_MANGO_PROGRAM_ID_V2)
export const MANGO_GROUP_ACCOUNT = new PublicKey(process.env.REACT_APP_MANGO_GROUP_ACCOUNT)
export const MANGO_VAULT_ACCOUNT_USDC = new PublicKey(process.env.REACT_APP_MANGO_VAULT_ACCOUNT_USDC)

export const ORACLE_BTC_DEVNET = new PublicKey(process.env.REACT_APP_ORACLE_BTC_DEVNET)
export const ORACLE_ETH_DEVNET = new PublicKey(process.env.REACT_APP_ORACLE_ETH_DEVNET)
export const ORACLE_SOL_DEVNET = new PublicKey(process.env.REACT_APP_ORACLE_SOL_DEVNET)
export const ORACLE_SRM_DEVNET = new PublicKey(process.env.REACT_APP_ORACLE_SRM_DEVNET)

export const SOL_USDC_MARKET = new PublicKey(process.env.REACT_APP_SOL_USDC_MARKET)

export const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111')
export const TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
export const MEMO_PROGRAM_ID = new PublicKey('Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo')
export const RENT_PROGRAM_ID = new PublicKey('SysvarRent111111111111111111111111111111111')
export const CLOCK_PROGRAM_ID = new PublicKey('SysvarC1ock11111111111111111111111111111111')


