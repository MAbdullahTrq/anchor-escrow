use anchor_lang::prelude::*;
use anchor_spl::token::{ self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer };
use spl_token::instruction::AuthorityType;

declare_id!("4MFq2RjxeKARmiAWYZ3zCyuvqjCwg1HatNyqDmPgS83g");

pub mod exchange_tokens;
pub mod cancel;

#[program]
// Escrow program logic - Will be initialized by the buyer
// Escrow program will escrow the buyer's funds till the seller transfers the Token
pub mod escrow_tokens {
  use super::*;
  // Initialize escrow account
  const ESCROW_PA_SEED: &[u8] = b"escrow";
// Initializing the Room escrows
// Creating and initializing the both types of Token Accounts
  pub fn initialize(
    ctx: Context<Initialize>,
    _v_account_bump: u8,
    i_amount: u64,
    t_amount: u64
  ) -> Result<()> {
    ctx.accounts.escrow_account.initializer_key = *ctx.accounts.initializer.key;
    ctx.accounts.escrow_account.initializer_deposit_token_account =
      *ctx.accounts.initializer_deposit_token_account.to_account_info().key;
    ctx.accounts.escrow_account.initializer_receive_token_account =
      *ctx.accounts.initializer_receive_token_account.to_account_info().key;
    ctx.accounts.escrow_account.i_amount = i_amount;
    ctx.accounts.escrow_account.t_amount = t_amount;
// Checking the room admin
    let (room_authority, _room_authority_bump) = Pubkey::find_program_address(
      &[ESCROW_PA_SEED],
      ctx.program_id
    );
    // CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    token::set_authority(
      ctx.accounts.into_set_authority_context(),
      AuthorityType::AccountOwner,
      Some(room_authority)
    )?;
    // Transfer tokens from initializer to escrow account
    token::transfer(
      ctx.accounts.into_transfer_to_pda_context(),
      ctx.accounts.escrow_account.i_amount
    )?;

    Ok(())
  }
// Struct for account
  #[account]
  pub struct EscrowAccount {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub i_amount: u64,
    pub t_amount: u64,
  }

  #[derive(Accounts)]
  #[instruction(room_account_bump:  u8,  i_amount:  u64)]
  pub struct Initialize<'info> {
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    #[account(mut,  signer)]
    pub initializer: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
      init,
      seeds = [b"token-seed".as_ref()],
      bump,
      payer = initializer,
      token::mint = mint,
      token::authority = initializer
    )]
    pub room_account: Account<'info, TokenAccount>,
    #[account(
                mut,
                constraint  =  initializer_deposit_token_account.amount  >=  i_amount
        )]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    pub initializer_receive_token_account: Account<'info, TokenAccount>,
    #[account(zero)]
    pub escrow_account: Box<Account<'info, EscrowAccount>>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    ///  CHECK:  This  is  not  dangerous  because  we  don't  read  or  write  from  this  account
    pub token_program: AccountInfo<'info>,
  }
//   implimentation of the Initialize struct
// would be used to initialize the room accounts
  impl<'info> Initialize<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
        from: self.initializer_deposit_token_account.to_account_info().clone(),
        to: self.room_account.to_account_info().clone(),
        authority: self.initializer.clone(),
      };
      CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
      let cpi_accounts = SetAuthority {
        account_or_mint: self.room_account.to_account_info().clone(),
        current_authority: self.initializer.clone(),
      };
      CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
  }
}
