# SDL_VPN

SDL_VPN is a high-performance, next-generation Peer-to-Peer (P2P) Virtual Private Network. It provides secure, anonymous internet access while utilizing a unique value exchange model.

## The Value Exchange

Our service operates on a hybrid model to give users the ultimate flexibility:
1. **Free Tier**: Enjoy unmetered, secure VPN access for free by opting into our proxy relay network. When your system is idle, you share a small fraction of your unused bandwidth with our enterprise residential proxy network. Strict guardrails ensure your CPU and bandwidth limits are respected, and the relay automatically pauses when you game or stream.
2. **Paid Subscription**: Prefer not to share your bandwidth? Opt into a premium subscription for a low monthly fee to access the VPN without any background relaying.

## Architecture

The ecosystem consists of three primary components, all written in pure **Rust** for memory safety, minimal footprint, and zero FFI overhead:
- **Central Orchestration Gateway (`crates/gateway`)**: A high-performance headless backend built on Tokio that handles node registration and proxies traffic.
- **Desktop Client (`apps/desktop-client`)**: A sleek consumer-facing application built with Tauri (Rust + React). It uses under 20MB of RAM.
- **Headless Client Daemon (`crates/daemon`)**: A lightweight CLI daemon for Linux servers and Docker enthusiasts.

## Build Instructions

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (1.80+ or Edition 2024 compatible)
- [Node.js](https://nodejs.org/) (for Tauri UI)

### Compiling
To build the entire workspace (Gateway, Core Protocol, and Daemon):
```bash
cargo build --release
```

To run the Tauri Desktop Application:
```bash
cd apps/desktop-client/src-tauri
npm install
cargo tauri dev
```
