# Solana HTTP Server

A Rust-based HTTP server providing Solana blockchain functionality through RESTful endpoints

## Features

- Generate Solana keypairs
- Create SPL token mint instructions
- Create token minting instructions
- Sign and verify messages using Ed25519
- Create SOL transfer instructions
- Create SPL token transfer instructions

## Setup

1. Install Rust and Cargo
2. Clone this repository
3. Install dependencies:

   ```bash
   cargo build
   ```

4. Run the server:

   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000`

## API Endpoints

### Health Check

```bash
curl http://localhost:3000/
```

### Generate Keypair

```bash
curl -X POST http://localhost:3000/keypair
```

### Create Token Mint

```bash
curl -X POST http://localhost:3000/token/create \
  -H "Content-Type: application/json" \
  -d '{
    "mintAuthority": "11111111111111111111111111111112",
    "mint": "11111111111111111111111111111113",
    "decimals": 6
  }'
```

### Mint Tokens

```bash
curl -X POST http://localhost:3000/token/mint \
  -H "Content-Type: application/json" \
  -d '{
    "mint": "mint-address",
    "destination": "destination-address", 
    "authority": "authority-address",
    "amount": 1000000
  }'
```

### Sign Message

```bash
curl -X POST http://localhost:3000/message/sign \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello, Solana!",
    "secret": "base58-encoded-secret-key"
  }'
```

### Verify Message

```bash
curl -X POST http://localhost:3000/message/verify \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello, Solana!",
    "signature": "base64-encoded-signature",
    "pubkey": "base58-encoded-public-key"
  }'
```

### Send SOL

```bash
curl -X POST http://localhost:3000/send/sol \
  -H "Content-Type: application/json" \
  -d '{
    "from": "sender-address",
    "to": "recipient-address",
    "lamports": 100000
  }'
```

### Send Tokens

```bash
curl -X POST http://localhost:3000/send/token \
  -H "Content-Type: application/json" \
  -d '{
    "destination": "destination-address",
    "mint": "mint-address",
    "owner": "owner-address", 
    "amount": 100000
  }'
```

## Response Format

All endpoints return JSON responses with this structure:

**Success (200):**

```json
{
  "success": true,
  "data": { /* endpoint-specific data */ }
}
```

**Error (400):**

```json
{
  "success": false,
  "error": "Error description"
}
```

## Security Notes

- Private keys are never stored on the server
- All cryptographic operations use standard, audited libraries
- Input validation is performed on all endpoints
- Ed25519 signatures are used for message signing/verification

## Dependencies

- `axum` - Web framework
- `solana-sdk` - Solana blockchain SDK
- `spl-token` - SPL Token program bindings
- `ed25519-dalek` - Ed25519 cryptographic signatures
- `tokio` - Async runtime
- `serde` - Serialization framework
