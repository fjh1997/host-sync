use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .expect("key derivation failed");
    key
}

/// Encrypts plaintext with AES-256-GCM + Argon2 key derivation.
/// Output: base64(salt ‖ nonce ‖ ciphertext)
pub fn encrypt(plaintext: &str, passphrase: &str) -> String {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();

    let mut combined = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    B64.encode(&combined)
}

/// Decrypts data produced by [encrypt].
pub fn decrypt(encoded: &str, passphrase: &str) -> Result<String, &'static str> {
    let combined = B64.decode(encoded).map_err(|_| "invalid base64")?;
    if combined.len() < SALT_LEN + NONCE_LEN + 1 {
        return Err("data too short");
    }
    let salt = &combined[..SALT_LEN];
    let nonce_bytes = &combined[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &combined[SALT_LEN + NONCE_LEN..];

    let key = derive_key(passphrase, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "decryption failed")?;
    String::from_utf8(plaintext).map_err(|_| "invalid utf8")
}

/// Generates a random 64-char hex key for first-time setup.
pub fn generate_random_key() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(&bytes)
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let msg = "hello world 你好世界";
        let pass = "test-passphrase";
        let enc = encrypt(msg, pass);
        let dec = decrypt(&enc, pass).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let enc = encrypt("secret", "correct");
        assert!(decrypt(&enc, "wrong").is_err());
    }
}
