import { Blob, seq, struct,Structure, u32, u8, u16, ns64 } from 'buffer-layout';
import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

export const NUM_TOKENS = 8
export const MAX_TOKENS = 50
export const NUM_MARGIN = 2
export const MAX_INVESTORS = 10
export const MAX_LIMIT_ORDERS= 2

class PublicKeyLayout extends Blob {
  constructor(property) {
    super(32, property);
  }

  decode(b, offset) {
    return new PublicKey(super.decode(b, offset));
  }

  encode(src, b, offset) {
    return super.encode(src.toBuffer(), b, offset);
  }
}

export function publicKeyLayout(property = "") {
  return new PublicKeyLayout(property);
}

class BNLayout extends Blob {
  constructor(number, property) {
    super(number, property);
    // restore prototype chain
    Object.setPrototypeOf(this, new.target.prototype)
  }

  decode(b, offset) {
    return new BN(super.decode(b, offset), 10, 'le');
  }

  encode(src, b, offset) {
    return super.encode(src.toArrayLike(Buffer, 'le', this['span']), b, offset);
  }
}

class U64F64Layout extends Blob {
  constructor(property) {
    super(16, property);
  }

  decode(b, offset) {
    const raw = new BN(super.decode(b, offset), 10, 'le');

    return raw / Math.pow(2, 64);
  }

  encode(src, b, offset) {
    console.log("src ::: ", src)
    return super.encode(src.toArrayLike(Buffer, 'le', this['span']), b, offset);
  }
}

export function U64F64(property = "") {
  return new U64F64Layout(property)
}

export function u64(property = "") {
  return new BNLayout(8, property);
}

export function u128(property = "") {
  return new BNLayout(16, property);
}

const zeroKey = new PublicKey(new Uint8Array(32));
export class MangoInfo {

  mango_account;
  perp_markets;
  deposit_index;
  markets_active;
  deposits_active;
  xpadding;
  investor_debts;
  padding;

  constructor(decoded) {
    Object.assign(this, decoded);
  }
  // isEmpty() {
  //   return this.mango_account.equals(zeroKey);
  // }
}

 
// export class MangoInfoLayout extends Structure {
//   constructor(property) {
//     super(
//       [
//         publicKeyLayout('mango_account'),
//         seq(u8('perp_markets'), 4),
//         u8('deposit_index'),
//         u8('markets_active'),
//         u8('deposits_active'),
//         u8('xpadding'),
//         seq(u64('investor_debts'), 2),
//         seq(u8('padding'), 24),
//       ],
//       property,
//     );
//   }
//   decode(b, offset) {
//     return new MangoInfo(super.decode(b, offset));
//   }
//   encode(src, b, offset) {
//     return super.encode(src.toBuffer(), b, offset);
//   }
// }
// export function mangoInfoLayout(property = '') {
//   return new MangoInfoLayout(property);
// }
 
export const PLATFORM_DATA = struct([
  u8('is_initialized'),
  u8('version'),
  u8('router_nonce'),
  u8('no_of_active_funds'),
  u8('token_count'),
  // seq(u8(), 3, 'padding'),

  u8('padding'),
  u16('total_v3_funds'),

  publicKeyLayout('router'),
  publicKeyLayout('investin_admin'),
  publicKeyLayout('investin_vault'),

  seq(
    struct([
      publicKeyLayout('mint'),
      u64('decimals'),
      publicKeyLayout('pool_coin_account'),
      publicKeyLayout('pool_pc_account'),
      U64F64('pool_price'),
      ns64('last_updated'),
      u8('token_id'),
      u8('pc_index'),
      seq(u8(), 6, 'padding')
    ]),
    MAX_TOKENS, 'token_list'
  ),

])

export const FUND_DATA = struct([
  u8('is_initialized'),
  u8('number_of_active_investments'),
  u8('no_of_investments'),
  u8('signer_nonce'),
  u8('no_of_margin_positions'),
  u8('no_of_assets'),
  u16('position_count'),

  u8('version'),
  u8('is_private'),
  u16('fund_v3_index'),
  seq(u8(), 4, 'padding'),

  u64('min_amount'),
  U64F64('min_return'),
  U64F64('performance_fee_percentage'),
  U64F64('total_amount'),
  U64F64('prev_performance'),

  u64('amount_in_router'),
  U64F64('performance_fee'),
  publicKeyLayout('manager_account'),
  publicKeyLayout('fund_pda'),
  seq(
    struct([
      u8('is_active'),
      seq(u8(),3,'index'),
      u8('mux'),
      u8('is_on_mango'),
      seq(u8(), 2, 'padding'),
      u64('balance'),
      u64('debt'),
      publicKeyLayout('vault')
    ]),
    NUM_TOKENS, 'tokens'
  ),
  seq(publicKeyLayout(), MAX_INVESTORS, 'investors'),
  
  struct([
      publicKeyLayout('mango_account'),
      seq(u8(),4,'perp_markets'),
      u8('deposit_index'),
      u8('markets_active'),
      u8('deposits_active'),
      u8('xpadding'),
      seq(u64(), 2, 'investor_debts'),
      seq(u8('padding'), 24),
    ],'mango_positions'),

  // mangoInfoLayout('mango_positions'),
  
     
  seq(u8(), 80, 'margin_update_padding'),
  seq(u8(), 32, 'padding'),

])

export const INVESTOR_DATA = struct([
  u8('is_initialized'),
  u8('has_withdrawn'),
  u8('withdrawn_from_margin'),
  seq(u8('padding'), 5),


  publicKeyLayout('owner'),
  u64('amount'),
  U64F64('start_performance'),
  u64('amount_in_router'),
  publicKeyLayout('manager'),
  seq(U64F64(), NUM_MARGIN, 'margin_debt'),
  seq(u64(), NUM_MARGIN, 'margin_position_id'),

  seq(u8(), NUM_TOKENS, 'token_indexes'),
  seq(u64(), NUM_TOKENS, 'token_debts'),

  seq(u8(), 32, 'xpadding')

])

export const SPL_TOKEN_MINT_DATA = struct([
  publicKeyLayout('mint_authority'),
  u64('supply'),
  u8('decimals'),
  u8('is_initialized'),
  publicKeyLayout('freez_authority'),
])



export const AMM_INFO_LAYOUT_V4 = struct([
  u64('status'),
  u64('nonce'),
  u64('orderNum'),
  u64('depth'),
  u64('coinDecimals'),
  u64('pcDecimals'),
  u64('state'),
  u64('resetFlag'),
  u64('minSize'),
  u64('volMaxCutRatio'),
  u64('amountWaveRatio'),
  u64('coinLotSize'),
  u64('pcLotSize'),
  u64('minPriceMultiplier'),
  u64('maxPriceMultiplier'),
  u64('systemDecimalsValue'),
  // Fees
  u64('minSeparateNumerator'),
  u64('minSeparateDenominator'),
  u64('tradeFeeNumerator'),
  u64('tradeFeeDenominator'),
  u64('pnlNumerator'),
  u64('pnlDenominator'),
  u64('swapFeeNumerator'),
  u64('swapFeeDenominator'),
  // OutPutData
  u64('needTakePnlCoin'),
  u64('needTakePnlPc'),
  u64('totalPnlPc'),
  u64('totalPnlCoin'),
  u128('poolTotalDepositPc'),
  u128('poolTotalDepositCoin'),
  u128('swapCoinInAmount'),
  u128('swapPcOutAmount'),
  u64('swapCoin2PcFee'),
  u128('swapPcInAmount'),
  u128('swapCoinOutAmount'),
  u64('swapPc2CoinFee'),

  publicKeyLayout('poolCoinTokenAccount'),
  publicKeyLayout('poolPcTokenAccount'),
  publicKeyLayout('coinMintAddress'),
  publicKeyLayout('pcMintAddress'),
  publicKeyLayout('lpMintAddress'),
  publicKeyLayout('ammOpenOrders'),
  publicKeyLayout('serumMarket'),
  publicKeyLayout('serumProgramId'),
  publicKeyLayout('ammTargetOrders'),
  publicKeyLayout('poolWithdrawQueue'),
  publicKeyLayout('poolTempLpTokenAccount'),
  publicKeyLayout('ammOwner'),
  publicKeyLayout('pnlOwner')
])


// Aggregator Accounts
export const PRICE_DATA = struct([
  u32('count'),
  u32('decimals'),
  seq(
    struct([
      publicKeyLayout('token_mint'),
      publicKeyLayout('pool_account'),
      publicKeyLayout('base_pool_account'),
      u64('decimals'),
      u64('token_price'),
      ns64('last_updated'),
    ]),
    MAX_TOKENS, 'prices'
  ),
])

export const FRIKTION_VOLT = struct(
  [
    u64('discrim'),
    publicKeyLayout('adminKey'),
    publicKeyLayout('seed'),
    u64('transferWindow'),
    u64('startTransferTime'),
    u64('endTransferTime'),
    u8('initialized'),
    u8('currOptionWasSettled'),
    u8('mustSwapPremiumToUnderlying'),
    u8('nextOptionWasSet'),
    u8('firstEverOptionWasSet'),
    u8('instantTransfersEnabled'),
    u8('prepareIsFinished'),
    u8('enterIsFinished'),
    u8('roundHasStarted'),
    u64('roundNumber'),
    u64('totalUnderlyingPreEnter'),
    u64('totalUnderlyingPostSettle'),
    u64('totalVoltTokensPostSettle'),
    publicKeyLayout('vaultAuthority'),
    publicKeyLayout('depositPool'),
    publicKeyLayout('premiumPool'),
    publicKeyLayout('optionPool'),
    publicKeyLayout('writerTokenPool'),
    publicKeyLayout('vaultMint'),
    publicKeyLayout('underlyingAssetMint'),
    publicKeyLayout('quoteAssetMint'),
    publicKeyLayout('optionMint'),
    publicKeyLayout('writerTokenMint'),
    publicKeyLayout('optionMarket'),
    u64('vaultType'),
    u64('underlyingAmountPerContract'),
    u64('quoteAmountPerContract'),
    ns64('expirationUnixTimestamp'),
    u64('expirationInterval'),
    u64('upperBoundOtmStrikeFactor'),
    u8('haveTakenWithdrawalFees'),
    publicKeyLayout('serumSpotMarket'),
    u8('openOrdersBump'),
    u8('openOrdersInitBump'),
    u8('ulOpenOrdersBump'),
    publicKeyLayout('ulOpenOrders'),
    u8('ulOpenOrdersInitialized'),
    u8('bumpAuthority'),
    u64('serumOrderSizeOptions'),
    u64('individualCapacity'),
    u64('serumOrderType'),
    u16('serumLimit'),
    u16('serumSelfTradeBehavior'),
    u64('serumClientOrderId'),
    publicKeyLayout('whitelistTokenMint'),
    publicKeyLayout('permissionedMarketPremiumMint'),
    publicKeyLayout('permissionedMarketPremiumPool'),
    u64('capacity')
  ]
)



export const FUND_PDA_DATA = struct([
  u8('is_initialized'),
  u8('number_of_active_investments'),
  u8('no_of_investments'),
  u8('signer_nonce'),
  u8('no_of_margin_positions'),
  u8('no_of_assets'),
  u16('position_count'),

  u8('version'),
  u8('is_private'),
  u16('fund_v3_index'),
  seq(u8(), 4, 'padding'),

  u64('min_amount'),
  U64F64('min_return'),
  U64F64('performance_fee_percentage'),
  U64F64('total_amount'),
  U64F64('prev_performance'),

  u64('amount_in_router'),
  U64F64('performance_fee'),
  publicKeyLayout('manager_account'),
  publicKeyLayout('fund_pda'),
  seq(
    struct([
      u8('is_active'),
      seq(u8(),3,'index'),
      u8('mux'),
      u8('is_on_mango'),
      seq(u8(), 2, 'padding'),
      u64('balance'),
      u64('debt'),
      publicKeyLayout('vault')
    ]),
    NUM_TOKENS, 'tokens'
  ),
  seq(publicKeyLayout(), MAX_INVESTORS, 'investors'),
  
  struct([
      publicKeyLayout('mango_account'),
      seq(u8(),3,'perp_markets'),
      u8('padding'),
      u8('deposit_index'),
      u8('markets_active'),
      u8('deposits_active'),
      u8('xpadding'),
      seq(u64(), 2, 'investor_debts'),
      seq(u8('padding'), 24),
    ],'mango_positions'),

  
  struct([
      u8('is_active'),
      u8('is_split'),
      u8('hop'),
      u8('count'),
      u8('token_in_slot'),
      u8('token_out_slot'),
      seq(u8('padding'), 2),
   
      publicKeyLayout('token_in'),
      publicKeyLayout('token_out'),
      u64('amount_in'),
      u64('min_amount_out'),
  ],'guard'),

  seq(
    struct([
      u64('price'),
      u64('max_base_quantity'),
      u64('max_quote_quantity'),
      u64('client_order_id'),
      u64('expiry_timestamp'),
      u8('is_repost_processing'),
      u8('perp_market_id'),
      u8('side'),
      u8('reduce_only'),
      u8('limit'),
      seq(u8(),3,'padding'),
    ]),
    MAX_LIMIT_ORDERS, 'limit_orders'
  ),
  struct([
    u64('last_updated'),
    publicKeyLayout('volt_vault_id'),
    u64('total_value_in_ul'),
    u64('fc_token_balance'),
    u64('ul_token_balance'),
    u64('fc_token_debt'),
    u64('ul_token_debt'),
    u8('ul_token_slot'),
    u8('is_active'),
    seq(u8(),6,'padding'),
  ], 'friktion_vault'),
  
  seq(u8(), 1864, 'migration_additonal_padding'),
])