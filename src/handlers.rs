use axum::{http::StatusCode, response::Json};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use solana_program::{
    system_instruction,
};
use solana_sdk::signature::{Keypair as SolanaKeypair, Signer as SolanaSigner};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    instruction::{initialize_mint, mint_to, transfer},
};
use base64::{Engine as _, engine::general_purpose};

use crate::types::*;
use crate::utils::*;

pub async fn generate_keypair() -> Json<ApiResponse<KeypairResponse>> {
    let keypair = SolanaKeypair::new();
    let pubkey = bs58::encode(keypair.pubkey().to_bytes()).into_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Json(ApiResponse::success(KeypairResponse { pubkey, secret }))
}

pub async fn create_token(
    Json(payload): Json<CreateTokenRequest>,
) -> Result<Json<ApiResponse<InstructionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check for missing fields
    if payload.mint_authority.is_empty() || payload.mint.is_empty() {
        return Err(error_response("Missing required fields".to_string()));
    }

    // Validate inputs
    let mint_authority = validate_pubkey(&payload.mint_authority)
        .map_err(|e| error_response(format!("Invalid mint authority: {}", e)))?;
    
    let mint = validate_pubkey(&payload.mint)
        .map_err(|e| error_response(format!("Invalid mint: {}", e)))?;
    
    validate_decimals(payload.decimals)
        .map_err(|e| error_response(e))?;

    // Create initialize mint instruction
    let instruction = initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None, // freeze authority
        payload.decimals,
    )
    .map_err(|e| error_response(format!("Failed to create instruction: {}", e)))?;

    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: acc.pubkey.to_string(),
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect(),
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn mint_token(
    Json(payload): Json<MintTokenRequest>,
) -> Result<Json<ApiResponse<InstructionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check for missing fields
    if payload.mint.is_empty() || payload.destination.is_empty() || payload.authority.is_empty() {
        return Err(error_response("Missing required fields".to_string()));
    }

    // Validate inputs
    let mint = validate_pubkey(&payload.mint)
        .map_err(|e| error_response(format!("Invalid mint: {}", e)))?;
    
    let destination = validate_pubkey(&payload.destination)
        .map_err(|e| error_response(format!("Invalid destination: {}", e)))?;
    
    let authority = validate_pubkey(&payload.authority)
        .map_err(|e| error_response(format!("Invalid authority: {}", e)))?;
    
    validate_amount(payload.amount)
        .map_err(|e| error_response(e))?;

    // Create mint to instruction
    let instruction = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        payload.amount,
    )
    .map_err(|e| error_response(format!("Failed to create instruction: {}", e)))?;

    let response = InstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: acc.pubkey.to_string(),
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect(),
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn sign_message(
    Json(payload): Json<SignMessageRequest>,
) -> Result<Json<ApiResponse<SignMessageResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check for missing fields
    if payload.message.is_empty() || payload.secret.is_empty() {
        return Err(error_response("Missing required fields".to_string()));
    }

    // Validate secret key
    let secret_bytes = validate_base58_secret(&payload.secret)
        .map_err(|e| error_response(e))?;

    // Create keypair from secret
    let secret_key = SecretKey::from_bytes(&secret_bytes[..32])
        .map_err(|e| error_response(format!("Invalid secret key: {}", e)))?;
    
    let public_key = PublicKey::from(&secret_key);
    let keypair = Keypair { secret: secret_key, public: public_key };

    // Sign the message
    let signature = keypair.sign(payload.message.as_bytes());
    
    let response = SignMessageResponse {
        signature: general_purpose::STANDARD.encode(signature.to_bytes()),
        public_key: bs58::encode(public_key.to_bytes()).into_string(),
        message: payload.message,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn verify_message(
    Json(payload): Json<VerifyMessageRequest>,
) -> Result<Json<ApiResponse<VerifyMessageResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check for missing fields
    if payload.message.is_empty() || payload.signature.is_empty() || payload.pubkey.is_empty() {
        return Err(error_response("Missing required fields".to_string()));
    }

    // Validate public key
    let pubkey_bytes = bs58::decode(&payload.pubkey)
        .into_vec()
        .map_err(|_| error_response("Invalid public key format".to_string()))?;
    
    let public_key = PublicKey::from_bytes(&pubkey_bytes)
        .map_err(|e| error_response(format!("Invalid public key: {}", e)))?;

    // Decode signature
    let signature_bytes = general_purpose::STANDARD.decode(&payload.signature)
        .map_err(|_| error_response("Invalid signature format".to_string()))?;
    
    let signature = Signature::try_from(&signature_bytes[..])
        .map_err(|e| error_response(format!("Invalid signature: {}", e)))?;

    // Verify signature
    let valid = public_key.verify(payload.message.as_bytes(), &signature).is_ok();

    let response = VerifyMessageResponse {
        valid,
        message: payload.message,
        pubkey: payload.pubkey,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn send_sol(
    Json(payload): Json<SendSolRequest>,
) -> Result<Json<ApiResponse<SendSolResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check for missing fields
    if payload.from.is_empty() || payload.to.is_empty() {
        return Err(error_response("Missing required fields".to_string()));
    }

    // Validate inputs
    let from = validate_pubkey(&payload.from)
        .map_err(|e| error_response(format!("Invalid from address: {}", e)))?;
    
    let to = validate_pubkey(&payload.to)
        .map_err(|e| error_response(format!("Invalid to address: {}", e)))?;
    
    validate_amount(payload.lamports)
        .map_err(|e| error_response(e))?;

    // Create transfer instruction
    let instruction = system_instruction::transfer(&from, &to, payload.lamports);

    let response = SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| acc.pubkey.to_string())
            .collect(),
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn send_token(
    Json(payload): Json<SendTokenRequest>,
) -> Result<Json<ApiResponse<SendTokenResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check for missing fields
    if payload.destination.is_empty() || payload.mint.is_empty() || payload.owner.is_empty() {
        return Err(error_response("Missing required fields".to_string()));
    }

    // Validate inputs
    let destination = validate_pubkey(&payload.destination)
        .map_err(|e| error_response(format!("Invalid destination: {}", e)))?;
    
    let mint = validate_pubkey(&payload.mint)
        .map_err(|e| error_response(format!("Invalid mint: {}", e)))?;
    
    let owner = validate_pubkey(&payload.owner)
        .map_err(|e| error_response(format!("Invalid owner: {}", e)))?;
    
    validate_amount(payload.amount)
        .map_err(|e| error_response(e))?;

    // Get associated token addresses
    let source = get_associated_token_address(&owner, &mint);
    let destination_ata = get_associated_token_address(&destination, &mint);

    // Create transfer instruction
    let instruction = transfer(
        &spl_token::id(),
        &source,
        &destination_ata,
        &owner,
        &[],
        payload.amount,
    )
    .map_err(|e| error_response(format!("Failed to create instruction: {}", e)))?;

    let response = SendTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts: instruction
            .accounts
            .iter()
            .map(|acc| SendTokenAccountMeta {
                pubkey: acc.pubkey.to_string(),
                is_signer: acc.is_signer,
            })
            .collect(),
        instruction_data: general_purpose::STANDARD.encode(&instruction.data),
    };

    Ok(Json(ApiResponse::success(response)))
}

fn error_response(message: String) -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::error(message)),
    )
}