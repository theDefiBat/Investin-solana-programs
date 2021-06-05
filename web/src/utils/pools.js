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
    poolCoinTokenAccount: '3qbeXHwh9Sz4zabJxbxvYGJc57DZHrFgYMCWnaeNJENT',
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
    name: 'WSOL-USDC',
    coin: { ...TOKENS.WSOL },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['SOL-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: '58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'HRk9CMrpq7Jn9sh7mzxE8CChHG8dneX9p475QKz4Fsfc',
    ammTargetOrders: 'CZza3Ej4Mc58MnxWA385itCC9jCo3L1D7zc3LKy1bZMR',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: 'DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz',
    poolPcTokenAccount: 'HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz',
    poolWithdrawQueue: 'G7xeGGLevkRwB5f44QNgQtrPKBdMfkT6ZZwpS9xcC97n',
    poolTempLpTokenAccount: 'Awpt6N7ZYPBa4vG4BQNFhFxDj4sxExAA9rpBAoBw2uok',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT',
    serumBids: '14ivtgssEBoBjuZJtSAPKYgpUK7DmnSwuPMqJoVTSgKJ',
    serumAsks: 'CEQdAFKdycHugujQg9k2wbmxjcpdYZyVLfV9WerTnafJ',
    serumEventQueue: '5KKsLVU6TcbVDK4BS6K1DGDxnh4Q9xjYJ8XaDCG5t8ht',
    serumCoinVaultAccount: '36c6YqAwyGKQG66XEp2dJc5JqjaBNv7sVghEtJv4c7u6',
    serumPcVaultAccount: '8CFo8bL8mZQK8abbFyypFMwEDd8tVJjHTTojMLgQTUSZ',
    serumVaultSigner: 'F8Vyqk3unwxkXukZFQeYyGmFfTG3CAX4v24iyrjEYBJV',
    official: true
  },
  {
    name: 'SRM-USDC',
    coin: { ...TOKENS.SRM },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['SRM-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: '8tzS7SkUZyHPQY7gLqsMCXZ5EDCgjESUHcB17tiR1h3Z',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'GJwrRrNeeQKY2eGzuXGc3KBrBftYbidCYhmA6AZj2Zur',
    ammTargetOrders: '26LLpo8rscCpMxyAnJsqhqESPnzjMGiFdmXA4eF2Jrk5',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: 'zuLDJ5SEe76L3bpFp2Sm9qTTe5vpJL3gdQFT5At5xXG',
    poolPcTokenAccount: '4usvfgPDwXBX2ySX11ubTvJ3pvJHbGEW2ytpDGCSv5cw',
    poolWithdrawQueue: '7c1VbXTB7Xqx5eQQeUxAu5o6GHPq3P1ByhDsnRRUWYxB',
    poolTempLpTokenAccount: '2sozAi6zXDUCCkpgG3usphzeCDm4e2jTFngbm5atSdC9',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: 'ByRys5tuUWDgL73G8JBAEfkdFf8JWBzPBDHsBVQ5vbQA',
    serumBids: 'AuL9JzRJ55MdqzubK4EutJgAumtkuFcRVuPUvTX39pN8',
    serumAsks: '8Lx9U9wdE3afdqih1mCAXy3unJDfzSaXFqAvoLMjhwoD',
    serumEventQueue: '6o44a9xdzKKDNY7Ff2Qb129mktWbsCT4vKJcg2uk41uy',
    serumCoinVaultAccount: 'Ecfy8et9Mft9Dkavnuh4mzHMa2KWYUbBTA5oDZNoWu84',
    serumPcVaultAccount: 'hUgoKy5wjeFbZrXDW4ecr42T4F5Z1Tos31g68s5EHbP',
    serumVaultSigner: 'GVV4ZT9pccwy9d17STafFDuiSqFbXuRTdvKQ1zJX6ttX',
    official: true
  },
  {
    name: 'STEP-USDC',
    coin: { ...TOKENS.STEP },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['STEP-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: '4Sx1NLrQiK4b9FdLKe2DhQ9FHvRzJhzKN3LoD6BrEPnf',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'EXgME2sUuzBxEc2wuyoSZ8FZNZMC3ChhZgFZRAW3nCQG',
    ammTargetOrders: '78bwAGKJjaiPQqmwKmbj4fhrRTLAdzwqNwpFdpTzrhk1',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: '8Gf8Cc6yrxtfUZqM2vf2kg5uR9bGPfCHfzdYRVBAJSJj',
    poolPcTokenAccount: 'ApLc86fHjVbGbU9QFzNPNuWM5VYckZM92q6sgJN1SGYn',
    poolWithdrawQueue: '5bzBcB7cnJYGYvGPFxKcZETn6sGAyBbXgFhUbefbagYh',
    poolTempLpTokenAccount: 'CpfWKDYNYfvgk42tqR8HEHUWohGSJjASXfRBm3yaKJre',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '97qCB4cAVSTthvJu3eNoEx6AY6DLuRDtCoPm5Tdyg77S',
    serumBids: '5Xdpf7CMGFDkJj1smcVQAAZG6GY9gqAns18QLKbPZKsw',
    serumAsks: '6Tqwg8nrKJrcqsr4zR9wJuPv3iXsHAMN65FxwJ3RMH8S',
    serumEventQueue: '5frw4m8pEZHorTKVzmMzvf8xLUrj65vN7wA57KzaZFK3',
    serumCoinVaultAccount: 'CVNye3Xr9Jv26c8TVqZZHq4F43BhoWWfmrzyp1M9YA67',
    serumPcVaultAccount: 'AnGbReAhCDFkR83nB8mXTDX5dQJFB8Pwicu6pGMfCLjt',
    serumVaultSigner: 'FbwU5U1Doj2PSKRJi7pnCny4dFPPJURwALkFhHwdHaMW',
    official: true
  },

  {
    name: 'ALEPH-USDC',
    coin: { ...TOKENS.ALEPH },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['ALEPH-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'GDHXjn9wF2zxW35DBkCegWQdoTfFBC9LXt7D5ovJxQ5B',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'AtUeUK7MZayoDktjrRSJAFsyPiPwPsbAeTsunM5pSnnK',
    ammTargetOrders: 'FMYSGYEL1CPYz8cpgAor5jV2HqeEQRDLMEggoz6wAiFV',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: 'BT3QMKHrha4fhqpisnYKaPDsv42XeHU2Aovhdu5Bazru',
    poolPcTokenAccount: '9L4tXPyuwuLhmtmX4yaRTK6TB7tYFNHupeENoCdPceq',
    poolWithdrawQueue: '4nRbmEUp7DQroG71jXv6cJjrhnh91ePdPhzmBSjinwB8',
    poolTempLpTokenAccount: '9JdpGvmo6aPZYf4hkiZNUjceXgd2RtR1fJgvjuoAuhsM',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: 'GcoKtAmTy5QyuijXSmJKBtFdt99e6Buza18Js7j9AJ6e',
    serumBids: 'HmpcmzzajDvhFSXb4pmJo5mb23zW8Cj9FEeB3hVT78jV',
    serumAsks: '8sfGm6jsFTAcb4oLuqMKr1xNEBd5CXuNPAKZEdbeezA',
    serumEventQueue: '99Cd6D9QnFfTdKpcwtoF3zAZdQAuZQi5NsPMERresj1r',
    serumCoinVaultAccount: 'EBRqW7DaUGFBHRbfgRagpSf9jTSS3yp9MAi3RvabdBGz',
    serumPcVaultAccount: '9QTMfdkgPWqLriB9J7FcYvroUEqfw6zW2VCi1dAabdUt',
    serumVaultSigner: 'HKt6xFufxTBBs719WQPbro9t1DfDxffurxFhTPntMgoe',
    official: true
  },
  {
    name: 'ETH-USDC',
    coin: { ...TOKENS.ETH },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['ETH-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'AoPebtuJC4f2RweZSxcVCcdeTgaEXY64Uho8b5HdPxAR',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: '7PwhFjfFaYp7w9N8k2do5Yz7c1G5ebp3YyJRhV4pkUJW',
    ammTargetOrders: 'BV2ucC7miDqsmABSkXGzsibCVWBp7gGPcvkhevDSTyZ1',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: 'EHT99uYfAnVxWHPLUMJRTyhD4AyQZDDknKMEssHDtor5',
    poolPcTokenAccount: '58tgdkogRoMsrXZJubnFPsFmNp5mpByEmE1fF6FTNvDL',
    poolWithdrawQueue: '9qPsKm82ZFacGn4ipV1DH85k7efP21Zbxrxbxm5v3GPb',
    poolTempLpTokenAccount: '2WtX2ow4h5FK1vb8VjwpJ3hmwmYKfJfa1hy1rcDBohBT',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '4tSvZvnbyzHXLMTiFonMyxZoHmFqau1XArcRCVHLZ5gX',
    serumBids: '8tFaNpFPWJ8i7inhKSfAcSestudiFqJ2wHyvtTfsBZZU',
    serumAsks: '2po4TC8qiTgPsqcnbf6uMZRMVnPBzVwqqYfHP15QqREU',
    serumEventQueue: 'Eac7hqpaZxiBtG4MdyKpsgzcoVN6eMe9tAbsdZRYH4us',
    serumCoinVaultAccount: '7Nw66LmJB6YzHsgEGQ8oDSSsJ4YzUkEVAvysQuQw7tC4',
    serumPcVaultAccount: 'EsDTx47jjFACkBhy48Go2W7AQPk4UxtT4765f3tpK21a',
    serumVaultSigner: 'C5v68qSzDdGeRcs556YoEMJNsp8JiYEiEhw2hVUR8Z8y',
    official: true
  },
  {
    name: 'ROPE-USDC',
    coin: { ...TOKENS.ROPE },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['ROPE-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'BuS4ScFcZjEBixF1ceCTiXs4rqt4WDfXLoth7VcM2Eoj',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'ASkE1yKPBei2aUxKHrLRptB2gpC3a6oTSxafMikoHYTG',
    ammTargetOrders: '5isDwR41fBJocfmcrcfwRtTnmSf7CdssdpsmBy2N2Eym',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: '3mS8mb1vDrD45v4zoxbSdrvbyVM1pBLM31cYLT2RfS2U',
    poolPcTokenAccount: 'BWfzmvvXhQ5V8ZWDMC4u82sEWgc6HyRLnq6nauwrtz5x',
    poolWithdrawQueue: '9T1cwwE5zZr3D2Rim8e5xnJoPJ9yKbTXvaRoxeVoqffo',
    poolTempLpTokenAccount: 'FTFx4Vg6hgKLZMLBUvazvPbM7AzDe5GpfeBZexe2S6WJ',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: '4Sg1g8U2ZuGnGYxAhc6MmX9MX7yZbrrraPkCQ9MdCPtF',
    serumBids: 'BDYAnAUSoBTtX7c8TKHeqmSy7U91V2pDg8ojvLs2fnCb',
    serumAsks: 'Bdm3R8X7Vt1FpTruE9SQVESSd3BjAyFhcobPwAoK2LSw',
    serumEventQueue: 'HVzqLTfcZKVC2PanNpyt8jVRJfDW8M5LgDs5NVVDa4G3',
    serumCoinVaultAccount: 'F8PdvS5QFhSqgVdUFo6ivXdXC4nDEiKGc4XU97ZhCKgH',
    serumPcVaultAccount: '61zxdnLpgnFgdk9Jom5f6d6cZ6cTbwnC6QqmJag1N9jB',
    serumVaultSigner: 'rCFXUwdmQvRK9jtnCip3SdDm1cLn8nB6HHgEHngzfjQ',
    official: true
  },
  {
    name: 'MEDIA-USDC',
    coin: { ...TOKENS.MEDIA },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['MEDIA-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: '94CQopiGxxUXf2avyMZhAFaBdNatd62ttYGoTVQBRGdi',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'EdS5vqjihxRbRujPkqqzHYwBqcTP9QPbrBc9CDtnBDwo',
    ammTargetOrders: '6Rfew8qvNp97PVN14C9Wg8ybqRdF9HUEUhuqqZBWcAUW',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: '7zfTWDFmMi3Tzbbd3FZ2vZDdBm1w7whiZq1DrCxAHwMj',
    poolPcTokenAccount: 'FWUnfg1hHuanU8LxJv31TAfEWSvuWWffeMmHpcZ9BYVr',
    poolWithdrawQueue: 'F7MUnGrShtQqSvi9DoWyBNRo7FUpRiYPsS9aw77auhiS',
    poolTempLpTokenAccount: '7oX2VcPYwEV6EUUyMUoTKVVxAPAvGQZcGiGzotX43wNM',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: 'FfiqqvJcVL7oCCu8WQUMHLUC2dnHQPAPjTdSzsERFWjb',
    serumBids: 'GmqbTDL5QSAhWL7UsE8MriTHSnodWM1HyGR8Cn8GzZV5',
    serumAsks: 'CrTBp7ThkRRYJBL4tprke2VbKYj2wSxJp3Q1LDoHcQwP',
    serumEventQueue: 'HomZxFZNGmH2XedBavMsrXgLnWFpMLT95QV8nCYtKszd',
    serumCoinVaultAccount: 'D8ToFvpVWmNnfJzjHuumRJ4eoJc39hsWWcLtFZQpzQTt',
    serumPcVaultAccount: '6RSpnBYaegSKisXaJxeP36mkdVPe9SP3p2kDERz8Ahhi',
    serumVaultSigner: 'Cz2m3hW2Vcb8oEFz12uoWcdq8mKb9D1N7RTyXpigoFXU',
    official: true
  },

  {
    name: 'MER-USDC',
    coin: { ...TOKENS.MER },
    pc: { ...TOKENS.USDC },
    lp: { ...LP_TOKENS['MER-USDC-V4'] },

    version: 4,
    programId: LIQUIDITY_POOL_PROGRAM_ID_V4,

    ammId: 'BkfGDk676QFtTiGxn7TtEpHayJZRr6LgNk9uTV2MH4bR',
    ammAuthority: '5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1',
    ammOpenOrders: 'FNwXaqyYNKNwJ8Qc39VGzuGnPcNTCVKExrgUKTLCcSzU',
    ammTargetOrders: 'DKgXbNmsm1uCJ2eyh6xcnTe1G6YUav8RgzaxrbkG4xxe',
    // no need
    ammQuantities: NATIVE_SOL.mintAddress,
    poolCoinTokenAccount: '6XZ1hoJQZARtyA17mXkfnKSHWK2RvocC3UDNsY7f4Lf6',
    poolPcTokenAccount: 'F4opwQUoVhVRaf3CpMuCPpWNcB9k3AXvMMsfQh52pa66',
    poolWithdrawQueue: '8mqpqWGL7W2xh8B1s6XDZJsmPuo5zRedcM5sF55hhEKo',
    poolTempLpTokenAccount: '9ex6kCZsLR4ZbMCN4TcCuFzkw8YhiC9sdsJPavsrqCws',
    serumProgramId: SERUM_PROGRAM_ID_V3,
    serumMarket: 'G4LcexdCzzJUKZfqyVDQFzpkjhB1JoCNL8Kooxi9nJz5',
    serumBids: 'DVjhW8nLFWrpRwzaEi1fgJHJ5heMKddssrqE3AsGMCHp',
    serumAsks: 'CY2gjuWxUFGcgeCy3UiureS3kmjgDSRF59AQH6TENtfC',
    serumEventQueue: '8w4n3fcajhgN8TF74j42ehWvbVJnck5cewpjwhRQpyyc',
    serumCoinVaultAccount: '4ctYuY4ZvCVRvF22QDw8LzUis9yrnupoLQNXxmZy1BGm',
    serumPcVaultAccount: 'DovDds7NEzFn493DJ2yKBRgqsYgDXg6z38pUGXe1AAWQ',
    serumVaultSigner: 'BUDJ4F1ZknbZiwHb6xHEsH6o1LuW394DE8wKT8CoAYNF',
    official: true
  },
]