pub mod utils;
use borsh::{BorshDeserialize,BorshSerialize};
use {
    crate::utils::*,
    anchor_lang::{
        prelude::*,
        AnchorDeserialize,
        AnchorSerialize,
        Key,
        solana_program::{
            program_pack::Pack,
            sysvar::{clock::Clock},
            msg,
            program::{invoke},
        }      
    },
    spl_token::state,
    metaplex_token_metadata::{
        instruction::{
            create_metadata_accounts,
            create_master_edition,
            update_metadata_accounts,
        },
    }
};
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_anchor {
    use super::*;

    pub fn init_pool(
        ctx : Context<InitPool>,
        _bump : u8,
        _mythic_period : i64,
        _baby_pheonix_period : i64,
        _mythic_pheonix_period : i64,
        _mythic_amount : u64,
        _baby_pheonix_amount : u64,
        _mythic_pheonix_amount : u64,
        ) -> ProgramResult {
        let pool = &mut ctx.accounts.pool;
        
        pool.owner = *ctx.accounts.owner.key;
        pool.mythic_period = _mythic_period;
        pool.baby_pheonix_period = _baby_pheonix_period;
        pool.mythic_pheonix_period = _mythic_pheonix_period;
        pool.mythic_amount = _mythic_amount;
        pool.baby_pheonix_amount = _baby_pheonix_amount;
        pool.mythic_pheonix_amount = _mythic_pheonix_amount;
        pool.bump = _bump;
        pool.rand = *ctx.accounts.rand.key;
        Ok(())
    }

    pub fn init_wallet(
        ctx : Context<InitWallet>,
        _evolving_amount: u16,
        _exalated_breeding_amount: u8,
        ) -> ProgramResult {
        let pool = &mut ctx.accounts.pool;
        pool.wallet = *ctx.accounts.owner.key;
        pool.evolving_amount = _evolving_amount;
        pool.exalated_breeding_amount = _exalated_breeding_amount;
        Ok(())
    }

    pub fn breed(
        ctx : Context<Breed>,
        _b_type: u8,
        _data : Metadata,
        ) -> ProgramResult {
        let pool = &ctx.accounts.pool;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        let breed_data = &mut ctx.accounts.breed_data;
        let wallet_data = &mut ctx.accounts.wallet_data;

        breed_data.b_time = clock.unix_timestamp;
        breed_data.b_type = _b_type;
        breed_data.mint =  *ctx.accounts.mint.key;

        if _b_type == 1 {
            wallet_data.evolving_amount += 1;
        }

        if _b_type == 3 {
            wallet_data.exalated_breeding_amount += 1;
        }

        let mint : state::Mint = state::Mint::unpack_from_slice(&ctx.accounts.mint.data.borrow())?;
        if mint.decimals != 0 {
            return Err(PoolError::InvalidMintAccount.into());
        }
        if mint.supply != 0 {
            return Err(PoolError::InvalidMintAccount.into());
        }
        spl_token_mint_to(
            TokenMintToParams{
                mint : ctx.accounts.mint.clone(),
                account : ctx.accounts.token_account.clone(),
                owner : ctx.accounts.owner.clone(),
                token_program : ctx.accounts.token_program.clone(),
                amount : 1 as u64,
            }
        )?;
        let mut creators : Vec<metaplex_token_metadata::state::Creator> = 
            vec![metaplex_token_metadata::state::Creator{
                address: pool.key(),
                verified : true,
                share : 0,
            }];
        creators.pop();
        for c in _data.creators {

            creators.push(metaplex_token_metadata::state::Creator{
                address : c.address,
                verified : false,
                share : c.share,
            });
        }

        invoke(
            &create_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.mint.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                _data.name,
                _data.symbol,
                _data.uri,
                Some(creators),
                _data.seller_fee_basis_points,
                true,
                _data.is_mutable,
            ),
            &[
                ctx.accounts.metadata.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ]
        )?;

        invoke(
            &create_master_edition(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.master_edition.key,
                *ctx.accounts.mint.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.owner.key,
                None,
            ),
            &[
                ctx.accounts.master_edition.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ]
        )?;

        invoke(
            &update_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.owner.key,
                None,
                None,
                Some(true),
            ),
            &[
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.owner.clone(),                
            ]
        )?;
        Ok(())
    }

    pub fn claim(
        ctx : Context<Claim>,
        _data : Metadata,
        ) -> ProgramResult {
        let pool = &ctx.accounts.pool;
        let breed_data = &mut ctx.accounts.breed_data;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;

        if breed_data.mint != ctx.accounts.breed_mint.key() {
            return Err(PoolError::InvalidMintAccount.into());
        }
        if breed_data.b_type == 1 && clock.unix_timestamp - breed_data.b_time < pool.mythic_period {
            msg!("Can not Claim.");
            return Err(PoolError::InvalidTime.into());
        }

        if breed_data.b_type == 2 && clock.unix_timestamp - breed_data.b_time < pool.baby_pheonix_period {
            msg!("Can not Claim.");
            return Err(PoolError::InvalidTime.into());
        }
        
        if breed_data.b_type == 3 && clock.unix_timestamp - breed_data.b_time < pool.mythic_pheonix_period {
            msg!("Can not Claim.");
            return Err(PoolError::InvalidTime.into());
        }

        let mint : state::Mint = state::Mint::unpack_from_slice(&ctx.accounts.mint.data.borrow())?;
        if mint.decimals != 0 {
            return Err(PoolError::InvalidMintAccount.into());
        }
        if mint.supply != 0 {
            return Err(PoolError::InvalidMintAccount.into());
        }

        spl_token_mint_to(
            TokenMintToParams{
                mint : ctx.accounts.mint.clone(),
                account : ctx.accounts.token_account.clone(),
                owner : ctx.accounts.owner.clone(),
                token_program : ctx.accounts.token_program.clone(),
                amount : 1 as u64,
            }
        )?;

        let mut creators : Vec<metaplex_token_metadata::state::Creator> = 
            vec![metaplex_token_metadata::state::Creator{
                address: pool.key(),
                verified : true,
                share : 0,
            }];
        creators.pop();
        for c in _data.creators {

            creators.push(metaplex_token_metadata::state::Creator{
                address : c.address,
                verified : false,
                share : c.share,
            });
        }

        invoke(
            &create_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.mint.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                _data.name,
                _data.symbol,
                _data.uri,
                Some(creators),
                _data.seller_fee_basis_points,
                true,
                _data.is_mutable,
            ),
            &[
                ctx.accounts.metadata.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ]
        )?;

        invoke(
            &create_master_edition(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.master_edition.key,
                *ctx.accounts.mint.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.owner.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.owner.key,
                None,
            ),
            &[
                ctx.accounts.master_edition.clone(),
                ctx.accounts.mint.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.owner.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.token_program.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ]
        )?;

        invoke(
            &update_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.owner.key,
                None,
                None,
                Some(true),
            ),
            &[
                ctx.accounts.token_metadata_program.clone(),
                ctx.accounts.metadata.clone(),
                ctx.accounts.owner.clone(),                
            ]
        )?;

        breed_data.b_time = 0;
        breed_data.b_type = 0;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,   

    pool : ProgramAccount<'info, Pool>,

    #[account(mut)]
    breed_data : ProgramAccount<'info, BreedData>,

    #[account(address=spl_token::id())]
    token_program : AccountInfo<'info>,
    
    #[account(mut,owner=spl_token::id())]
    mint : AccountInfo<'info>,
    
    #[account(mut,owner=spl_token::id())]
    breed_mint : AccountInfo<'info>,

    #[account(mut,owner=spl_token::id())]
    token_account : AccountInfo<'info>,

    #[account(mut)]
    metadata : AccountInfo<'info>,


    #[account(mut)]
    master_edition : AccountInfo<'info>,

    #[account(address=metaplex_token_metadata::id())]
    token_metadata_program : AccountInfo<'info>,
    
    system_program : Program<'info,System>,

    rent : Sysvar<'info,Rent>,
    clock : AccountInfo<'info>,     
}

#[derive(Accounts)]
pub struct Breed<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>, 

    pool : ProgramAccount<'info,Pool>,

    #[account(init, payer=owner, space=8+BREED_DATA_SIZE)]
    breed_data : ProgramAccount<'info, BreedData>,
   
    #[account(mut)]
    wallet_data : ProgramAccount<'info, WalletData>,

    #[account(mut)]
    metadata : AccountInfo<'info>,

    #[account(mut,owner=spl_token::id())]
    mint : AccountInfo<'info>,

    #[account(mut,owner=spl_token::id())]
    token_account : AccountInfo<'info>,

    #[account(mut)]
    master_edition : AccountInfo<'info>,

    #[account(address=metaplex_token_metadata::id())]
    token_metadata_program : AccountInfo<'info>,

    #[account(address=spl_token::id())]
    token_program : AccountInfo<'info>,

    system_program : Program<'info,System>,

    clock : AccountInfo<'info>,    
    rent : Sysvar<'info,Rent>,

}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitPool<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(init, seeds=[(*rand.key).as_ref()], bump=_bump, payer=owner, space=8+POOL_SIZE)]
    pool : ProgramAccount<'info, Pool>,

    rand : AccountInfo<'info>,

    system_program : Program<'info,System>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitWallet<'info> {
    #[account(mut, signer)]
    owner : AccountInfo<'info>,

    #[account(init, payer=owner, space=8+WALLET_DATA_SIZE)]
    pool : ProgramAccount<'info, WalletData>,

    system_program : Program<'info,System>,
}

pub const POOL_SIZE : usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
pub const MYTHIC_PERIOD : i64 = 16 * 24 * 60 * 60;
pub const BABY_MYTHIC_PERIOD : i64 = 30 * 24 * 60 * 60;
pub const MYTHIC_PHEONIX_PERIOD : i64 = 2 * 24 * 60 * 60;

#[account]
pub struct Pool {
    pub owner : Pubkey,
    pub rand : Pubkey,
    pub mythic_period : i64,
    pub baby_pheonix_period : i64,
    pub mythic_pheonix_period : i64,
    pub mythic_amount : u64,
    pub baby_pheonix_amount : u64,
    pub mythic_pheonix_amount : u64,
    pub bump : u8,
}

#[derive(AnchorSerialize,AnchorDeserialize,Clone)]
pub struct Creator {
    pub address : Pubkey,
    pub verified : bool,
    pub share : u8,
}

#[derive(AnchorSerialize,AnchorDeserialize,Clone,Default)]
pub struct Metadata{
    pub name : String,
    pub symbol : String,
    pub uri : String,
    pub seller_fee_basis_points : u16,
    pub creators : Vec<Creator>,
    pub is_mutable : bool,
}

pub const BREED_DATA_SIZE : usize = 8 + 1 + 32;

#[account]
pub struct BreedData {
    pub b_time : i64,
    pub b_type: u8,
    pub mint: Pubkey,
}

pub const WALLET_DATA_SIZE : usize = 32 + 2 + 1;

#[account]
pub struct WalletData {
    pub wallet: Pubkey,
    pub evolving_amount: u16,
    pub exalated_breeding_amount: u8,
}

#[error]
pub enum PoolError {
    #[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid metadata")]
    InvalidMetadata,

    #[msg("Invalid Breeddata account")]
    InvalidBreedData,

    #[msg("Invalid time")]
    InvalidTime,

    #[msg("Invalid Period")]
    InvalidPeriod,

    #[msg("Already unBreedd")]
    AlreadyUnBreedd,

    #[msg("Not allowed unBreed")]
    NotAllowed,

    #[msg("Token Burn Failed")]
    TokenBurnFailed,

    #[msg("Invalid mint account")]
    InvalidMintAccount,

    #[msg("Exeed amount")]
    ExceedAmount,
}