use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

const SALT: &[u8] = b"flymode_v2_secret_salt_2026";
const NONCE_SIZE: usize = 12;

/// Derive a 256-bit AES key from a device_id and the app salt.
fn derive_key(device_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(SALT);
    hasher.finalize().into()
}

/// Encrypt a plaintext string using AES-256-GCM.
/// Returns a base64-encoded string containing nonce + ciphertext.
pub fn encrypt(plaintext: &str, device_id: &str) -> Result<String, String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let key_bytes = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Key init error: {}", e))?;

    let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption error: {}", e))?;

    // nonce || ciphertext → base64
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(combined))
}

/// Decrypt a base64-encoded AES-256-GCM ciphertext.
/// Returns the plaintext string.
pub fn decrypt(encoded: &str, device_id: &str) -> Result<String, String> {
    if encoded.is_empty() {
        return Ok(String::new());
    }

    let combined = BASE64
        .decode(encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    if combined.len() < NONCE_SIZE + 1 {
        return Err("Ciphertext too short".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key_bytes = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Key init error: {}", e))?;

    let mut plaintext_bytes = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed (wrong key or tampered data)".to_string())?;

    let plaintext = String::from_utf8(plaintext_bytes.clone())
        .map_err(|e| format!("UTF-8 decode error: {}", e))?;

    // Zeroize sensitive data from memory
    plaintext_bytes.zeroize();
    Ok(plaintext)
}

/// Encrypt an optional password field. Returns None if input is None.
pub fn encrypt_password(password: Option<&str>, device_id: &str) -> Result<Option<String>, String> {
    match password {
        Some(pw) if !pw.is_empty() => Ok(Some(encrypt(pw, device_id)?)),
        _ => Ok(None),
    }
}

/// Decrypt an optional encrypted password field. Returns None if input is None.
pub fn decrypt_password(encrypted: Option<&str>, device_id: &str) -> Result<Option<String>, String> {
    match encrypted {
        Some(enc) if !enc.is_empty() => Ok(Some(decrypt(enc, device_id)?)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TEST_DEVICE: &str = "test-device-id-12345";

    // ── round-trip tests ────────────────────────────────────────

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let password = "my_secret_password_123!";
        let encrypted = encrypt(password, TEST_DEVICE).unwrap();
        let decrypted = decrypt(&encrypted, TEST_DEVICE).unwrap();
        assert_eq!(decrypted, password);
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let password = "密碼🔑中文";
        let encrypted = encrypt(password, TEST_DEVICE).unwrap();
        let decrypted = decrypt(&encrypted, TEST_DEVICE).unwrap();
        assert_eq!(decrypted, password);
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let encrypted = encrypt("", TEST_DEVICE).unwrap();
        assert_eq!(encrypted, "");
        let decrypted = decrypt("", TEST_DEVICE).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_encrypt_produces_different_output_each_time() {
        let password = "same_password";
        let enc1 = encrypt(password, TEST_DEVICE).unwrap();
        let enc2 = encrypt(password, TEST_DEVICE).unwrap();
        assert_ne!(enc1, enc2, "Different nonces should produce different ciphertexts");

        // But both decrypt to the same value
        assert_eq!(decrypt(&enc1, TEST_DEVICE).unwrap(), password);
        assert_eq!(decrypt(&enc2, TEST_DEVICE).unwrap(), password);
    }

    // ── key/device isolation ────────────────────────────────────

    #[test]
    fn test_wrong_device_id_fails_decryption() {
        let encrypted = encrypt("secret", TEST_DEVICE).unwrap();
        let result = decrypt(&encrypted, "wrong-device-id");
        assert!(result.is_err(), "Decryption with wrong key should fail");
    }

    #[test]
    fn test_different_devices_produce_different_ciphertext() {
        let password = "same_password";
        let enc_a = encrypt(password, "device-a").unwrap();
        let enc_b = encrypt(password, "device-b").unwrap();
        // Different devices → different keys → different ciphertexts
        // (Also different nonces, but even with same nonce the ciphertext would differ)
        assert_ne!(enc_a, enc_b);
    }

    // ── tamper detection ────────────────────────────────────────

    #[test]
    fn test_tampered_ciphertext_fails() {
        let encrypted = encrypt("secret", TEST_DEVICE).unwrap();
        let mut bytes = BASE64.decode(&encrypted).unwrap();
        // Flip a byte in the ciphertext
        if let Some(b) = bytes.last_mut() {
            *b ^= 0xFF;
        }
        let tampered = BASE64.encode(&bytes);
        let result = decrypt(&tampered, TEST_DEVICE);
        assert!(result.is_err(), "Tampered ciphertext should fail decryption");
    }

    #[test]
    fn test_invalid_base64_fails() {
        let result = decrypt("not_valid_base64!!!", TEST_DEVICE);
        assert!(result.is_err());
    }

    #[test]
    fn test_too_short_ciphertext_fails() {
        let short = BASE64.encode(b"short");
        let result = decrypt(&short, TEST_DEVICE);
        assert!(result.is_err());
    }

    // ── password helper functions ───────────────────────────────

    #[test]
    fn test_encrypt_password_some() {
        let enc = encrypt_password(Some("secret"), TEST_DEVICE).unwrap();
        assert!(enc.is_some());
        let dec = decrypt_password(enc.as_deref(), TEST_DEVICE).unwrap();
        assert_eq!(dec, Some("secret".to_string()));
    }

    #[test]
    fn test_encrypt_password_none() {
        let enc = encrypt_password(None, TEST_DEVICE).unwrap();
        assert!(enc.is_none());
    }

    #[test]
    fn test_encrypt_password_empty() {
        let enc = encrypt_password(Some(""), TEST_DEVICE).unwrap();
        assert!(enc.is_none());
    }

    #[test]
    fn test_decrypt_password_none() {
        let dec = decrypt_password(None, TEST_DEVICE).unwrap();
        assert!(dec.is_none());
    }
}
