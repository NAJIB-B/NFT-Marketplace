use anchor_lang::prelude::*;
pub mod state;
pub mod context;
pub mod error;


declare_id!("9PWnquDC7xCBT9mTcQihKowi9NWy5Rsod8QCPEQN5X3A");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
