import { IDS } from "@blockworks-foundation/mango-client";
import { Connection, PublicKey } from "@solana/web3.js";

// if devnet =2 mainnet=0
export const idsIndex = 0;
const ids = IDS['groups'][idsIndex]


export const PLATFORM_ACCOUNT_KEY = "pacck"; //"platAccKey_11";
export const FUND_ACCOUNT_KEY = "facck"; //"fundAccKey_11";
export const MARGIN_ACCOUNT_KEY_1 = "macck1"; //"margin_account_key_11"
export const MARGIN_ACCOUNT_KEY_2 = "macck2";//"margin_account_key_22"

// export const PERP_ACCOUNT_KEY_1 = "pacck1"; //"margin_account_key_11"
// export const PERP_ACCOUNT_KEY_2 = "pacck2"; //"margin_account_key_11"
// export const PERP_ACCOUNT_KEY_3 = "pacck3"; //"margin_account_key_11"
// export const PERP_ACCOUNT_KEY_4 = "pacck4"; //"margin_account_key_11"

export const PRICE_ACCOUNT_KEY = "margin_account_key_22"

export const cluster = process.env.REACT_APP_CLUSTER_URL;

export const mangoCluster = "https://mango.rpcpool.com"
// export const cluster = "https://investin.rpcpool.com";
export const publicCluster = "https://solana-api.projectserum.com";
export const genesysGluster = "https://investinpro.genesysgo.net";

export const connection = new Connection(publicCluster, "processed");

export const adminAccount = new PublicKey(process.env.REACT_APP_ADMIN_ACCOUNT)
// 8dbbmZXbLsUirEsgaBVcPBEdciESza6L2zkEuer4crR
// DGWPuR54RgRYVdUHF8TaG7SP63BzWUSAYBScuAEbV1fD
export const programId = new PublicKey('8dbbmZXbLsUirEsgaBVcPBEdciESza6L2zkEuer4crR');
export const platformStateAccount = new PublicKey(process.env.REACT_APP_PLATFORM_STATE_ACCOUNT)
export const priceStateAccount = new PublicKey(process.env.REACT_APP_PRICE_STATE_ACCOUNT)

export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(process.env.REACT_APP_ASSOCIATED_TOKEN_PROGRAM_ID)

export const LIQUIDITY_POOL_PROGRAM_ID_V4 = new PublicKey(process.env.REACT_APP_LIQUIDITY_POOL_PROGRAM_ID_V4)
export const SERUM_PROGRAM_ID_V3 = new PublicKey(process.env.REACT_APP_SERUM_PROGRAM_ID_V3)
export const ORCA_SWAP_PROGRAM_ID = new PublicKey('9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP')

export const MANGO_GROUP_ACCOUNT = new PublicKey(ids.publicKey)
export const MANGO_PROGRAM_ID = new PublicKey(ids.mangoProgramId)

// USDC nodebank vault
export const MANGO_VAULT_ACCOUNT_USDC = new PublicKey(ids.tokens[0].nodeKeys[0])

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

export const PERP_MARKETS = [
    {
        "name": "BTC-PERP",
        "publicKey": "DtEcjPLyD4YtTBB4q8xwFZ9q49W89xZCZtJyrGebi5t8",
        "baseSymbol": "BTC",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 1,
        "bidsKey": "Bc8XaK5UTuDSCBtiESSUxBSb9t6xczhbAJnesPamMRir",
        "asksKey": "BkWRiarqxP5Gwx7115LQPbjRmr3NjuSRXWBnduXXLGWR",
        "eventsKey": "7t5Me8RieYKsFpfLEV8jnpqcqswNpyWD95ZqgUXuLV8Z",
        "contractSize": 0.0001,
        "perpMarketId": 1,
        "leverage": 20,
        "baseLotSize": 100,
        "quoteLotSize": 10
    },
    {
        "name": "ETH-PERP",
        "publicKey": "DVXWg6mfwFvHQbGyaHke4h3LE9pSkgbooDSDgA4JBC8d",
        "baseSymbol": "ETH",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 2,
        "bidsKey": "DQv2sWhaHYbKrobHH6jAdkAXw13mnDdM9hVfRQtrUcMe",
        "asksKey": "8NhLMV6huneGAqijuUgUFSshbAfXxdNj6ZMHSLb9aW8K",
        "eventsKey": "9vDfKNPJkCvQv9bzR4JNTGciQC2RVHPVNMMHiVDgT1mw",
        "contractSize": 0.001,
        "perpMarketId": 2,
        "leverage": 10,
        "baseLotSize": 1000,
        "quoteLotSize": 100
    },
    {
        "name": "SOL-PERP",
        "publicKey": "2TgaaVoHgnSeEtXvWTx13zQeTf4hYWAMEiMQdcG6EwHi",
        "baseSymbol": "SOL",
        "baseDecimals": 9,
        "quoteDecimals": 6,
        "marketIndex": 3,
        "bidsKey": "Fu8q5EiFunGwSRrjFKjRUoMABj5yCoMEPccMbUiAT6PD",
        "asksKey": "9qUxMSWBGAeNmXusQHuLfgSuYJqADyYoNLwZ63JJSi6V",
        "eventsKey": "31cKs646dt1YkA3zPyxZ7rUAkxTBz279w4XEobFXcAKP",
        "contractSize": 0.01,
        "perpMarketId": 3,
        "leverage": 10,
        "baseLotSize": 10000000,
        "quoteLotSize": 100
    },
    {
        "name": "AVAX-PERP",
        "publicKey": "EAC7jtzsoQwCbXj1M3DapWrNLnc3MBwXAarvWDPr2ZV9",
        "baseSymbol": "AVAX",
        "baseDecimals": 8,
        "quoteDecimals": 6,
        "marketIndex": 12,
        "bidsKey": "BD1vpQjLXx7Rmd5n1SFNTLcwujPYTnFpoaArvPd9ixB9",
        "asksKey": "8Q11iGHXFTr267J4bgbeEeWPYPSANVcs6NQWHQK4UrNs",
        "eventsKey": "5Grgo9kLu692SUcJ6S7jtbi1WkdwiyRWgThAfN1PcvbL",
        "contractSize": 0.01,
        "perpMarketId": 12,
        "leverage": 10,
        "baseLotSize": 1000000,
        "quoteLotSize": 100
    },
    {
        "name": "LUNA-PERP",
        "publicKey": "BCJrpvsB2BJtqiDgKVC4N6gyX1y24Jz96C6wMraYmXss",
        "baseSymbol": "LUNA",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 13,
        "bidsKey": "AiBurBkETJHHujZxNHm6UPvBQ1LLLkNkckPoZLeuLnS1",
        "asksKey": "7Vcbxj2M8fqaNGfRDsau47uXumfCBhCTA97D6PNDPWfe",
        "eventsKey": "HDJ43o9Dxxu6yWRWPEce44gtCHauRGLXJwwtvD7GwEBx",
        "contractSize": 0.01,
        "perpMarketId": 13,
        "leverage": 10,
        "baseLotSize": 10000,
        "quoteLotSize": 100
    },
]

export const MANGO_TOKENS = {
    'USDC': {
        symbol: "USDC",
        mintAddress: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        decimals: 6,
        rootKey: "AMzanZxMirPCgGcBoH9kw4Jzi9LFMomyUCXbpzDeL2T8",
        nodeKeys: ["BGcwkj1WudQwUUjFk78hAjwd1uAm8trh1N4CJSa51euh"],
        mangoTokenIndex: 15
    },
    'MNGO': {
        symbol: "MNGO",
        mintAddress: "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac",
        decimals: 6,
        rootKey: "8HjXYFntHMDNJKCJpHFufDaFYXfuAk6c6odfFnWc4xWy",
        nodeKeys: ["8XZx15vqdUbt3eVTXsxPfEMS3o2KXJ5sM7G2qXmmkETk"],
        mangoTokenIndex: 0
    },
    'BTC': {
        symbol: "BTC",
        mintAddress: "9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E",
        decimals: 6,
        rootKey: "8VwAANqu3t4KQKpMq7wrS6yg5GTHwJBFsrK4Tk2cFN3q",
        nodeKeys: ["7CfvGCV7qMf7im7mcqftZxQZGTweGappvL1maH7PMZ3Q"],
        mangoTokenIndex: 1
    },
    'ETH': {
        symbol: "ETH",
        mintAddress: "2FPyTwcZLUg1MDrwsyoP4D6s1tM7hAkHYRjkNb5w6Pxk",
        decimals: 6,
        rootKey: "FDpHjPQnUkmYVpAEVBpzb3sQgjZM7fanJoRb1VVtjF6u",
        nodeKeys: ["B6mYWs6PKda8DtJwvkvk2UV88NCChdmFGhcWSrgxY5vb"],
        mangoTokenIndex: 2
    },
    'WSOL': {
        symbol: "WSOL",
        mintAddress: "So11111111111111111111111111111111111111112",
        decimals: 9,
        rootKey: "7jH1uLmiB2zbHNe6juZZYjQCrvquakTwd3yMaQpeP8rR",
        nodeKeys: ["2bqJYcA1A8gw4qJFjyE2G4akiUunpd9rP6QzfnxHqSqr"],
        mangoTokenIndex: 3
    },
    'SRM': {
        symbol: "SRM",
        mintAddress: "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt",
        decimals: 6,
        rootKey: "AjMbjA1JsHh574Eo1RRV2XXtB8St139oBXKPXPo2HLdU",
        nodeKeys: ["qsGcM7VLiywm1wvvvjzWd7SynnyMcg8Pc7QxKUW4CUY"],
        mangoTokenIndex: 5
    },
    'RAY': {
        symbol: "RAY",
        mintAddress: "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
        decimals: 6,
        rootKey: "7TNHrBUDH3FL9uy9hxjmRcKNNaCBG9sYPuDJSJuj3LGs",
        nodeKeys: ["GDNCSCaVzhD2L164GwUv8JqTdaHCuYGg21JjXQDtuofk"],
        mangoTokenIndex: 6
    },
    'COPE': {
        symbol: "COPE",
        mintAddress: "8HGyAAB1yoM1ttS7pXjHMa3dukTFGQggnFFH3hJZgzQh",
        decimals: 6,
        rootKey: "6cMrtzhWNEEDkcSMx19orcred8h9HyRb31MtbCkKDdf6",
        nodeKeys: ["2CpaAtjDt4s9Fps7dyJUaUMeUGBDPAUUui5rxYbVzPfA"],
        mangoTokenIndex: 7

    },
    'FTT': {
        symbol: "FTT",
        mintAddress: "AGFEad2et2ZJif9jaGpdMixQqvW5i81aBdvKe7PHNfz3",
        decimals: 6,
        rootKey: "9i35wTe5W9vVLUJnzuhnFZbLThYJr2NF38MhEGVHJY5T",
        nodeKeys: ["8Q9JVDynPbyqXfnDXT31mncD7LAnoHAoSv2ywxZHjPFJ"],
        mangoTokenIndex: 8
    },
}


