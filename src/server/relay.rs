use crate::config::{ACK_SIGNAL, MAX_DONE_WAIT_SECS, READY_SIGNAL};
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
    #[serde(rename = "fileHash")]
    file_hash: String,
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
    #[serde(rename = "fileHash")]
    file_hash: String,
    #[serde(rename = "socketPort")]
    socket_port: u16,
    message: String,
}

/// Active transfer session with socket connection
#[allow(dead_code)]
pub struct TransferSession {
    session_id: String,
    role: TransferRole,
    socket: TcpStream,
    // Metadata (only populated for receiver)
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub signature: Option<String>,
    pub sender_fp: Option<String>,
    pub file_hash: Option<String>,
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
        file_hash: String,
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
            file_hash,
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
            file_hash: None,
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
            file_hash: Some(session.file_hash),
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

        // Wait for READY signal from server (indicates pairing complete)
        use tokio::io::AsyncReadExt;
        let mut ready_buffer = [0u8; 6]; // "READY\n" is 6 bytes
        socket
            .read_exact(&mut ready_buffer)
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to read READY signal: {}", e)))?;

        let ready_signal = String::from_utf8_lossy(&ready_buffer);
        if ready_signal.as_bytes() != READY_SIGNAL {
            return Err(Error::NetworkError(format!(
                "Expected READY signal, got: {}",
                ready_signal.trim()
            )));
        }

        // Send ACK to confirm we're ready to receive/send data
        socket
            .write_all(ACK_SIGNAL)
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to send ACK: {}", e)))?;

        // VERY CRITICAL!!! -> Give server time to process ACK and activate relay before data starts flowing
        tokio::time::sleep(tokio::time::Duration::from_millis(MAX_DONE_WAIT_SECS)).await;

        Ok(socket)
    }
}
