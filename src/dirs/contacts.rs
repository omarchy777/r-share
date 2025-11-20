use crate::utils::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contact {
    pub name: String,
    pub public_key: String, // Hex-encoded
    pub added_at: String,   // Timestamp
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ContactList {
    pub contacts: HashMap<String, Contact>,
}

impl ContactList {
    /// Add a new contact
    pub fn add(&mut self, name: String, public_key: String) -> Result<()> {
        // Validate public key format
        let _ = hex::decode(&public_key)
            .map_err(|_e| Error::InvalidInput("Invalid public key format".to_string()));

        if self.contacts.contains_key(&name) {
            return Err(Error::InvalidInput(format!(
                "Contact '{}' already exists",
                name
            )));
        }

        let contact = Contact {
            name: name.clone(),
            public_key,
            added_at: chrono::Utc::now().to_rfc3339(),
        };

        self.contacts.insert(name, contact);
        Ok(())
    }

    #[allow(dead_code)]
    /// Get a contact by name
    pub fn get(&self, name: &str) -> Option<&Contact> {
        self.contacts.get(name)
    }

    /// Remove a contact
    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.contacts
            .remove(name)
            .ok_or_else(|| Error::InvalidInput(format!("Contact '{}' not found", name)))?;
        Ok(())
    }

    /// List all contacts
    pub fn list(&self) -> Vec<&Contact> {
        self.contacts.values().collect()
    }
}

/// Get contacts file path
fn get_contacts_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| Error::FileError("Could not find home directory".to_string()))?;
    Ok(home.join(".rshare").join("contact.json"))
}

/// Load contacts from disk
pub fn load_contacts() -> Result<ContactList> {
    let path = get_contacts_path()?;

    if !path.exists() {
        return Ok(ContactList::default());
    }

    let content = std::fs::read_to_string(&path)?;
    let contacts: ContactList = serde_json::from_str(&content)
        .map_err(|_e| Error::ConfigError("Invalid contacts json file".to_string()))?;

    Ok(contacts)
}

/// Save contacts to disk
pub fn save_contacts(contacts: &ContactList) -> Result<()> {
    let contact_path = get_contacts_path()?;

    // Ensure parent directory exists
    if let Some(parent) = contact_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_e| Error::FileError("Failed to create contact directory".to_string()))?;
    }

    let content = serde_json::to_string_pretty(contacts)
        .map_err(|_e| Error::ConfigError("Failed to serialize contacts".to_string()))?;

    std::fs::write(&contact_path, content)?;

    Ok(())
}
