use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rusqlite::Connection;
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
};

use crate::db::Database;

const SECRETS_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS secrets (
        key         TEXT PRIMARY KEY NOT NULL,
        value       TEXT NOT NULL,  -- base64(encrypted blob)
        created_at  TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
    );
";

/// Legacy DeepSeek key name retained for upgrades from pre-provider builds.
pub const AI_API_KEY: &str = "ai_api_key";
const DEEPSEEK_API_KEY: &str = "ai_api_key:deepseek";
const OPENAI_API_KEY: &str = "ai_api_key:openai";

fn provider_key(provider: &str) -> Result<&'static str, String> {
    match provider {
        "deepseek" => Ok(DEEPSEEK_API_KEY),
        "openai" => Ok(OPENAI_API_KEY),
        _ => Err("Unknown AI provider.".to_string()),
    }
}

/// Store a credential under its provider namespace. Values never cross providers.
pub fn store_provider_key(db: &Database, provider: &str, value: &str) -> Result<(), String> {
    store(db, provider_key(provider)?, value)
}

/// Read a provider credential. DeepSeek falls back to the legacy key without
/// rewriting or deleting it, so an interrupted upgrade cannot lose the secret.
pub fn get_provider_key(db: &Database, provider: &str) -> Result<Option<String>, String> {
    let current = get(db, provider_key(provider)?)?;
    if current.is_some() || provider != "deepseek" {
        return Ok(current);
    }
    get(db, AI_API_KEY)
}

/// Remove only the requested provider's credential. DeepSeek also removes its
/// backward-compatible legacy entry, while OpenAI cannot affect it.
pub fn remove_provider_key(db: &Database, provider: &str) -> Result<bool, String> {
    let removed = remove(db, provider_key(provider)?)?;
    if provider == "deepseek" {
        Ok(remove(db, AI_API_KEY)? || removed)
    } else {
        Ok(removed)
    }
}

/// Encryption boundary used by credential persistence.
pub trait SecretCrypto: Send + Sync {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String>;
}

/// Windows Data Protection API implementation, bound to the current user.
pub struct DpapiCrypto;

impl DpapiCrypto {
    fn blob_for(data: &[u8]) -> Result<CRYPT_INTEGER_BLOB, String> {
        let length = u32::try_from(data.len())
            .map_err(|_| "Credential data exceeds the Windows DPAPI size limit".to_string())?;

        Ok(CRYPT_INTEGER_BLOB {
            cbData: length,
            pbData: data.as_ptr().cast_mut(),
        })
    }

    fn copy_and_free(blob: CRYPT_INTEGER_BLOB) -> Result<Vec<u8>, String> {
        if blob.cbData == 0 {
            if !blob.pbData.is_null() {
                // SAFETY: this is a DPAPI-owned output buffer released exactly once.
                unsafe {
                    let _ = LocalFree(blob.pbData.cast());
                }
            }
            return Ok(Vec::new());
        }

        if blob.cbData > 0 && blob.pbData.is_null() {
            return Err("Windows DPAPI returned an invalid output buffer".to_string());
        }

        // SAFETY: DPAPI owns `pbData` and reports its exact byte length in `cbData`.
        // The bytes are copied before the buffer is released with the matching LocalFree API.
        let bytes =
            unsafe { std::slice::from_raw_parts(blob.pbData, blob.cbData as usize) }.to_vec();
        if !blob.pbData.is_null() {
            // SAFETY: DPAPI allocates output buffers with LocalAlloc; LocalFree is the
            // documented matching deallocator, and this pointer is freed exactly once.
            unsafe {
                let _ = LocalFree(blob.pbData.cast());
            }
        }

        Ok(bytes)
    }
}

impl SecretCrypto for DpapiCrypto {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let input = Self::blob_for(data)?;
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        // SAFETY: all pointers refer to valid blobs for the duration of this call;
        // optional parameters are null, and output ownership is handled by copy_and_free.
        let succeeded = unsafe {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };

        if succeeded == 0 {
            return Err(format!(
                "Windows DPAPI encryption failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        Self::copy_and_free(output)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let input = Self::blob_for(data)?;
        let mut output = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        // SAFETY: all pointers refer to valid blobs for the duration of this call;
        // optional parameters are null, and output ownership is handled by copy_and_free.
        let succeeded = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };

        if succeeded == 0 {
            return Err(format!(
                "Windows DPAPI decryption failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        Self::copy_and_free(output)
    }
}

/// Ensure the secrets table exists.
pub fn ensure_table(conn: &Connection) -> Result<(), String> {
    conn.execute(SECRETS_TABLE_SQL, [])
        .map_err(|e| format!("Failed to create secrets table: {}", e))?;
    Ok(())
}

/// Encrypt and store a secret value.
pub fn store(db: &Database, key_name: &str, value: &str) -> Result<(), String> {
    let encrypted = db.crypto.encrypt(value.as_bytes())?;
    let encoded = BASE64.encode(encrypted);

    let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    ensure_table(&conn)?;
    conn.execute(
        "INSERT INTO secrets (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
        rusqlite::params![key_name, encoded],
    )
    .map_err(|e| format!("Failed to store secret: {}", e))?;

    Ok(())
}

/// Retrieve and decrypt a secret value.
pub fn get(db: &Database, key_name: &str) -> Result<Option<String>, String> {
    let encoded: Option<String> = {
        let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        ensure_table(&conn)?;
        conn.query_row(
            "SELECT value FROM secrets WHERE key = ?1",
            [key_name],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Query error: {}", e))?
    };

    let Some(encoded) = encoded else {
        return Ok(None);
    };

    let encrypted = BASE64
        .decode(encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    let plaintext = db.crypto.decrypt(&encrypted)?;

    String::from_utf8(plaintext)
        .map(Some)
        .map_err(|e| format!("UTF-8 error: {}", e))
}

/// Delete a stored secret.
pub fn remove(db: &Database, key_name: &str) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    ensure_table(&conn)?;

    let deleted = conn
        .execute("DELETE FROM secrets WHERE key = ?1", [key_name])
        .map_err(|e| format!("Failed to delete secret: {}", e))?;

    Ok(deleted > 0)
}

trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
struct TestCrypto;

#[cfg(test)]
impl TestCrypto {
    const KEY: [u8; 32] = [0xA5; 32];
}

#[cfg(test)]
impl SecretCrypto for TestCrypto {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
        use ring::rand::{SecureRandom, SystemRandom};

        let key = UnboundKey::new(&AES_256_GCM, &Self::KEY)
            .map(LessSafeKey::new)
            .map_err(|_| "Test key creation failed".to_string())?;
        let mut nonce_bytes = [0u8; 12];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| "Test nonce generation failed".to_string())?;
        let mut encrypted = data.to_vec();
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut encrypted,
        )
        .map_err(|_| "Test encryption failed".to_string())?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&encrypted);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

        if data.len() < 12 + AES_256_GCM.tag_len() {
            return Err("Invalid test ciphertext".to_string());
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::assume_unique_for_key(
            nonce_bytes
                .try_into()
                .map_err(|_| "Invalid test nonce".to_string())?,
        );
        let key = UnboundKey::new(&AES_256_GCM, &Self::KEY)
            .map(LessSafeKey::new)
            .map_err(|_| "Test key creation failed".to_string())?;
        let mut plaintext = ciphertext.to_vec();
        let opened = key
            .open_in_place(nonce, Aad::empty(), &mut plaintext)
            .map_err(|_| "Test decryption failed".to_string())?;
        Ok(opened.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        Database {
            conn: Mutex::new(conn),
            crypto: Box::new(TestCrypto),
        }
    }

    #[test]
    fn test_store_and_get() {
        let db = test_db();
        store(&db, "test_key", "my-secret-value").unwrap();
        let result = get(&db, "test_key").unwrap();
        assert_eq!(result, Some("my-secret-value".to_string()));
    }

    #[test]
    fn test_missing_key() {
        let db = test_db();
        let result = get(&db, "nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_remove() {
        let db = test_db();
        store(&db, "test_key", "value").unwrap();
        let removed = remove(&db, "test_key").unwrap();
        assert!(removed);
        let result = get(&db, "test_key").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_overwrite() {
        let db = test_db();
        store(&db, "test_key", "first").unwrap();
        store(&db, "test_key", "second").unwrap();
        let result = get(&db, "test_key").unwrap();
        assert_eq!(result, Some("second".to_string()));
    }

    #[test]
    fn provider_keys_are_isolated() {
        let db = test_db();
        store_provider_key(&db, "deepseek", "deep-key").unwrap();
        store_provider_key(&db, "openai", "openai-key").unwrap();
        assert_eq!(
            get_provider_key(&db, "deepseek").unwrap().as_deref(),
            Some("deep-key")
        );
        assert_eq!(
            get_provider_key(&db, "openai").unwrap().as_deref(),
            Some("openai-key")
        );
        remove_provider_key(&db, "openai").unwrap();
        assert_eq!(
            get_provider_key(&db, "deepseek").unwrap().as_deref(),
            Some("deep-key")
        );
        assert_eq!(get_provider_key(&db, "openai").unwrap(), None);
    }

    #[test]
    fn deepseek_reads_and_removes_legacy_key() {
        let db = test_db();
        store(&db, AI_API_KEY, "legacy-key").unwrap();
        assert_eq!(
            get_provider_key(&db, "deepseek").unwrap().as_deref(),
            Some("legacy-key")
        );
        assert!(remove_provider_key(&db, "deepseek").unwrap());
        assert_eq!(get(&db, AI_API_KEY).unwrap(), None);
    }

    #[test]
    fn unknown_provider_cannot_create_arbitrary_secret_names() {
        let db = test_db();
        assert!(store_provider_key(&db, "custom", "secret").is_err());
        assert!(get_provider_key(&db, "custom").is_err());
        assert!(remove_provider_key(&db, "custom").is_err());
    }
}
