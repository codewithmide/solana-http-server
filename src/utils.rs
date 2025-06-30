use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn validate_pubkey(pubkey_str: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(pubkey_str).map_err(|_| "Invalid public key format".to_string())
}

pub fn validate_amount(amount: u64) -> Result<(), String> {
    if amount == 0 {
        return Err("Amount must be greater than 0".to_string());
    }
    Ok(())
}

pub fn validate_decimals(decimals: u8) -> Result<(), String> {
    if decimals > 9 {
        return Err("Decimals must be between 0 and 9".to_string());
    }
    Ok(())
}

pub fn validate_base58_secret(secret: &str) -> Result<Vec<u8>, String> {
    bs58::decode(secret)
        .into_vec()
        .map_err(|_| "Invalid base58 secret key format".to_string())
        .and_then(|bytes| {
            if bytes.len() != 64 {
                Err("Secret key must be 64 bytes".to_string())
            } else {
                Ok(bytes)
            }
        })
}
