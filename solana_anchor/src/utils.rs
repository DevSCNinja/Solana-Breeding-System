use {
    crate::PoolError,
    anchor_lang::{
        prelude::{AccountInfo, ProgramResult,},
        solana_program::{
            program::{invoke},
        },
    },
};

pub struct TokenMintToParams<'a> {
    pub mint : AccountInfo<'a>,
    pub account : AccountInfo<'a>,
    pub owner : AccountInfo<'a>,
    pub token_program : AccountInfo<'a>,
    pub amount : u64,
}

#[inline(always)]
pub fn spl_token_mint_to(params : TokenMintToParams<'_>) -> ProgramResult {
    let TokenMintToParams {
        mint,
        account,
        owner,
        token_program,
        amount,
    } = params;
    let result = invoke(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            account.key,
            owner.key,
            &[],
            amount,
        )?,
        &[mint,account,owner,token_program],
    );
    result.map_err(|_| PoolError::TokenMintToFailed.into())
}