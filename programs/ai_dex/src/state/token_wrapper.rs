use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct TokenWrapper {
    pub ai_dex_config: Pubkey, // 32
    pub token_mint: Pubkey, // 32
    // 128 RESERVE
}

/// Struct representing a token wrapper.
///
/// The `TokenWrapper` struct holds information about the AI Dex configuration and the token mint.
/// It also provides a method to initialize the struct with the given AI Dex configuration and token mint.
impl TokenWrapper {
    /// Length of the `TokenWrapper` struct in bytes.
    pub const LEN: usize = 8 + 32 + 32 + 128;

    /// Initializes the `TokenWrapper` struct with the given AI Dex configuration and token mint.
    ///
    /// # Arguments
    ///
    /// * `ai_dex_config` - The AI Dex configuration pubkey.
    /// * `token_mint` - The token mint pubkey.
    ///
    /// # Errors
    ///
    /// Returns an error if the initialization fails.
    pub fn initialize(
        &mut self,
        ai_dex_config: Pubkey,
        token_mint: Pubkey,
    ) -> Result<()> {
        self.ai_dex_config = ai_dex_config;
        self.token_mint = token_mint;
        Ok(())
    }
}

#[cfg(test)]
mod token_wrapper_initialize_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        let token_wrapper = TokenWrapper {
            ..Default::default()
        };
        assert_eq!(token_wrapper.ai_dex_config, Pubkey::default());
        assert_eq!(token_wrapper.token_mint, Pubkey::default());
    }

    #[test]
    fn test_initialize() {
        let mut token_wrapper = TokenWrapper {
            ..Default::default()
        };
        let ai_dex_config = 
            Pubkey::from_str("EW3iWUphydEjoV7sCc6CK3LLEdrpDa9CKTJBxbCpuUQY").unwrap();
        let token_mint =
            Pubkey::from_str("8y6jyKgGcfDHzi3DgQn3ZHVimjawCU5o7Pr46RrB81fV").unwrap();

        let result = token_wrapper.initialize(
            ai_dex_config,
            token_mint,
        );
        assert!(result.is_ok());

        assert_eq!(ai_dex_config, token_wrapper.ai_dex_config);
        assert_eq!(token_mint, token_wrapper.token_mint);
    }
}
