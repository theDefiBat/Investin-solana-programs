import { IDS } from "@blockworks-foundation/mango-client";
import { Connection, PublicKey } from "@solana/web3.js";

// if devnet =2 mainnet=0
export const idsIndex = 2;
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
export const connection = new Connection(cluster, "confirmed");

export const adminAccount = new PublicKey(process.env.REACT_APP_ADMIN_ACCOUNT)
export const programId = new PublicKey(process.env.REACT_APP_PROGRAMID);
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
        "name": "MNGO-PERP",
        "publicKey": "98wPi7vBkiJ1sXLPipQEjrgHYcMBcNUsg9avTyWUi26j",
        "baseSymbol": "MNGO",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 0,
        "bidsKey": "5Zpfa8VbFKBJQFueomXz82EjbbtP6nFFQmBkHPCxfKpb",
        "asksKey": "4Z9xHcCUMY9QLevHu3JpzxnwiHzzaQACMJERZ1XVJcSa",
        "eventsKey": "uaUCSQejWYrDeYSuvn4As4kaCwJ2rLnRQSsSjY3ogZk",
        "contractSize": 1

    },
    {
        "name": "BTC-PERP",
        "publicKey": "FHQtNjRHA9U5ahrH7mWky3gamouhesyQ5QvpeGKrTh2z",
        "baseSymbol": "BTC",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 1,
        "bidsKey": "F1Dcnq6F8NXR3gXADdsYqrXYBUUwoT7pfCtRuQWSyQFd",
        "asksKey": "BFEBZsLYmEhj4quWDRKbyMKhW1Q9c7gu3LqsnipNGTVn",
        "eventsKey": "Bu17U2YdBM9gRrqQ1zD6MpngQBb71RRAAn8dbxoFDSkU",
        "contractSize": 0.0001

    },
    {
        "name": "ETH-PERP",
        "publicKey": "8jKPf3KJKWvvSbbYnunwZYv62UoRPpyGb93NWLaswzcS",
        "baseSymbol": "ETH",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 2,
        "bidsKey": "6jGBscmZgRXk6oVLWbnQDpRftmzrDVu82TARci9VHKuW",
        "asksKey": "FXSvghvoaWFHRXzWUHi5tjK9YhgcPgMPpypFXBd4Aq3r",
        "eventsKey": "8WLv5fKLYkyZpFG74kRmp2RALHQFcNKmH7eJn8ebHC13",
        "contractSize": 0.01

    },
    {
        "name": "SOL-PERP",
        "publicKey": "58vac8i9QXStG1hpaa4ouwE1X7ngeDjY9oY7R15hcbKJ",
        "baseSymbol": "SOL",
        "baseDecimals": 9,
        "quoteDecimals": 6,
        "marketIndex": 3,
        "bidsKey": "7HRgm8iXEDx2TmSETo3Lq9SXkF954HMVKNiq8t5sKvQS",
        "asksKey": "4oNxXQv1Rx3h7aNWjhTs3PWBoXdoPZjCaikSThV4yGb8",
        "eventsKey": "CZ5MCRvkN38d5pnZDDEEyMiED3drgDUVpEUjkuJq31Kf",
        "contractSize": 0.01
    },
    {
        "name": "ADA-PERP",
        "publicKey": "Ai2579GtT3mYEu6LDB3FoZxJT7tiuo91t1joreQTfj9p",
        "baseSymbol": "ADA",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 7,
        "bidsKey": "5ugnXufA13HVgY6P9QLkFiSR6jy6XUv96WLbDV2Sf5i5",
        "asksKey": "45MdNs8jpedfHLaHvL7nyfhHSwHXzTYzPQvR2FAnXG1p",
        "eventsKey": "5v5fz2cCSy2VvrgVf5Vu7PF23RiZjv6BL36bgg48bA1c",
        "contractSize": 1

    },
    {
        "name": "FTT-PERP",
        "publicKey": "8fKNzMe22bZ6H9TP8KpyM8B6b6DhZQyNmodChvQRbV8P",
        "baseSymbol": "FTT",
        "baseDecimals": 6,
        "quoteDecimals": 6,
        "marketIndex": 8,
        "bidsKey": "78fRmLeyvMQ96GwJuusxN5Zn2QKbYh752GoAGc2qVE6q",
        "asksKey": "9GTHBjPNUBBWuqhYinxwqdesW8amzFBCAVS96waKkE5L",
        "eventsKey": "7rswj7FVZcMYUKxcTLndZhWBmuVNc2GuxqjuXU8KcPWv",
        "contractSize": 0.1

    }
]

export const MANGO_TOKENS = [
    {
        "symbol": "USDC",
        "mintKey": "8FRFC6MoGGkMFQwngccyu69VnYbzykGeez7ignHVAFSN",
        "decimals": 6,
        "rootKey": "HUBX4iwWEUK5VrXXXcB7uhuKrfT4fpu2T9iZbg712JrN",
        "nodeKeys": ["J2Lmnc1e4frMnBEJARPoHtfpcohLfN67HdK1inXjTFSM"],
        "mangoTokenIndex" : 15
    },
    {
        "symbol": "MNGO",
        "mintKey": "Bb9bsTQa1bGEtQ5KagGkvSHyuLqDWumFUcRqFusFNJWC",
        "decimals": 6,
        "rootKey": "CY4nMV9huW5KCYFxWChrmoLwGCsZiXoiREeo2PMrBm5o",
        "nodeKeys": ["6rkPNJTXF37X6Pf5ct5Y6E91PozpZpZNNU1AGATomKjD"],
        "mangoTokenIndex" : 0
    },
    {
        "symbol": "BTC",
        "mintKey": "3UNBZ6o52WTWwjac2kPUb4FyodhU1vFkRJheu1Sh2TvU",
        "decimals": 6,
        "rootKey": "BeEoyDq1v2DYJCoXDQAJKfmrsoRRvfmV856f2ijkXbtp",
        "nodeKeys": ["4X3nP921qyh6BKJSAohKGNCykSXahFFwg1LxtC993Fai"],
        "mangoTokenIndex" : 1
    },
    {
        "symbol": "ETH",
        "mintKey": "Cu84KB3tDL6SbFgToHMLYVDJJXdJjenNzSKikeAvzmkA",
        "decimals": 6,
        "rootKey": "AxwY5sgwSq5Uh8GD6A6ZtSzGd5fqvW2hwgGLLgZ4v2eW",
        "nodeKeys": ["3FPjawEtvrwvwtAetaURTbkkucu9BJofxWZUNPGHJtHg"],
        "mangoTokenIndex" : 2
    },
    {
        "symbol": "SOL",
        "mintKey": "So11111111111111111111111111111111111111112",
        "decimals": 9,
        "rootKey": "8GC81raaLjhTx3yedctxCJW46qdmmSRybH2s1eFYFFxT",
        "nodeKeys": ["7mYqCavd1K24fnL3oKTpX3YM66W5gfikmVHJWM3nrWKe"],
        "mangoTokenIndex" : 3

    },
    {
        "symbol": "SRM",
        "mintKey": "AvtB6w9xboLwA145E221vhof5TddhqsChYcx7Fy3xVMH",
        "decimals": 6,
        "rootKey": "73W29LAZog2zSyE1uNYivBW8SMZQX3WBX4qfTMrMJxW2",
        "nodeKeys": ["9wkpWmkSUSn9fitLhVh12cLbiDa5Bbhf6ZBGmPtcdMqN"],
        "mangoTokenIndex" : 4
    },
    {
        "symbol": "RAY",
        "mintKey": "3YFQ7UYJ7sNGpXTKBxM3bYLVxKpzVudXAe4gLExh5b3n",
        "decimals": 6,
        "rootKey": "49S76N83tSBBozugLtNYrMojFqDb3VvYq4wBB6bcAhfV",
        "nodeKeys": ["JBHBTED3ttzk5u3U24txdjBFadm4Dnohb7g2pwcxU4rx"],
        "mangoTokenIndex" : 5

    },
    {
        "symbol": "USDT",
        "mintKey": "DAwBSXe6w9g37wdE2tCrFbho3QHKZi4PjuBytQCULap2",
        "decimals": 6,
        "rootKey": "7JTHE8C1kvB4h67RVvhdHjDqHXsWkSeoKcBsHV7wVhu",
        "nodeKeys": ["ERkKh9yUKzJ3kkHWhMNd3xGaync11TpzQiDFukEatHEQ"],
        "mangoTokenIndex" : 6
    },
    {
        "symbol": "FTT",
        "mintKey": "Fxh4bpZnRCnpg2vcH11ttmSTDSEeC5qWbPRZNZWnRnqY",
        "decimals": 6,
        "rootKey": "4m3kgpf8qQRvaoTJdqmdeiRL5u2NaifYyTAHccKMtQhT",
        "nodeKeys": ["2k89sUjCE2ZSm4MPhXM9JV1zFEV2SjgEzvvJN6EsMFWa"],
        "mangoTokenIndex" : 8
    }
]


