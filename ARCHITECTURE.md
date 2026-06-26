# SDL_VPN Architecture Design

This document details the internal architecture, cryptographic exchange, and concurrency mechanisms of the SDL_VPN ecosystem.

## 1. Network Topology (Reverse Tunnels)

Unlike traditional proxies where the proxy server binds a public IP/Port, residential internet connections are usually hidden behind CGNATs (Carrier-Grade NAT) or firewalls.

To traverse this, SDL_VPN uses a **Reverse Tunneling Architecture**:
1. The **Client Daemon** (or Desktop app) opens an outbound, persistent WebSocket (or raw TCP over TLS) connection to the **Central Gateway**.
2. The Gateway maps this open socket to the client's unique `Uuid` in its `dashmap` registry.
3. When an external scraper/VPN user wants to route traffic through a specific residential node, they connect to the Gateway.
4. The Gateway wraps the scraper's raw TCP traffic in a `TunnelMessage::Payload` packet and sends it *down* the already-open reverse tunnel to the client.
5. The Client unwraps the payload, dials the target website locally, and sends the HTTP response back up the tunnel.

## 2. Cryptographic Authentication (Ed25519)

To prevent spoofing or unauthorized bandwidth siphoning, clients must authenticate themselves using **Ed25519** public-key cryptography.

1. **Identity Generation**: Upon first boot, the Client Daemon generates a secure `SigningKey` (Private Key) via `OsRng` and derives the `VerifyingKey` (Public Key).
2. **Registration Payload**: 
   - The Client builds a `NodeInfo` struct containing its metadata.
   - It serializes the struct to bytes using `serde_json`.
   - It signs the bytes using the private key to generate a cryptographic `Signature`.
3. **Gateway Verification**:
   - The Gateway receives the `TunnelMessage::Register` packet.
   - It extracts the `NodeInfo` and the `Signature`.
   - It uses the embedded `public_key` to strictly verify that the signature matches the payload. If the signature is invalid, the connection is instantly dropped.

## 3. Concurrency and Scalability

Handling 10,000+ persistent websockets requires careful memory management.

- **Lock-Free Registry**: The `DashMap` crate shards the registry into many small segments. This ensures that a new node connecting does not block the entire gateway from routing traffic to existing nodes.
- **Bounded Channels**: Data moving between the proxy server and the client tunnels is passed over `tokio::sync::mpsc` channels with a strict capacity bound (e.g., 1024). This provides automatic backpressure. If a residential connection is slow, the channel fills up, and the gateway will drop packets rather than queuing them in memory until it OOMs.
- **Exponential Backoff**: If the Gateway restarts or drops packets, the Client Daemon uses the `backoff` crate to incrementally delay reconnections (1s, 2s, 4s, 8s...), preventing a Thundering Herd DDOS attack on the Gateway.
