use crate::utils::error::{Error, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Transfer role in the relay session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferRole {
    Sender,
    Receiver,
}

impl TransferRole {
    pub fn as_str(&self) -> &str {
        match self {
            TransferRole::Sender => "sender",
            TransferRole::Receiver => "receiver",
        }
    }
}

/// HTTP API request/response structures
#[derive(Debug, Serialize)]
struct ServeRequest {
    #[serde(rename = "senderFp")]
    sender_fingerprint: String,
    #[serde(rename = "receiverFp")]
    receiver_fingerprint: String,
    filename: String,
    #[serde(rename = "fileSize")]
    file_size: u64,
    signature: String,
}

#[derive(Debug, Serialize)]
struct ListenRequest {
    #[serde(rename = "receiverFp")]
    receiver_fingerprint: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ServeResponse {
    status: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "socketPort")]
    socket_port: u16,
    message: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListenResponse {
    status: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "senderFp")]
    sender_fp: String,
    filename: String,
    #[serde(rename = "fileSize")]
    file_size: u64,
    signature: String,
    #[serde(rename = "socketPort")]
    socket_port: u16,
    message: String,
}

/// Active transfer session with socket connection
pub struct TransferSession {
    session_id: String,
    role: TransferRole,
    socket: TcpStream,
    // Metadata (only populated for receiver)
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub signature: Option<String>,
    pub sender_fp: Option<String>,
}

impl TransferSession {
    /// Read data from the socket connection
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.socket
            .read(buf)
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to read from socket: {}", e)))
    }

    /// Write data to the socket connection
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.socket
            .write(data)
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to write to socket: {}", e)))
    }

    /// Write all data to the socket connection
    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.socket
            .write_all(data)
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to write all to socket: {}", e)))
    }

    /// Flush the socket connection
    pub async fn flush(&mut self) -> Result<()> {
        self.socket
            .flush()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to flush socket: {}", e)))
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the transfer role
    pub fn role(&self) -> TransferRole {
        self.role
    }
}

/// Client for interacting with the relay server
pub struct RelayClient {
    http_base_url: String,
    socket_host: String,
    socket_port: u16,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(http_base_url: String, socket_host: String, socket_port: u16) -> Self {
        Self {
            http_base_url,
            socket_host,
            socket_port,
        }
    }

    /// Initiate a file transfer as sender (blocks until receiver connects)
    pub async fn serve(
        &self,
        sender_fingerprint: String,
        receiver_fingerprint: String,
        filename: String,
        file_size: u64,
        signature: String,
    ) -> Result<TransferSession> {
        // Call HTTP API to create session
        let client = reqwest::Client::new();
        let url = format!("{}/api/relay/serve", self.http_base_url);

        let request = ServeRequest {
            sender_fingerprint,
            receiver_fingerprint,
            filename,
            file_size,
            signature,
        };

        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to call serve API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::NetworkError(format!(
                "Serve API failed with status {}: {}",
                status, body
            )));
        }

        let session: ServeResponse = response
            .json()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to parse session response: {}", e)))?;

        // Connect to socket server
        let socket = self
            .connect_socket(&session.session_id, TransferRole::Sender)
            .await?;

        Ok(TransferSession {
            session_id: session.session_id,
            role: TransferRole::Sender,
            socket,
            filename: None,
            file_size: None,
            signature: None,
            sender_fp: None,
        })
    }

    /// Join a file transfer as receiver (blocks until sender connects)
    pub async fn listen(&self, receiver_fingerprint: String) -> Result<TransferSession> {
        // Call HTTP API to join session
        let client = reqwest::Client::new();
        let url = format!("{}/api/relay/listen", self.http_base_url);

        let request = ListenRequest {
            receiver_fingerprint,
        };

        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to call listen API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::NetworkError(format!(
                "Listen API failed with status {}: {}",
                status, body
            )));
        }

        let session: ListenResponse = response
            .json()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to parse session response: {}", e)))?;

        // Connect to socket server
        let socket = self
            .connect_socket(&session.session_id, TransferRole::Receiver)
            .await?;

        Ok(TransferSession {
            session_id: session.session_id,
            role: TransferRole::Receiver,
            socket,
            filename: Some(session.filename),
            file_size: Some(session.file_size),
            signature: Some(session.signature),
            sender_fp: Some(session.sender_fp),
        })
    }

    /// Connect to the socket server and perform handshake
    async fn connect_socket(&self, session_id: &str, role: TransferRole) -> Result<TcpStream> {
        let addr = format!("{}:{}", self.socket_host, self.socket_port);

        let mut socket = TcpStream::connect(&addr).await.map_err(|e| {
            Error::NetworkError(format!("Failed to connect to socket server: {}", e))
        })?;

        // Send handshake: "session_id:role"
        let handshake = format!("{}:{}\n", session_id, role.as_str());
        socket
            .write_all(handshake.as_bytes())
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to send handshake: {}", e)))?;

        Ok(socket)
    }
}
