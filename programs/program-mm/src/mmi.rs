pub fn mango_init_margin_account (
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    // min_amount: u64,
    // min_return: u64,
    // performance_fee_percentage: u64,
    // perp_market_index: u8
) -> Result<(), ProgramError> {

    const NUM_FIXED:usize = 8;
    let accounts = array_ref![accounts, 0, NUM_FIXED];

    let [
        fund_state_ai,
        manager_ai,
        fund_pda_ai,
        fund_vault_ai,
        fund_mngo_vault_ai,
        mango_group_ai,
        mango_account_ai,
        mango_prog_ai,
    ] = accounts;

    let mut fund_data = FundData::load_mut_checked(fund_state_ai, program_id)?;

    // check for manager's signature
    check!(manager_ai.is_signer, ProgramError::MissingRequiredSignature);
    // save manager's wallet address
    fund_data.manager_account = *manager_ai.key;

    // get nonce for signing later
    let (pda, nonce) = Pubkey::find_program_address(&[&*manager_ai.key.as_ref()], program_id);
    fund_data.fund_pda = pda;
    fund_data.signer_nonce = nonce;

    // check for ownership of vault
    let fund_vault = parse_token_account(fund_vault_ai)?;
    let fund_mngo_vault = parse_token_account(fund_mngo_vault_ai)?;

    check_eq!(fund_vault.owner, fund_data.fund_pda);
    check_eq!(fund_mngo_vault.owner, fund_data.fund_pda);
    check_eq!(&fund_mngo_vault.mint, &mngo_token::ID); // check for mngo mint

    fund_data.vault_key = *fund_vault_ai.key;
    fund_data.mngo_vault_key = *fund_mngo_vault_ai.key;
    fund_data.vault_balance = 0;

    // Init Mango account for the fund
    invoke_signed(
        &init_mango_account(mango_prog_ai.key, mango_group_ai.key, mango_account_ai.key, fund_pda_ai.key)?,
        &[
            mango_prog_ai.clone(),
            mango_group_ai.clone(),
            mango_account_ai.clone(),
            fund_pda_ai.clone(),
        ],
        &[&[fund_data.manager_account.as_ref(), bytes_of(&fund_data.signer_nonce)]]
    )?;
    fund_data.mango_account = *mango_account_ai.key;

    Ok(())
}