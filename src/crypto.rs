use anyhow::{anyhow, bail, Result};
use aes_gcm::{aead::{Aead, KeyInit, OsRng}, Aes256Gcm, Nonce};
use rand::RngCore;
use argon2::Argon2;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

// App-managed key is stored in base dir as app.key (random 32 bytes)
const APP_KEY_FILE: &str = "app.key";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub kdf: String, // argon2id
    pub salt_b64: String,
    pub params: KdfParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams { pub m_cost: u32, pub t_cost: u32, pub p_cost: u32 }

pub fn ensure_app_key(base: &Path) -> Result<()> {
    let key_path = base.join(APP_KEY_FILE);
    if !key_path.exists() {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        fs::write(&key_path, &key)?;
        key.zeroize();
    }
    Ok(())
}

pub fn load_app_key(base: &Path) -> Result<[u8;32]> {
    let data = fs::read(base.join(APP_KEY_FILE)).map_err(|e| anyhow!("read app.key: {e}"))?;
    if data.len()!=32 { bail!("invalid app.key size") }
    let mut key = [0u8;32];
    key.copy_from_slice(&data);
    Ok(key)
}

pub fn encrypt_with_key(key: &[u8;32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("cipher init: {e}"))?;
    let mut nonce_bytes = [0u8;12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut out = Vec::with_capacity(12 + plaintext.len() + 16);
    out.extend_from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, plaintext).map_err(|e| anyhow!("encrypt: {e}"))?;
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt_with_key(key: &[u8;32], data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 12 { bail!("cipher too short") }
    let (nonce_bytes, ct) = data.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("cipher init: {e}"))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let pt = cipher.decrypt(nonce, ct).map_err(|e| anyhow!("decrypt: {e}"))?;
    Ok(pt)
}

pub fn derive_key_from_password(password: &str, lock: &LockInfo) -> Result<[u8;32]> {
    let salt_bytes = B64.decode(&lock.salt_b64).map_err(|e| anyhow!("salt b64: {e}"))?;
    let params = argon2::Params::new(lock.params.m_cost, lock.params.t_cost, lock.params.p_cost, Some(32)).map_err(|e| anyhow!("params: {e}"))?;
    let argon = Argon2::new_with_secret(&[], argon2::Algorithm::Argon2id, argon2::Version::V0x13, params).map_err(|e| anyhow!("argon: {e}"))?;
    let mut out = [0u8;32];
    argon.hash_password_into(password.as_bytes(), &salt_bytes, &mut out).map_err(|e| anyhow!("derive: {e}"))?;
    Ok(out)
}

pub fn create_lock(password: &str) -> Result<LockInfo> {
    let mut salt = [0u8;16];
    OsRng.fill_bytes(&mut salt);
    let params = KdfParams { m_cost: 19456, t_cost: 2, p_cost: 1 }; // sensible defaults
    let paramsx = argon2::Params::new(params.m_cost, params.t_cost, params.p_cost, Some(32)).map_err(|e| anyhow!("params: {e}"))?;
    let argon = Argon2::new_with_secret(&[], argon2::Algorithm::Argon2id, argon2::Version::V0x13, paramsx).map_err(|e| anyhow!("argon: {e}"))?;
    // derive once to validate
    let mut out = [0u8;32];
    argon.hash_password_into(password.as_bytes(), &salt, &mut out).map_err(|e| anyhow!("derive: {e}"))?;
    Ok(LockInfo { kdf: "argon2id".into(), salt_b64: B64.encode(salt), params })
}
