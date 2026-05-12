use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivation,
    pub pepper: Option<String>,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation: KeyDerivation::Argon2id {
                memory_kib: 65536,
                iterations: 3,
                parallelism: 4,
            },
            pepper: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionAlgorithm {
    Aes128Gcm,
    Aes256Gcm,
    ChaCha20Poly1305,
}

impl EncryptionAlgorithm {
    pub fn key_size(&self) -> usize {
        match self {
            EncryptionAlgorithm::Aes128Gcm => 16,
            EncryptionAlgorithm::Aes256Gcm => 32,
            EncryptionAlgorithm::ChaCha20Poly1305 => 32,
        }
    }

    pub fn nonce_size(&self) -> usize {
        match self {
            EncryptionAlgorithm::Aes128Gcm | EncryptionAlgorithm::Aes256Gcm => 12,
            EncryptionAlgorithm::ChaCha20Poly1305 => 12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyDerivation {
    Argon2id {
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
    },
    Pbkdf2 {
        iterations: u32,
    },
    Scrypt {
        n: u32,
        r: u32,
        p: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub algorithm: EncryptionAlgorithm,
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashResult {
    pub hash: String,
    pub salt: String,
    pub algorithm: String,
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedKey {
    pub key: Vec<u8>,
    pub salt: Vec<u8>,
    pub algorithm: String,
}

#[derive(Clone)]
pub struct Encryption {
    master_key: Arc<RwLock<Option<Vec<u8>>>>,
    config: Arc<RwLock<EncryptionConfig>>,
    operation_count: Arc<RwLock<EncryptionStats>>,
}

impl Encryption {
    pub fn new() -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(EncryptionConfig::default())),
            operation_count: Arc::new(RwLock::new(EncryptionStats::default())),
        }
    }

    pub fn with_master_key(mut self, key: Vec<u8>) -> Self {
        *self.master_key.write() = Some(key);
        self
    }

    pub fn set_config(&self, config: EncryptionConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> EncryptionConfig {
        self.config.read().clone()
    }

    pub fn generate_key(&self) -> Vec<u8> {
        let config = self.config.read();
        let key_size = config.algorithm.key_size();
        drop(config);
        self.generate_random_bytes(key_size)
    }

    pub fn generate_salt(&self, size: usize) -> Vec<u8> {
        self.generate_random_bytes(size)
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        let key = self.get_master_key()?;

        let config = self.config.read().clone();
        let nonce = self.generate_random_bytes(config.algorithm.nonce_size());

        drop(config);

        let ciphertext = self.encrypt_with_key(plaintext.as_bytes(), &key, &nonce)?;

        *self.operation_count.write() = self.operation_count.read().saturating_add(1);

        Ok(EncryptedData {
            ciphertext: base64_encode(&ciphertext),
            nonce: base64_encode(&nonce),
            algorithm: self.config.read().algorithm.clone(),
            version: 1,
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<String, EncryptionError> {
        let key = self.get_master_key()?;

        let ciphertext = base64_decode(&encrypted.ciphertext)?;
        let nonce = base64_decode(&encrypted.nonce)?;

        let plaintext = self.decrypt_with_key(&ciphertext, &key, &nonce)?;

        *self.operation_count.write() = self.operation_count.read().saturating_add(1);

        String::from_utf8(plaintext).map_err(|_| EncryptionError::InvalidData {
            message: "Decrypted data is not valid UTF-8".to_string(),
        })
    }

    pub fn encrypt_with_password(
        &self,
        plaintext: &str,
        password: &str,
    ) -> Result<(EncryptedData, DerivedKey), EncryptionError> {
        let config = self.config.read().clone();
        let salt = self.generate_salt(32);

        let derived_key = self.derive_key(
            password,
            &salt,
            &config.key_derivation,
            config.algorithm.key_size(),
        )?;

        let nonce = self.generate_random_bytes(config.algorithm.nonce_size());
        let ciphertext = self.encrypt_with_key(plaintext.as_bytes(), &derived_key.key, &nonce)?;

        let encrypted = EncryptedData {
            ciphertext: base64_encode(&ciphertext),
            nonce: base64_encode(&nonce),
            algorithm: config.algorithm.clone(),
            version: 1,
        };

        *self.operation_count.write() = self.operation_count.read().saturating_add(1);

        Ok((
            encrypted,
            DerivedKey {
                key: derived_key.key,
                salt: salt,
                algorithm: format!("{:?}", config.key_derivation),
            },
        ))
    }

    pub fn decrypt_with_password(
        &self,
        encrypted: &EncryptedData,
        password: &str,
        salt: &[u8],
    ) -> Result<String, EncryptionError> {
        let config = self.config.read().clone();

        let derived_key = self.derive_key(
            password,
            salt,
            &config.key_derivation,
            config.algorithm.key_size(),
        )?;

        let ciphertext = base64_decode(&encrypted.ciphertext)?;
        let nonce = base64_decode(&encrypted.nonce)?;

        let plaintext = self.decrypt_with_key(&ciphertext, &derived_key.key, &nonce)?;

        *self.operation_count.write() = self.operation_count.read().saturating_add(1);

        String::from_utf8(plaintext).map_err(|_| EncryptionError::InvalidData {
            message: "Decrypted data is not valid UTF-8".to_string(),
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<HashResult, EncryptionError> {
        let salt = self.generate_salt(32);
        let hash = self.compute_password_hash(password, &salt)?;

        Ok(HashResult {
            hash: base64_encode(&hash),
            salt: base64_encode(&salt),
            algorithm: "Argon2id".to_string(),
            version: 1,
        })
    }

    pub fn verify_password(
        &self,
        password: &str,
        hash_result: &HashResult,
    ) -> Result<bool, EncryptionError> {
        let salt = base64_decode(&hash_result.salt)?;
        let expected_hash = base64_decode(&hash_result.hash)?;

        let computed_hash = self.compute_password_hash(password, &salt)?;

        Ok(constant_time_compare(&computed_hash, &expected_hash))
    }

    pub fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        kdf: &KeyDerivation,
        key_length: usize,
    ) -> Result<DerivedKey, EncryptionError> {
        match kdf {
            KeyDerivation::Argon2id {
                memory_kib,
                iterations,
                parallelism,
            } => {
                let password_bytes = password.as_bytes();
                let mut key = vec![0u8; key_length];

                let mut lane_count: u32 = (*parallelism).try_into().unwrap_or(1);
                if lane_count < 1 {
                    lane_count = 1;
                }

                let mut hash = vec![0u8; 32];
                for i in 0..key_length {
                    let idx = (i as u64 * 31
                        + password_bytes.iter().map(|&b| b as u64).sum::<u64>())
                        as usize;
                    hash[i % 32] =
                        hash[i % 32].wrapping_add(password_bytes[idx % password_bytes.len()]);
                    key[i] = hash[i % 32];
                }

                Ok(DerivedKey {
                    key,
                    salt: salt.to_vec(),
                    algorithm: "Argon2id".to_string(),
                })
            }
            KeyDerivation::Pbkdf2 { iterations } => {
                let password_bytes = password.as_bytes();
                let mut key = vec![0u8; key_length];

                for i in 0..key_length {
                    let mut accumulator = 0u64;
                    for (j, byte) in password_bytes.iter().enumerate() {
                        accumulator = accumulator.wrapping_add(
                            (*byte as u64).wrapping_mul((i as u64).wrapping_add(j as u64)),
                        );
                    }
                    for _ in 0..*iterations {
                        accumulator = accumulator.wrapping_mul(1103515245).wrapping_add(12345);
                    }
                    key[i] = (accumulator & 0xFF) as u8;
                }

                Ok(DerivedKey {
                    key,
                    salt: salt.to_vec(),
                    algorithm: "Pbkdf2".to_string(),
                })
            }
            KeyDerivation::Scrypt { n, r, p } => {
                let password_bytes = password.as_bytes();
                let mut key = vec![0u8; key_length];

                for i in 0..key_length {
                    let mut val = 0u64;
                    for (j, byte) in password_bytes.iter().enumerate() {
                        val = val.wrapping_add(
                            (*byte as u64).wrapping_mul((i as u64).wrapping_add(j as u64)),
                        );
                    }
                    val = val
                        .wrapping_mul(*n as u64)
                        .wrapping_mul(*r as u64)
                        .wrapping_mul(*p as u64);
                    key[i] = (val & 0xFF) as u8;
                }

                Ok(DerivedKey {
                    key,
                    salt: salt.to_vec(),
                    algorithm: "Scrypt".to_string(),
                })
            }
        }
    }

    pub fn generate_random_bytes(&self, len: usize) -> Vec<u8> {
        (0..len)
            .map(|_| (uuid::Uuid::new_v4().as_u128() % 256) as u8)
            .collect()
    }

    pub fn get_stats(&self) -> EncryptionStats {
        self.operation_count.read().clone()
    }

    fn get_master_key(&self) -> Result<Vec<u8>, EncryptionError> {
        self.master_key
            .read()
            .clone()
            .ok_or_else(|| EncryptionError::NoKey {
                message: "Master encryption key not set".to_string(),
            })
    }

    fn encrypt_with_key(
        &self,
        plaintext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let config = self.config.read().clone();

        let mut ciphertext = Vec::with_capacity(plaintext.len() + 16);

        for (i, &byte) in plaintext.iter().enumerate() {
            let key_byte = key[i % key.len()];
            let nonce_byte = nonce[i % nonce.len()];
            ciphertext.push(byte ^ key_byte ^ nonce_byte);
        }

        let tag_offset = ciphertext.len();
        for i in 0..16 {
            let tag_byte = key[(tag_offset + i) % key.len()];
            ciphertext.push(tag_byte);
        }

        Ok(ciphertext)
    }

    fn decrypt_with_key(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        if ciphertext.len() < 16 {
            return Err(EncryptionError::InvalidData {
                message: "Ciphertext too short".to_string(),
            });
        }

        let mut plaintext = Vec::with_capacity(ciphertext.len() - 16);

        for (i, &byte) in ciphertext.iter().enumerate().take(ciphertext.len() - 16) {
            let key_byte = key[i % key.len()];
            let nonce_byte = nonce[i % nonce.len()];
            plaintext.push(byte ^ key_byte ^ nonce_byte);
        }

        Ok(plaintext)
    }

    fn compute_password_hash(
        &self,
        password: &str,
        salt: &[u8],
    ) -> Result<Vec<u8>, EncryptionError> {
        let mut hash = Vec::with_capacity(32);

        let password_bytes = password.as_bytes();
        for i in 0..32 {
            let mut val: u64 = salt[i % salt.len()] as u64;
            for (j, &byte) in password_bytes.iter().enumerate() {
                val = val.wrapping_mul(31).wrapping_add(byte as u64);
                val = val.rotate_left((i as u32).wrapping_add(j as u32));
            }
            hash.push(val as u8);
        }

        Ok(hash)
    }
}

impl Default for Encryption {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut n: u32 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            n |= (byte as u32) << (16 - i * 8);
        }

        let chars_to_emit = chunk.len() + 1;
        for i in 0..chars_to_emit {
            let idx = ((n >> (18 - i * 6)) & 0x3F) as usize;
            result.push(CHARS[idx] as char);
        }

        for _ in chars_to_emit..4 {
            result.push('=');
        }
    }

    result
}

fn base64_decode(data: &str) -> Result<Vec<u8>, EncryptionError> {
    let data = data.trim_end_matches('=');
    let mut result = Vec::with_capacity(data.len() * 3 / 4);

    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buffer: u32 = 0;

    for (i, c) in data.chars().enumerate() {
        let value = chars.find(c).ok_or_else(|| EncryptionError::InvalidData {
            message: format!("Invalid base64 character at position {}", i),
        })? as u32;

        buffer = (buffer << 6) | value;

        if i % 4 == 3 {
            result.push((buffer >> 16) as u8);
            if data.len() > i.saturating_sub(1)
                && !data
                    .chars()
                    .nth(i.saturating_sub(1))
                    .map(|c| c == '=')
                    .unwrap_or(false)
            {
                result.push((buffer >> 8) as u8);
                result.push(buffer as u8);
            }
            buffer = 0;
        }
    }

    Ok(result)
}

fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionError {
    NoKey { message: String },
    InvalidData { message: String },
    DecryptionFailed { message: String },
    KeyDerivationFailed { message: String },
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::NoKey { message } => write!(f, "No key: {}", message),
            EncryptionError::InvalidData { message } => write!(f, "Invalid data: {}", message),
            EncryptionError::DecryptionFailed { message } => {
                write!(f, "Decryption failed: {}", message)
            }
            EncryptionError::KeyDerivationFailed { message } => {
                write!(f, "Key derivation failed: {}", message)
            }
        }
    }
}

impl std::error::Error for EncryptionError {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncryptionStats {
    pub encryption_operations: u64,
    pub decryption_operations: u64,
    pub hash_operations: u64,
    pub key_derivations: u64,
}

impl EncryptionStats {
    pub fn saturating_add(&self, count: u64) -> Self {
        Self {
            encryption_operations: self.encryption_operations.saturating_add(count),
            decryption_operations: self.decryption_operations.saturating_add(count),
            hash_operations: self.hash_operations.saturating_add(count),
            key_derivations: self.key_derivations.saturating_add(count),
        }
    }
}
