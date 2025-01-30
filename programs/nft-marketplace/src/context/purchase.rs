
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{associated_token::AssociatedToken, metadata::{MasterEditionAccount, Metadata, MetadataAccount}, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked, CloseAccount, close_account}};

use crate::state::{Listing, Marketplace};
use crate::error::MarketplaceError;

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_mint,
        associated_token::authority = taker
    )]
    pub taker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"reward", marketplace.key().as_ref()],
        bump = marketplace.reward_mint_bump,
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump = marketplace.treasury_bump,
    )]
    pub treasury: SystemAccount<'info>,
    pub maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"listing", marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = listing
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>
}	

impl<'info> Purchase<'info> {
    pub fn send_sol(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.taker.to_account_info(),
            to: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let fee = (self.marketplace.fee as u64).checked_mul(self.listing.price).unwrap().checked_div(10000_u64).unwrap();

        let amount = self.listing.price.checked_sub(fee).unwrap();

        transfer(cpi_ctx, amount);

        //send fee to treasury

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.taker.to_account_info(),
            to: self.treasury.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, fee);

        Ok(())
    }

    pub fn send_nft(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_ata.to_account_info(),
            mint: self.maker_mint.to_account_info(),
            authority: self.listing.to_account_info(),
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

        let cpi_account = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.listing.to_account_info()
        };
        
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"listing",
            self.marketplace.to_account_info().key.as_ref(),
            self.maker_mint.to_account_info().key.as_ref(),
            &[self.listing.bump]
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, &signer_seeds);

        close_account(cpi_ctx)?;

        Ok(())
    } 
} 

