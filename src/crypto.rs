//! Cryptography utilities: app key management and password KDF + AEAD.
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use anyhow::{Result, anyhow, bail};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use rand::{RngCore, rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

// App-managed key is stored in base dir as app.key (random 32 bytes)
const APP_KEY_FILE: &str = "app.key";

// Cryptographic constants
const AES_KEY_SIZE: usize = 32; // 256 bits for AES-256
const AES_GCM_NONCE_SIZE: usize = 12; // 96 bits standard for AES-GCM
const SALT_SIZE: usize = 16; // 128 bits for salt
const ARGON2_OUTPUT_SIZE: usize = 32; // 256 bits output

// Default Argon2 parameters (sensible security defaults)
const DEFAULT_ARGON2_M_COST: u32 = 19456; // 19 MiB
const DEFAULT_ARGON2_T_COST: u32 = 2;    // 2 iterations
const DEFAULT_ARGON2_P_COST: u32 = 1;    // 1 lane

/// Info needed to derive a key from a password using Argon2id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub kdf: String, // argon2id
    pub salt_b64: String,
    pub params: KdfParams,
}

/// Argon2id parameterization used for KDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

/// Ensure an app-managed 32-byte key exists at `app.key` under `base`.
pub fn ensure_app_key(base: &Path) -> Result<()> {
    let key_path = base.join(APP_KEY_FILE);
    if !key_path.exists() {
        let mut key = [0u8; AES_KEY_SIZE];
        rng().fill_bytes(&mut key);
        fs::write(&key_path, &key)?;
        key.zeroize();
    }
    Ok(())
}

/// Load the 32-byte app key from disk.
pub fn load_app_key(base: &Path) -> Result<[u8; AES_KEY_SIZE]> {
    let data = fs::read(base.join(APP_KEY_FILE)).map_err(|e| anyhow!("read app.key: {e}"))?;
    if data.len() != AES_KEY_SIZE {
        bail!("invalid app.key size")
    }
    let mut key = [0u8; AES_KEY_SIZE];
    key.copy_from_slice(&data);
    Ok(key)
}

/// Encrypt plaintext with AES-256-GCM, prepending a random 12-byte nonce.
pub fn encrypt_with_key(key: &[u8; AES_KEY_SIZE], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("cipher init: {e}"))?;
    let mut nonce_bytes = [0u8; AES_GCM_NONCE_SIZE];
    rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut out = Vec::with_capacity(AES_GCM_NONCE_SIZE + plaintext.len() + 16);
    out.extend_from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("encrypt: {e}"))?;
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decrypt data produced by `encrypt_with_key`.
pub fn decrypt_with_key(key: &[u8; AES_KEY_SIZE], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < AES_GCM_NONCE_SIZE {
        bail!("cipher too short")
    }
    let (nonce_bytes, ct) = data.split_at(AES_GCM_NONCE_SIZE);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("cipher init: {e}"))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let pt = cipher
        .decrypt(nonce, ct)
        .map_err(|e| anyhow!("decrypt: {e}"))?;
    Ok(pt)
}

/// Derive a 32-byte key from `password` using the provided `LockInfo`.
pub fn derive_key_from_password(password: &str, lock: &LockInfo) -> Result<[u8; AES_KEY_SIZE]> {
    let salt_bytes = B64
        .decode(&lock.salt_b64)
        .map_err(|e| anyhow!("salt b64: {e}"))?;
    let params = argon2::Params::new(
        lock.params.m_cost,
        lock.params.t_cost,
        lock.params.p_cost,
        Some(ARGON2_OUTPUT_SIZE),
    )
    .map_err(|e| anyhow!("params: {e}"))?;
    let argon = Argon2::new_with_secret(
        &[],
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    )
    .map_err(|e| anyhow!("argon: {e}"))?;
    let mut out = [0u8; ARGON2_OUTPUT_SIZE];
    argon
        .hash_password_into(password.as_bytes(), &salt_bytes, &mut out)
        .map_err(|e| anyhow!("derive: {e}"))?;
    Ok(out)
}

/// Derive a 32-byte key from `password` using a custom salt (for master lock).
pub fn derive_key_from_password_and_salt(password: &str, salt: &[u8]) -> Result<[u8; AES_KEY_SIZE]> {
    // Use consistent parameters for master lock derivation
    let params = argon2::Params::new(
        DEFAULT_ARGON2_M_COST,
        DEFAULT_ARGON2_T_COST,
        DEFAULT_ARGON2_P_COST,
        Some(ARGON2_OUTPUT_SIZE),
    )
    .map_err(|e| anyhow!("params: {e}"))?;
    let argon = Argon2::new_with_secret(
        &[],
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    )
    .map_err(|e| anyhow!("argon: {e}"))?;
    let mut out = [0u8; ARGON2_OUTPUT_SIZE];
    argon
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|e| anyhow!("derive: {e}"))?;
    Ok(out)
}

/// Create a `LockInfo` using fresh random salt and default costs; validates derivation once.
pub fn create_lock(password: &str) -> Result<LockInfo> {
    let mut salt = [0u8; SALT_SIZE];
    rng().fill_bytes(&mut salt);
    let params = KdfParams {
        m_cost: DEFAULT_ARGON2_M_COST,
        t_cost: DEFAULT_ARGON2_T_COST,
        p_cost: DEFAULT_ARGON2_P_COST,
    }; // sensible defaults
    let paramsx = argon2::Params::new(params.m_cost, params.t_cost, params.p_cost, Some(ARGON2_OUTPUT_SIZE))
        .map_err(|e| anyhow!("params: {e}"))?;
    let argon = Argon2::new_with_secret(
        &[],
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        paramsx,
    )
    .map_err(|e| anyhow!("argon: {e}"))?;
    // derive once to validate
    let mut out = [0u8; ARGON2_OUTPUT_SIZE];
    argon
        .hash_password_into(password.as_bytes(), &salt, &mut out)
        .map_err(|e| anyhow!("derive: {e}"))?;
    Ok(LockInfo {
        kdf: "argon2id".into(),
        salt_b64: B64.encode(salt),
        params,
    })
}

/// Create an Argon2 hash from a password for storage/verification
pub fn argon2_hash(password: &[u8]) -> Result<String> {
    // Generate a random salt using the same RNG system as other parts
    let mut salt_bytes = [0u8; SALT_SIZE];
    rng().fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|e| anyhow!("encode salt: {e}"))?;

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password, &salt)
        .map_err(|e| anyhow!("hash password: {e}"))?
        .to_string();
    Ok(password_hash)
}

/// Verify a password against an Argon2 hash
pub fn argon2_verify(password: &[u8], hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| anyhow!("parse hash: {e}"))?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password, &parsed_hash).is_ok())
}
