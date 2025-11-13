use crate::config::{DEFAULT_HTTP_PORT, DEFAULT_SOCKET_PORT};
use crate::dirs::config::load_config;
use crate::server::RelayClient;
use crate::utils::error::Result;
use colored::Colorize;

pub async fn run(local: bool, public: bool) -> Result<()> {
    match load_config() {
        Ok(loaded_config) => {
            println!("{} Found config file", "✓".bright_green());

            let relay_client_public = RelayClient::new(
                loaded_config.server.public_ip.clone(),
                DEFAULT_HTTP_PORT.to_string().clone(),
                DEFAULT_SOCKET_PORT.clone(),
            );

            let relay_client_local = RelayClient::new(
                loaded_config.server.private_ip.clone(),
                DEFAULT_HTTP_PORT.to_string().clone(),
                DEFAULT_SOCKET_PORT.clone(),
            );

            if local {
                match relay_client_local.health_check().await {
                    Ok(_) => println!("{} Local relay is healthy", "✓".bright_green()),
                    Err(_) => println!("{} Local relay is unhealthy", "✗".bright_red()),
                }
            }

            if public {
                match relay_client_public.health_check().await {
                    Ok(_) => println!("{} Public relay is healthy", "✓".bright_green()),
                    Err(_) => println!("{} Public relay is unhealthy", "✗".bright_red()),
                }
            }
        }
        Err(_) => {
            println!();
            println!("{} No config file found", "✗".bright_red());
            println!(" rs init   Initialize rshare");
        }
    }

    Ok(())
}
