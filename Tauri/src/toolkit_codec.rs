use std::io::Cursor;

use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedPayload {
    pub algorithm: &'static str,
    pub key_derivation: &'static str,
    pub nonce_base64: String,
    pub ciphertext_base64: String,
}

fn derive_key(passphrase: &str) -> [u8; 32] {
    let digest = Sha256::digest(passphrase.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    key
}

pub fn compress_text_lzma(payload: String) -> Result<String, String> {
    let mut compressed = Vec::new();
    lzma_rs::lzma_compress(&mut Cursor::new(payload.into_bytes()), &mut compressed)
        .map_err(|error| format!("LZMA compression failed: {error}"))?;
    Ok(BASE64.encode(compressed))
}

pub fn decompress_text_lzma(payload_base64: String) -> Result<String, String> {
    let compressed = BASE64
        .decode(payload_base64)
        .map_err(|error| format!("Invalid base64 payload: {error}"))?;
    let mut decompressed = Vec::new();
    lzma_rs::lzma_decompress(&mut Cursor::new(compressed), &mut decompressed)
        .map_err(|error| format!("LZMA decompression failed: {error}"))?;
    String::from_utf8(decompressed)
        .map_err(|error| format!("Decoded text is not valid UTF-8: {error}"))
}

pub fn encrypt_text_payload(
    payload: String,
    passphrase: String,
) -> Result<EncryptedPayload, String> {
    if passphrase.len() < 8 {
        return Err("Passphrase must be at least 8 characters long.".to_string());
    }

    let key = derive_key(&passphrase);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|error| format!("Unable to initialize cipher: {error}"))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload.as_bytes())
        .map_err(|error| format!("Encryption failed: {error}"))?;

    Ok(EncryptedPayload {
        algorithm: "AES-256-GCM",
        key_derivation: "SHA-256(passphrase)",
        nonce_base64: BASE64.encode(nonce_bytes),
        ciphertext_base64: BASE64.encode(ciphertext),
    })
}

pub fn decrypt_text_payload(
    ciphertext_base64: String,
    nonce_base64: String,
    passphrase: String,
) -> Result<String, String> {
    let key = derive_key(&passphrase);
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|error| format!("Unable to initialize cipher: {error}"))?;

    let nonce_bytes = BASE64
        .decode(nonce_base64)
        .map_err(|error| format!("Invalid nonce base64: {error}"))?;

    if nonce_bytes.len() != 12 {
        return Err("Nonce must be 12 bytes long after decoding.".to_string());
    }

    let ciphertext = BASE64
        .decode(ciphertext_base64)
        .map_err(|error| format!("Invalid ciphertext base64: {error}"))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .map_err(|error| format!("Decryption failed: {error}"))?;

    String::from_utf8(plaintext)
        .map_err(|error| format!("Decrypted payload is not valid UTF-8: {error}"))
}
