package com.scar.server.Dto;

public class ListenRequest {
    private String receiverFp;
    private String receiverEphemeralKey; // X25519 public key (hex-encoded)

    // Constructors
    public ListenRequest() {
    }

    public ListenRequest(String receiverFp, String receiverEphemeralKey) {
        this.receiverFp = receiverFp;
        this.receiverEphemeralKey = receiverEphemeralKey;
    }

    // Getters
    public String getReceiverFp() {
        return receiverFp;
    }

    public String getReceiverEphemeralKey() {
        return receiverEphemeralKey;
    }

    // Setters
    public void setReceiverFp(String receiverFp) {
        this.receiverFp = receiverFp;
    }

    public void setReceiverEphemeralKey(String receiverEphemeralKey) {
        this.receiverEphemeralKey = receiverEphemeralKey;
    }
}
