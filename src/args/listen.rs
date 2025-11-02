use crate::crypto::signing;
use crate::dirs::{config, contacts, keys};
use crate::server::RelayClient;
use crate::utils::error::{Error, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Listen for incoming file transfers
pub async fn run(
    path: Option<PathBuf>,
    from: Option<String>,
    _max_size: u32,
    _quiet: bool,
) -> Result<()> {
    println!("{}", "🎧 Starting listener...\n".bright_cyan().bold());

    // Load config and keys
    let config = config::load_config()?;
    let (_signing_key, verifying_key) = keys::load_keys_from(&config.path.keys_path)?;
    let my_fingerprint = hex::encode(verifying_key.to_bytes());

    // Determine download path
    let download_path = path.unwrap_or_else(|| config.path.download_path.clone());
    std::fs::create_dir_all(&download_path)?;

    println!("{} Ready to receive files", "✓".bright_green());
    println!(
        "   Save to: {}",
        download_path.display().to_string().bright_yellow()
    );
    //println!("   Max size: {} MB", _max_size);
    println!(
        "   Fingerprint: {}...",
        &my_fingerprint[..16].bright_cyan().dimmed()
    );

    // Load contacts for verification
    let contact_list = contacts::load_contacts()?;

    // If --from specified, validate contact exists
    let expected_sender = if let Some(ref contact_name) = from {
        let contact = contact_list
            .get(contact_name)
            .ok_or_else(|| Error::InvalidInput(format!("Contact '{}' not found", contact_name)))?;
        println!("   Only from: {}", contact_name.bright_yellow());
        Some(contact.public_key.clone())
    } else {
        None
    };

    println!();

    // Create relay client from config
    let relay_client = RelayClient::new(
        config.server.http_url.clone(),
        config.server.socket_host.clone(),
        config.server.socket_port,
    );

    // Join transfer session (blocks until sender connects)
    println!("{}", "⏳ Waiting for sender to connect...".yellow());
    let mut session = relay_client.listen(my_fingerprint.clone()).await?;

    println!(
        "{} Sender connected! Session: {}",
        "✓".bright_green(),
        session.session_id().bright_cyan()
    );
    println!();

    // Extract metadata from HTTP response
    let filename = session
        .filename
        .clone()
        .ok_or_else(|| Error::InvalidInput("No filename in session".into()))?;
    let filesize = session
        .file_size
        .ok_or_else(|| Error::InvalidInput("No file size in session".into()))?;
    let signature_hex = session
        .signature
        .clone()
        .ok_or_else(|| Error::InvalidInput("No signature in session".into()))?;
    let sender_fp = session
        .sender_fp
        .clone()
        .ok_or_else(|| Error::InvalidInput("No sender fingerprint in session".into()))?;

    // Verify signature if --from specified
    if let Some(expected_fp) = expected_sender {
        if expected_fp != sender_fp {
            return Err(Error::InvalidInput(format!(
                "Sender fingerprint mismatch! Expected {}, got {}",
                &expected_fp[..16],
                &sender_fp[..16]
            )));
        }

        // Decode sender's public key
        let sender_key_bytes = hex::decode(sender_fp)
            .map_err(|_| Error::InvalidInput("Invalid sender public key".into()))?;
        let sender_key = ed25519_dalek::VerifyingKey::from_bytes(
            sender_key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidInput("Invalid key length".into()))?,
        )
        .map_err(|_| Error::InvalidInput("Invalid sender key".into()))?;

        // Verify signature
        let metadata_msg = format!("{}|{}", filename, filesize);
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|_| Error::InvalidInput("Invalid signature hex".into()))?;
        let signature = ed25519_dalek::Signature::from_bytes(
            signature_bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidInput("Invalid signature length".into()))?,
        );

        signing::verify_signature(&sender_key, &metadata_msg, &signature)?;
        println!("{} Signature verified", "✓".bright_green());
    }

    println!("{} Incoming file transfer", "✓".bright_green());
    println!("   File: {}", filename.bright_yellow());
    println!(
        "   Size: {} bytes ({:.2} MB)",
        filesize,
        filesize as f64 / (1024.0 * 1024.0)
    );
    println!();
    println!("{} Receiving file...", "⬇".bright_cyan());

    // Receive file data with progress bar
    let file_path = download_path.join(filename);
    let mut file_writer = File::create(&file_path).await?;

    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks
    let mut total_received = 0u64;

    while total_received < filesize {
        let n = session.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        file_writer.write_all(&buffer[..n]).await?;
        total_received += n as u64;
        pb.set_position(total_received);
    }

    file_writer.flush().await?;
    pb.finish_with_message("Download complete!");

    println!();
    println!("{} File received successfully!", "✓".bright_green().bold());
    println!("   Saved to: {}", file_path.display());
    println!(
        "   Size: {} bytes ({:.2} MB)",
        total_received,
        total_received as f64 / (1024.0 * 1024.0)
    );

    Ok(())
}
