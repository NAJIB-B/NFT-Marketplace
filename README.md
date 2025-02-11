# NFT Marketplace

An **NFT marketplace** on the **Solana blockchain** using the **Anchor framework**, enabling secure and efficient minting, buying, and selling of NFTs.  

## Overview  
This example demonstrates how to create an **NFT marketplace** on Solana using the **Anchor framework**. It enables users to:  
- **List NFTs** for sale.  
- **Buy NFTs** securely and efficiently.  
- **Sell NFTs** with trustless transactions.
- **Delist NFTs** with ease.

## Let's walk through the architecture:

For this program, we will have a marketplace account:

```rust
#[account]
pub struct Marketplace {
    pub admin: Pubkey,
    pub fee: u16,
    pub bump: u8,
    pub treasury_bump: u8,
    pub reward_mint_bump: u8,
    pub name: String,
}

impl Space for Marketplace {
    const INIT_SPACE: usize = 8 + 32 + 2 + 1 + 1 + 1 + (4 + 32);
}
```

The marketplace account will hold the following data:


- `admin`: The account that manages the marketplace.
- `fee`: The transaction fee percentage collected by the marketplace.
- `bump`: The bump seed for the marketplace's Program Derived Address (PDA).
- `treasury_bump`: The bump seed for the treasury account.
- `reward_mint_bump`: The bump seed for the reward mint account.
- `name`: The name of the marketplace.

---

## The user will be able to initialize the marketplace. For that, we create the following context:

  ```rust
#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump,
        space = Marketplace::INIT_SPACE
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump
    )]
    pub treasury: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [b"reward", marketplace.key().as_ref()],
        bump,
        mint::authority = marketplace,
        mint::decimals = 6
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>
}	

```
Let's have a closer look at the accounts that we are passing in this context:

- `admin`: The account initializing the marketplace.  
- `marketplace`: The marketplace account that stores marketplace-related data.  
- `treasury`: The system account that will hold collected fees.  
- `reward_mint`: The mint account for marketplace reward tokens.  
- `token_program`: The token program used for token-related operations.  
- `system_program`: The system program used for account initialization and management.  


## We then implement the initialization function:

```rust
impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, name: String, fee: u16, bumps: &InitializeBumps) -> Result<()> {

            require!(name.len() > 0 && name.len() < 4 + 32, MarketplaceError::NameTooLong);

        self.marketplace.set_inner(Marketplace{
            admin: self.admin.key(),
            fee,
            bump: bumps.marketplace,
            treasury_bump: bumps.treasury,
            reward_mint_bump: bumps.reward_mint,
            name
        });
        Ok(())
    }
} 
```
## Now for the listing
For this program, we will have a listing account:

```rust
#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub maker: Pubkey,
    pub mint: Pubkey,
    pub price: u64,
    pub bump: u8,
}
```

The listing account will hold the following data:  

- `maker`: The account that created the listing.  
- `mint`: The mint address of the listed NFT.  
- `price`: The listing price of the NFT in lamports.  
- `bump`: The bump seed for the listing account’s Program Derived Address (PDA).


---

## Listing an NFT on the Marketplace  
In the `List` function, we initialize a new listing on the marketplace by transferring the NFT from the maker's associated token account to a vault account. The listing account is created to store the listing details, ensuring that only verified NFTs from a specified collection can be listed.  

---

## The `List` Context 

```rust
pub struct List<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        seeds = [b"reward", marketplace.key().as_ref()],
        bump = marketplace.reward_mint_bump,
    )]
    pub reward_mint: InterfaceAccount<'info, Mint>,
    pub maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = maker
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = maker_mint,
        associated_token::authority = listing
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        seeds = [b"listing", marketplace.key().as_ref(), maker_mint.key().as_ref()],
        bump,
        space = 8 + Listing::INIT_SPACE
    )]
    pub listing: Account<'info, Listing>,
    pub collection_mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [
        b"metadata",
        metadata_program.key().as_ref(),
        maker_mint.key().as_ref()
        ],
        seeds::program = metadata_program.key(),
        bump,
        constraint = metadata.collection.as_ref().unwrap().key.as_ref() == collection_mint.key().as_ref(),
        constraint = metadata.collection.as_ref().unwrap().verified == true

    )]
    pub metadata: Account<'info, MetadataAccount>,
    #[account(
        seeds = [
        b"metadata",
        metadata_program.key().as_ref(),
        maker_mint.key().as_ref(),
        b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump
    )]
         #[account(constraint = master_edition.supply > 0)]
    pub master_edition: Account<'info, MasterEditionAccount>,
    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>
}	
```
In this context, we are passing all the accounts needed to list an NFT on the marketplace:  

- `maker`: The account listing the NFT for sale.  
- `marketplace`: The marketplace account managing the listing.  
- `reward_mint`: The mint account for marketplace reward tokens.  
- `maker_mint`: The mint account of the NFT being listed.  
- `maker_ata`: The associated token account of the maker for the listed NFT.  
- `vault`: The token account that will hold the listed NFT during the sale.  
- `listing`: The listing account that stores the NFT's price and ownership details.  
- `collection_mint`: The mint address of the NFT's collection.  
- `metadata`: The metadata account verifying that the NFT belongs to the specified collection.  
- `master_edition`: The master edition account ensuring that the NFT is not a non-transferable edition.  
- `metadata_program`: The program managing metadata operations.  
- `associated_token_program`: The program handling associated token accounts.  
- `token_program`: The token program used for token transactions.  
- `system_program`: The system program used for account initialization and management.  


## We then implemented the functionalities for listing:

```rust 
impl<'info> List<'info> {
    pub fn create_listing(&mut self, price: u64,  bump: &ListBumps) -> Result<()> {

        self.listing.set_inner(Listing{
            maker: self.maker.key(),
            mint: self.maker_mint.key(),
            bump: bump.listing,
            price
        });

        Ok(())
    }

    pub fn deposit_nft(&mut self) -> Result<()> {

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.maker_ata.to_account_info(),
            to: self.vault.to_account_info(),
            mint: self.maker_mint.to_account_info(),
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(cpi_ctx, 1, self.maker_mint.decimals)?;


        Ok(())
    }
} 
```
## Purchasing an NFT from the Marketplace 

In the `Purchase` function, a buyer (taker) acquires an NFT from the marketplace by transferring the required payment to the seller (maker). The NFT is moved from the escrow vault to the buyer’s associated token account, while the marketplace collects a transaction fee.  

---

### The `Purchase` Context  

```rust
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
```

In this context, we are passing all the accounts needed to purchase an NFT from the marketplace:  

- **taker**: The account purchasing the NFT.  
- **maker**: The seller who listed the NFT for sale.  
- **marketplace**: The marketplace account managing the transaction.  
- **taker_ata**: The associated token account of the buyer where the NFT will be transferred.  
- **reward_mint**: The mint account for marketplace reward tokens.  
- **treasury**: The treasury account collecting transaction fees from the purchase.  
- **maker_mint**: The mint account of the NFT being purchased.  
- **listing**: The listing account storing the NFT sale details.  
- **vault**: The token account holding the NFT in escrow until purchased.  
- **associated_token_program**: The program handling associated token accounts.  
- **token_program**: The token program used for token transactions.  
- **system_program**: The system program used for payments and account management.  

## We then implement the functionality for our Purchase context:

```rust
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

```
## Delisting an NFT from the Marketplace 

## The `Delist` Context  

The `Delist` context defines the necessary accounts for removing an NFT listing from the marketplace and returning the NFT to the maker.
```rust
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
```

In this context, we are passing all the accounts needed to delist an NFT from the marketplace:  

- **maker**: The account that initially listed the NFT and is now removing it from the marketplace.  
- **marketplace**: The marketplace account that managed the listing.  
- **maker_ata**: The associated token account where the delisted NFT will be transferred back.  
- **maker_mint**: The mint account of the NFT being delisted.  
- **listing**: The listing account that stored the NFT’s sale details, now being closed.  
- **vault**: The token account that held the NFT while it was listed, returning the NFT to the maker.  
- **associated_token_program**: The program handling associated token accounts.  
- **token_program**: The token program used for token transfers.  
- **system_program**: The system program used for account closure and management.

## We then implemented the functionality for delisting:

```rust
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


```  
