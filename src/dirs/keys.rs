use crate::utils::error::{Error, Result};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::TryRngCore;
use rand::rngs::OsRng;
use std::fs;
use std::path::PathBuf;

const PRIVATE_KEY_FILE: &str = "private.key";
const PUBLIC_KEY_FILE: &str = "public.key";

/// Get the default directory for storing keys
pub fn get_default_keys_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::KeyGenerationFailed("Could not find home directory".into()))?;
    Ok(home.join(".rshare").join("keys"))
}

/// Check if keys exist at given path (or default)
pub fn keys_exist_at(custom_path: &PathBuf) -> bool {
    let private_path = custom_path.join(PRIVATE_KEY_FILE);
    let public_path = custom_path.join(PUBLIC_KEY_FILE);
    private_path.exists() && public_path.exists()
}

/// Generate new Ed25519 keypair
pub fn generate_keys() -> Result<(SigningKey, VerifyingKey)> {
    let mut secret_bytes = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut secret_bytes)
        .map_err(|e| Error::KeyGenerationFailed(format!("Failed to generate bytes: {e}")))?;

    // Build the signing key from random bytes
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key: VerifyingKey = signing_key.verifying_key();

    let test_case = b"Test Message";
    let signature = signing_key.sign(test_case.as_ref());

    verifying_key.verify(test_case, &signature).map_err(|e| {
        Error::KeyGenerationFailed(format!("Failed to generate valid keypair: {e}"))
    })?;

    Ok((signing_key, verifying_key))
}

/// Save keys to disk with proper OS-level security
pub fn save_keys_to(
    signing_key: &SigningKey,
    verifying_key: &VerifyingKey,
    custom_dir: PathBuf,
) -> Result<PathBuf> {
    // Create directory with restrictive permissions
    fs::create_dir_all(&custom_dir)
        .map_err(|e| Error::KeyGenerationFailed(format!("Failed to create keys directory: {e}")))?;

    let private_path = custom_dir.join(PRIVATE_KEY_FILE);
    let public_path = custom_dir.join(PUBLIC_KEY_FILE);

    // Write private key
    fs::write(&private_path, signing_key.to_bytes())
        .map_err(|e| Error::KeyGenerationFailed(format!("Failed to write private key: {e}")))?;

    // Write public key
    fs::write(&public_path, verifying_key.to_bytes())
        .map_err(|e| Error::KeyGenerationFailed(format!("Failed to write public key: {e}")))?;

    // Set OS-level permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Private key: only owner can read/write (600)
        fs::set_permissions(&private_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            Error::KeyGenerationFailed(format!("Failed to set private key permissions: {e}"))
        })?;

        // Public key: owner read/write, others read (644)
        fs::set_permissions(&public_path, fs::Permissions::from_mode(0o644)).map_err(|e| {
            Error::KeyGenerationFailed(format!("Failed to set public key permissions: {e}"))
        })?;

        // Directory: only owner access (700)
        fs::set_permissions(&custom_dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
            Error::KeyGenerationFailed(format!("Failed to set directory permissions: {e}"))
        })?;
    }

    // On Windows, use security attributes
    #[cfg(windows)]
    {
        set_windows_security(&private_path)?;
    }

    Ok(custom_dir)
}

/// Load keys from disk (custom or default path)
pub fn load_keys_from(custom_dir: &PathBuf) -> Result<(SigningKey, VerifyingKey)> {
    let private_path = custom_dir.join(PRIVATE_KEY_FILE);
    let public_path = custom_dir.join(PUBLIC_KEY_FILE);

    // Read private key bytes
    let private_bytes = fs::read(&private_path)
        .map_err(|e| Error::FileRead(format!("Failed to read private key: {e}")))?;

    let private_key_bytes: [u8; 32] = private_bytes
        .try_into()
        .map_err(|_| Error::InvalidInput("Invalid private key size".into()))?;

    // Read public key bytes
    let public_bytes = fs::read(&public_path)
        .map_err(|e| Error::FileRead(format!("Failed to read public key: {e}")))?;

    let public_key_bytes: [u8; 32] = public_bytes
        .try_into()
        .map_err(|_| Error::InvalidInput("Invalid public key size".into()))?;

    // Construct keys
    let signing_key = SigningKey::from_bytes(&private_key_bytes);
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|e| Error::KeyGenerationFailed(format!("Invalid public key: {e}")))?;

    Ok((signing_key, verifying_key))
}

/// Validate that the keypair is correct
pub fn validate_keypair(signing_key: &SigningKey, verifying_key: &VerifyingKey) -> Result<()> {
    let test_message = b"rshare validation";
    let signature = signing_key.sign(test_message);

    verifying_key
        .verify(test_message, &signature)
        .map_err(|e| Error::KeyGenerationFailed(format!("Keypair mismatch: {e}")))?;

    Ok(())
}

/// Get public key fingerprint for display (from custom or default path)
pub fn get_public_key_fingerprint_from(custom_dir: &PathBuf) -> Result<String> {
    let (_, verifying_key) = load_keys_from(custom_dir)?;
    let bytes = verifying_key.to_bytes();
    Ok(hex::encode(&bytes[..8])) // First 8 bytes as hex
}

// Windows-specific security
#[cfg(windows)]
fn set_windows_security(path: &PathBuf) -> Result<()> {
    let metadata = fs::metadata(path)
        .map_err(|e| Error::KeyGenerationFailed(format!("Failed to get file metadata: {e}")))?;

    let mut perms = metadata.permissions();
    perms.set_readonly(false);
    fs::set_permissions(path, perms)
        .map_err(|e| Error::KeyGenerationFailed(format!("Failed to set permissions: {e}")))?;

    Ok(())
}
