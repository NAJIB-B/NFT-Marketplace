

use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{associated_token::AssociatedToken, metadata::{MasterEditionAccount, Metadata, MetadataAccount}, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked, CloseAccount, close_account}};

use crate::state::{Listing, Marketplace};
use crate::error::MarketplaceError;

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
   pub maker: Signer<'info>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = maker_mint,
        associated_token::authority = maker
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,
    pub maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        close = maker,
        seeds = [b"listing", marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        associated_token::mint = maker_mint,
        associated_token::authority = listing
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>
}	

impl<'info> Delist<'info> {
    pub fn delist(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_ata.to_account_info(),
            mint: self.maker_mint.to_account_info(),
            authority: self.listing.to_account_info()
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"listing",
            self.marketplace.to_account_info().key.as_ref(),
            self.maker_mint.to_account_info().key.as_ref(),
            &[self.listing.bump]
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        transfer_checked(cpi_ctx, 1, self.maker_mint.decimals)?;
        
        Ok(())
    }

    pub fn close_vault(&mut self) -> Result<()> {

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.listing.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"listing",
            self.marketplace.to_account_info().key.as_ref(),
            self.maker_mint.to_account_info().key.as_ref(),
            &[self.listing.bump]
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        close_account(cpi_ctx)?;
        Ok(())
    }
} 

