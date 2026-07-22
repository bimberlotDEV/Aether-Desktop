use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rusqlite::Connection;
use crate::db::Database;
use std::sync::Mutex;

const SECRETS_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS secrets (
        key         TEXT PRIMARY KEY NOT NULL,
        value       TEXT NOT NULL,  -- base64(nonce || ciphertext)
        created_at  TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
    );
";

/// Key names stored in the secrets table
pub const AI_API_KEY: &str = "ai_api_key";

/// Ensure the secrets table exists
pub fn ensure_table(conn: &Connection) -> Result<(), String> {
    conn.execute(SECRETS_TABLE_SQL, [])
        .map_err(|e| format!("Failed to create secrets table: {}", e))?;
    Ok(())
}

/// Derive an AES-256 key from the database path (deterministic per installation)
fn derive_key(db: &Database) -> Result<UnboundKey, String> {
    // Use the database path as key material - unique per installation
    // In production, this would use a proper KDF with salt
    let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    let path: String = conn
        .query_row("SELECT file_name FROM pragma_database_list WHERE name='main'", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get DB path: {}", e))?;
    
    // Create a 32-byte key by hashing the path
    let digest = ring::digest::digest(&ring::digest::SHA256, path.as_bytes());
    let key_bytes: &[u8; 32] = digest.as_ref().try_into()
        .map_err(|_| "Failed to derive key".to_string())?;
    
    UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|e| format!("Key creation error: {}", e))
}

/// Encrypt + store a secret value
pub fn store(db: &Database, key_name: &str, value: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    ensure_table(&conn)?;
    
    let unbound_key = derive_key(db)?;
    let key = LessSafeKey::new(unbound_key);
    
    // Generate random nonce (96 bits for AES-GCM)
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| format!("RNG error: {}", e))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    
    // Encrypt
    let mut in_out = value.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Encryption error: {}", e))?;
    
    // Store as: base64(nonce || ciphertext)
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&in_out);
    let encoded = BASE64.encode(&combined);
    
    conn.execute(
        "INSERT INTO secrets (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
        rusqlite::params![key_name, encoded],
    )
    .map_err(|e| format!("Failed to store secret: {}", e))?;
    
    Ok(())
}

/// Decrypt + retrieve a secret value
pub fn get(db: &Database, key_name: &str) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    ensure_table(&conn)?;
    
    let encoded: Option<String> = conn
        .query_row(
            "SELECT value FROM secrets WHERE key = ?1",
            [key_name],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Query error: {}", e))?;
    
    let encoded = match encoded {
        Some(e) => e,
        None => return Ok(None),
    };
    
    let combined = BASE64.decode(&encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    
    if combined.len() < 12 + 16 {
        return Err("Invalid secret data".to_string());
    }
    
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::assume_unique_for_key(nonce_bytes.try_into()
        .map_err(|_| "Invalid nonce".to_string())?);
    
    let unbound_key = derive_key(db)?;
    let key = LessSafeKey::new(unbound_key);
    
    let mut in_out = ciphertext.to_vec();
    let plaintext = key.open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Decryption error: {}", e))?;
    
    String::from_utf8(plaintext.to_vec())
        .map_err(|e| format!("UTF-8 error: {}", e))
        .map(Some)
}

/// Delete a stored secret
pub fn remove(db: &Database, key_name: &str) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    ensure_table(&conn)?;
    
    let deleted = conn
        .execute("DELETE FROM secrets WHERE key = ?1", [key_name])
        .map_err(|e| format!("Failed to delete secret: {}", e))?;
    
    Ok(deleted > 0)
}

// Helper trait for optional query results
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    
    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        Database { conn: Mutex::new(conn) }
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
}
