import { LP_TOKENS, TOKENS, NATIVE_SOL, RAY_TOKENS } from "./tokens";
import { TEST_TOKENS, TEST_LP_TOKENS } from "./tokens";

import { LIQUIDITY_POOL_PROGRAM_ID_V4, SERUM_PROGRAM_ID_V3 } from "./constants";

export const devnet_pools = [
  {
    name: 'RAYT-USDR',
    coin: {...TEST_TOKENS.RAYT},
    pc: {...TEST_TOKENS.USDR},
    lp: { ...TEST_LP_TOKENS['RAYT-USDR'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'HeD1cekRWUNR25dcvW8c9bAHeKbr1r7qKEhv7pEegr4f',
    ammAuthority: 'DhVpojXMTbZMuTaCgiiaFU7U8GvEEhnYo4G9BUdiEYGh',
    ammOpenOrders: 'HboQAt9BXyejnh6SzdDNTx4WELMtRRPCr7pRSLpAW7Eq',
    ammTargetOrders: '6TzAjFPVZVMjbET8vUSk35J9U2dEWFCrnbHogsejRE5h',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,

    // dont care
    poolCoinTokenAccount: '3qbeXHwh9Sz4zabJxbxvYGJc57DZHrFgYMCWnaeNJENT',
    poolPcTokenAccount: 'FrGPG5D4JZVF5ger7xSChFVFL8M9kACJckzyCz8tVowz',
    poolWithdrawQueue: 'DD9nUbJoUbuE3FampcJeLfDPb5zKGvR7Ho7HE5rpcBGx',
    poolTempLpTokenAccount: '249BW2tWhsvwEFtWabpTXXX17Vh7NQSeHS4W7Ku6b27R',
    
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '3tsrPhKrWHWMB8RiPaqNxJ8GnBhZnDqL4wcu5EAMFeBe',

    serumBids: 'ANHHchetdZVZBuwKWgz8RSfVgCDsRpW9i2BNWrmG9Jh9',
    serumAsks: 'ESSri17GNbVttqrp7hrjuXtxuTcCqytnrMkEqr29gMGr',
    serumEventQueue: 'FGAW7QqNJGFyhakh5jPzGowSb8UqcSJ95ZmySeBgmVwt',
    serumCoinVaultAccount: 'E1E5kQqWXkXbaqVzpY5P2EQUSi8PNAHdCnqsj3mPWSjG',
    serumPcVaultAccount: '3sj6Dsw8fr8MseXpCnvuCSczR8mQjCWNyWDC5cAfEuTq',
    serumVaultSigner: 'C2fDkZJqHH5PXyQ7UWBNZsmu6vDXxrEbb9Ex9KF7XsAE'
  },
  {
    name: 'ALPHA-USDR',
    coin: {...TEST_TOKENS.ALPHA},
    pc: {...TEST_TOKENS.USDR},
    lp: { ...LP_TOKENS['RAY-USDP-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'HeD1cekRWUNR25dcvW8c9bAHeKbr1r7qKEhv7pEegr4f',
    ammAuthority: '5v3awV3f3RseEgUc6zV5jkBG3SmW9VsiHsVYtsP17CxP',
    ammOpenOrders: 'HboQAt9BXyejnh6SzdDNTx4WELMtRRPCr7pRSLpAW7Eq',
    ammTargetOrders: '6TzAjFPVZVMjbET8vUSk35J9U2dEWFCrnbHogsejRE5h',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,

    // dont care
    poolCoinTokenAccount: 'DUn4i71SXksHN7KtveP4uauWqsnfdSHa4PoEkzN8qqN6',
    poolPcTokenAccount: 'FrGPG5D4JZVF5ger7xSChFVFL8M9kACJckzyCz8tVowz',
    poolWithdrawQueue: 'DD9nUbJoUbuE3FampcJeLfDPb5zKGvR7Ho7HE5rpcBGx',
    poolTempLpTokenAccount: '249BW2tWhsvwEFtWabpTXXX17Vh7NQSeHS4W7Ku6b27R',
    
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '3tsrPhKrWHWMB8RiPaqNxJ8GnBhZnDqL4wcu5EAMFeBe',

    serumBids: 'AvKStCiY8LTp3oDFrMkiHHxxhxk4sQUWnGVcetm4kRpy',
    serumAsks: 'Hj9kckvMX96mQokfMBzNCYEYMLEBYKQ9WwSc1GxasW11',
    serumEventQueue: '58KcficuUqPDcMittSddhT8LzsPJoH46YP4uURoMo5EB',
    serumCoinVaultAccount: '2kVNVEgHicvfwiyhT2T51YiQGMPFWLMSp8qXc1hHzkpU',
    serumPcVaultAccount: '5AXZV7XfR7Ctr6yjQ9m9dbgycKeUXWnWqHwBTZT6mqC7',
    serumVaultSigner: 'HzWpBN6ucpsA9wcfmhLAFYqEUmHjE9n2cGHwunG5avpL'
  },
]
export const devnet2_pools = [
  {
    name: 'ALPHA-USDP',
    coin: { ...TEST_TOKENS.ALPHA },
    pc: { ...TEST_TOKENS.USDP },
    lp: { ...LP_TOKENS['RAY-USDP-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'DVa7Qmb5ct9RCpaU7UTpSaf3GVMYz17vNVU67XpdCRut',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'HboQAt9BXyejnh6SzdDNTx4WELMtRRPCr7pRSLpAW7Eq',
    ammTargetOrders: '3K2uLkKwVVPvZuMhcQAPLF8hw95somMeNwJS7vgWYrsJ',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,

    poolCoinTokenAccount: 'DUn4i71SXksHN7KtveP4uauWqsnfdSHa4PoEkzN8qqN6',
    poolPcTokenAccount: 'BUdDS4AUMSsvQ1QyHe4LLagvkFfUU4TW17udvxaDJaxR',

    // dont care
    poolWithdrawQueue: '8VuvrSWfQP8vdbuMAP9AkfgLxU9hbRR6BmTJ8Gfas9aK',
    poolTempLpTokenAccount: 'FBzqDD1cBgkZ1h6tiZNFpkh4sZyg6AG8K5P9DSuJoS5F',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: 'teE55QrL4a4QSfydR9dnHF97jgCfptpuigbb53Lo95g',
    serumBids: 'AvKStCiY8LTp3oDFrMkiHHxxhxk4sQUWnGVcetm4kRpy',
    serumAsks: 'Hj9kckvMX96mQokfMBzNCYEYMLEBYKQ9WwSc1GxasW11',
    serumEventQueue: '58KcficuUqPDcMittSddhT8LzsPJoH46YP4uURoMo5EB',
    serumCoinVaultAccount: '2kVNVEgHicvfwiyhT2T51YiQGMPFWLMSp8qXc1hHzkpU',
    serumPcVaultAccount: '5AXZV7XfR7Ctr6yjQ9m9dbgycKeUXWnWqHwBTZT6mqC7',
    serumVaultSigner: 'HzWpBN6ucpsA9wcfmhLAFYqEUmHjE9n2cGHwunG5avpL'
  },
  {
    name: 'BETA-USDP',
    coin: { ...TEST_TOKENS.BETA },
    pc: { ...TEST_TOKENS.USDP },
    lp: { ...LP_TOKENS['ETH-USDT-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'He3iAEV5rYjv6Xf7PxKro19eVrC3QAcdic5CF2D2obPt',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: '8x4uasC632WSrk3wgwoCWHy7MK7Xo2WKAe9vV93tj5se',
    ammTargetOrders: 'G1eji3rrfRFfvHUbPEEbvnjmJ4eEyXeiJBVbMTUPfKL1',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    
    poolCoinTokenAccount: '2Ab9oAp9XcarKgdthdAtTitAHctuEkafKHh2GtzSJRyt',
    poolPcTokenAccount: 'BUdDS4AUMSsvQ1QyHe4LLagvkFfUU4TW17udvxaDJaxR',
    
    // dont care
    poolWithdrawQueue: 'EispXkJcfh2PZA2fSXWsAanEGq1GHXzRRtu1DuqADQsL',
    poolTempLpTokenAccount: '9SrcJk8TB4JvutZcA4tMvvkdnxCXda8Gtepre7jcCaQr',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '7dLVkUfBVfCGkFhSXDCq1ukM9usathSgS716t643iFGF',
    serumBids: 'J8a3dcUkMwrE5kxN86gsL1Mwrg63RnGdvWsPbgdFqC6X',
    serumAsks: 'F6oqP13HNZho3bhwuxTmic4w5iNgTdn89HdihMUNR24i',
    serumEventQueue: 'CRjXyfAxboMfCAmsvBw7pdvkfBY7XyGxB7CBTuDkm67v',
    serumCoinVaultAccount: '2CZ9JbDYPux5obFXb9sefwKyG6cyteNBSzbstYQ3iZxE',
    serumPcVaultAccount: 'D2f4NG1NC1yeBM2SgRe5YUF91w3M4naumGQMWjGtxiiE',
    serumVaultSigner: 'CVVGPFejAj3A75qPy2116iJFma7zGEuL8DgnxhwUaFBF'
  },
]


export const pools = [
  {
    name: 'RAY-USDT',
    coin: { ...TOKENS.RAY },
    pc: { ...TOKENS.USDT },
    lp: { ...LP_TOKENS['RAY-USDT-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'DVa7Qmb5ct9RCpaU7UTpSaf3GVMYz17vNVU67XpdCRut',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: '7UF3m8hDGZ6bNnHzaT2YHrhp7A7n9qFfBj6QEpHPv5S8',
    ammTargetOrders: '3K2uLkKwVVPvZuMhcQAPLF8hw95somMeNwJS7vgWYrsJ',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: '3wqhzSB9avepM9xMteiZnbJw75zmTBDVmPFLTQAGcSMN',
    poolPcTokenAccount: '5GtSbKJEPaoumrDzNj4kGkgZtfDyUceKaHrPziazALC1',
    poolWithdrawQueue: '8VuvrSWfQP8vdbuMAP9AkfgLxU9hbRR6BmTJ8Gfas9aK',
    poolTempLpTokenAccount: 'FBzqDD1cBgkZ1h6tiZNFpkh4sZyg6AG8K5P9DSuJoS5F',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: 'teE55QrL4a4QSfydR9dnHF97jgCfptpuigbb53Lo95g',
    serumBids: 'AvKStCiY8LTp3oDFrMkiHHxxhxk4sQUWnGVcetm4kRpy',
    serumAsks: 'Hj9kckvMX96mQokfMBzNCYEYMLEBYKQ9WwSc1GxasW11',
    serumEventQueue: '58KcficuUqPDcMittSddhT8LzsPJoH46YP4uURoMo5EB',
    serumCoinVaultAccount: '2kVNVEgHicvfwiyhT2T51YiQGMPFWLMSp8qXc1hHzkpU',
    serumPcVaultAccount: '5AXZV7XfR7Ctr6yjQ9m9dbgycKeUXWnWqHwBTZT6mqC7',
    serumVaultSigner: 'HzWpBN6ucpsA9wcfmhLAFYqEUmHjE9n2cGHwunG5avpL'
  },
  {
    name: 'ETH-USDT',
    coin: { ...TOKENS.ETH },
    pc: { ...TOKENS.USDT },
    lp: { ...LP_TOKENS['ETH-USDT-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'He3iAEV5rYjv6Xf7PxKro19eVrC3QAcdic5CF2D2obPt',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: '8x4uasC632WSrk3wgwoCWHy7MK7Xo2WKAe9vV93tj5se',
    ammTargetOrders: 'G1eji3rrfRFfvHUbPEEbvnjmJ4eEyXeiJBVbMTUPfKL1',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: 'DZZwxvJakqbraXTbjRW3QoGbW5GK4R5nmyrrGrFMKWgh',
    poolPcTokenAccount: 'HoGPb5Rp44TyR1EpM5pjQQyFUdgteeuzuMHtimGkAVHo',
    poolWithdrawQueue: 'EispXkJcfh2PZA2fSXWsAanEGq1GHXzRRtu1DuqADQsL',
    poolTempLpTokenAccount: '9SrcJk8TB4JvutZcA4tMvvkdnxCXda8Gtepre7jcCaQr',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '7dLVkUfBVfCGkFhSXDCq1ukM9usathSgS716t643iFGF',
    serumBids: 'J8a3dcUkMwrE5kxN86gsL1Mwrg63RnGdvWsPbgdFqC6X',
    serumAsks: 'F6oqP13HNZho3bhwuxTmic4w5iNgTdn89HdihMUNR24i',
    serumEventQueue: 'CRjXyfAxboMfCAmsvBw7pdvkfBY7XyGxB7CBTuDkm67v',
    serumCoinVaultAccount: '2CZ9JbDYPux5obFXb9sefwKyG6cyteNBSzbstYQ3iZxE',
    serumPcVaultAccount: 'D2f4NG1NC1yeBM2SgRe5YUF91w3M4naumGQMWjGtxiiE',
    serumVaultSigner: 'CVVGPFejAj3A75qPy2116iJFma7zGEuL8DgnxhwUaFBF'
  },
  {
    name: 'SRM-USDT',
    coin: { ...TOKENS.SRM },
    pc: { ...TOKENS.USDT },
    lp: { ...LP_TOKENS['SRM-USDT-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'af8HJg2ffWoKJ6vKvkWJUJ9iWbRR83WgXs8HPs26WGr',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: '8E2GLzSgLmzWdpdXjjEaHbPXRXsA5CFehg6FP6N39q2e',
    ammTargetOrders: '8R5TVxXvRfCaYvT493FWAJyLt8rVssUHYVGbGupAbYaQ',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: '3qbeXHwh9Sz4zabJxbxvYGJc57DZHrFgYMCWnaeNJENT',
    poolPcTokenAccount: 'FrGPG5D4JZVF5ger7xSChFVFL8M9kACJckzyCz8tVowz',
    poolWithdrawQueue: 'DD9nUbJoUbuE3FampcJeLfDPb5zKGvR7Ho7HE5rpcBGx',
    poolTempLpTokenAccount: '249BW2tWhsvwEFtWabpTXXX17Vh7NQSeHS4W7Ku6b27R',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '3tsrPhKrWHWMB8RiPaqNxJ8GnBhZnDqL4wcu5EAMFeBe',
    serumBids: 'EE2CYFBSoMvcUR9mkEF6tt8kBFhW9zcuFmYqRM9GmqYb',
    serumAsks: 'nkNzrV3ZtkWCft6ykeNGXXCbNSemqcauYKiZdf5JcKQ',
    serumEventQueue: '2i34Kriz23ZaQaJK6FVhzkfLhQj8DSqdQTmMwz4FF9Cf',
    serumCoinVaultAccount: 'GxPFMyeb7BUnu2mtGV2Zvorjwt8gxHqwL3r2kVDe6rZ8',
    serumPcVaultAccount: '149gvUQZeip4u8bGra5yyN11btUDahDVHrixzknfKFrL',
    serumVaultSigner: '4yWr7H2p8rt11QnXb2yxQF3zxSdcToReu5qSndWFEJw'
  },
]