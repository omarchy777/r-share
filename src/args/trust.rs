use crate::dirs::contacts;
use crate::utils::error::Result;
use colored::Colorize;

/// Add a trusted contact
pub async fn add(name: String, pubkey: String) -> Result<()> {
    let mut contacts = contacts::load_contacts()?;

    // CLAP HANDLES THIS
    // Get public key from either --pubkey or --file
    /*let public_key = match pubkey {
        Some(key) => key,
        None => {
            return Err(crate::utils::error::Error::InvalidInput(
                "Must provide --pubkey".into(),
            ));
        }
    };*/

    contacts.add(name.clone(), pubkey)?;
    contacts::save_contacts(&contacts)?;

    println!("{} Trust added: {}", "✓".bright_green(), name.clone());

    Ok(())
}

/// List all trusted contacts
pub async fn list(verbose: bool) -> Result<()> {
    let contacts = contacts::load_contacts()?;

    if contacts.contacts.is_empty() {
        println!("{} No trust found", "✗".bright_yellow());
        return Ok(());
    }

    println!("{}", " Trusted Contacts:\n".bright_cyan().bold());

    for contact in contacts.list() {
        println!("{}", format!("  • {}", contact.name).bright_white().bold());

        if verbose {
            println!("    Key:   {}", &contact.public_key.bright_yellow());
            println!("    Added: {}", contact.added_at.dimmed());
        }

        println!();
    }

    Ok(())
}

/// Remove a trusted contact
pub async fn remove(name: String) -> Result<()> {
    let mut contacts = contacts::load_contacts()?;
    contacts.remove(&name)?;
    contacts::save_contacts(&contacts)?;

    println!("{} Removed contact: {}", "✓".bright_green(), name.clone());

    Ok(())
}
