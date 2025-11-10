package com.scar.server.Model;

public class Session {
    private String sessionId;
    private String senderFp;
    private String receiverFp;
    private String filename;
    private long fileSize;
    private String signature;
    private String fileHash;
    private String status; // "waiting_receiver", "waiting_sender", "matched", "timeout"
    private long createdAt;
    private long expiresAt;
    private static final int SOCKET_PORT = 10000; 
    private static final long SESSION_TIMEOUT_MS = 300_000; // 5 minutes
    private String senderEphemeralKey; // X25519 public key from sender
    private String receiverEphemeralKey; // X25519 public key from receiver

    public Session() {
    }

    // Constructor
    public Session(String sessionId, String senderFp, String receiverFp,
            String filename, long fileSize, String signature, String fileHash,
            String senderEphemeralKey) {
        this.sessionId = sessionId;
        this.senderFp = senderFp;
        this.receiverFp = receiverFp;
        this.filename = filename;
        this.fileSize = fileSize;
        this.signature = signature;
        this.fileHash = fileHash;
        this.senderEphemeralKey = senderEphemeralKey;
        this.createdAt = System.currentTimeMillis();
        this.expiresAt = this.createdAt + SESSION_TIMEOUT_MS;
        this.status = "waiting_receiver";
    }

    // Getters
    public String getSessionId() {
        return sessionId;
    }

    public String getSenderFp() {
        return senderFp;
    }

    public String getReceiverFp() {
        return receiverFp;
    }

    public String getFilename() {
        return filename;
    }

    public long getFileSize() {
        return fileSize;
    }

    public String getSignature() {
        return signature;
    }

    public String getFileHash() {
        return fileHash;
    }

    public String getSenderEphemeralKey() {
        return senderEphemeralKey;
    }

    public void setSenderEphemeralKey(String senderEphemeralKey) {
        this.senderEphemeralKey = senderEphemeralKey;
    }

    public String getReceiverEphemeralKey() {
        return receiverEphemeralKey;
    }

    public void setReceiverEphemeralKey(String receiverEphemeralKey) {
        this.receiverEphemeralKey = receiverEphemeralKey;
    }

    public String getStatus() {
        return status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public long getExpiresAt() {
        return expiresAt;
    }

    public int getSocketPort() {
        return SOCKET_PORT;
    }

    public boolean isExpired() {
        return System.currentTimeMillis() > expiresAt;
    }
}
