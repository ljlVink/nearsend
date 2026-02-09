# NearSend

A LocalSend protocol-compatible client built with GPUI and gpui-component.

## Overview

NearSend is a cross-platform file sharing application that implements the LocalSend protocol v2.0. It allows you to securely share files and messages with nearby devices over your local network without requiring an internet connection.

## Features

- ✅ **Device Discovery**: Automatic discovery of nearby devices using UDP multicast (port 53317, address 224.0.0.167)
- ✅ **LocalSend Protocol Compatibility**: Full compatibility with LocalSend protocol v2.0
- ✅ **HTTP/HTTPS Server**: Receives files and messages from other devices
- ✅ **HTTP/HTTPS Client**: Sends files and messages to other devices
- ✅ **Modern UI**: Built with GPUI and gpui-component for a native desktop experience
- ✅ **Cross-platform**: Works on macOS, Windows, and Linux

## Architecture

### Current Implementation

The project currently has a custom protocol implementation, but **we recommend migrating to use the `localsend-rs` crate** (see MIGRATION.md).

Current modules:
- **`protocol.rs`**: Defines the LocalSend protocol structures and types (can be replaced by localsend-rs)
- **`discovery.rs`**: Implements UDP multicast discovery service (can be replaced by localsend-rs)
- **`server.rs`**: HTTP/HTTPS server for receiving transfers (can be replaced by localsend-rs)
- **`client.rs`**: HTTP/HTTPS client for sending transfers (can be replaced by localsend-rs)
- **`app.rs`**: Main application UI and state management (UI part should be kept)

### Recommended Approach

Use the official `localsend` core crate from the LocalSend repository:
- ✅ Better protocol compatibility
- ✅ Less code to maintain
- ✅ Community-tested implementation
- ✅ Automatic protocol updates

See `MIGRATION.md` for migration guide.

## Protocol Compatibility

NearSend implements the LocalSend Protocol v2.0:

- **Discovery**: Uses UDP multicast on port 53317 with address 224.0.0.167
- **Communication**: REST API over HTTP/HTTPS
- **Endpoints**:
  - `GET /api/info` - Get device information
  - `POST /api/transfer` - Initiate a transfer
  - `POST /api/transfer/:transfer_id` - Accept/reject a transfer
  - `GET /api/transfer/:transfer_id/file/:file_id` - Download a file

## Building

```bash
cargo build --release
```

## Usage

1. Start the application
2. The app will automatically start:
   - Broadcasting device presence via UDP multicast
   - Listening for nearby devices
   - Starting an HTTP server on port 53317
3. Select files to send
4. Choose a device from the discovered devices list
5. Send files or messages

## Network Requirements

- Devices must be on the same local network
- Firewall must allow:
  - **Incoming**: TCP/UDP on port 53317
  - **Outgoing**: TCP/UDP on any port
- AP isolation must be disabled on your router

## Dependencies

- **GPUI**: UI framework
- **gpui-component**: Component library
- **localsend**: Official LocalSend core crate (from https://github.com/localsend/localsend.git)
- **Tokio**: Async runtime
- **Serde**: Serialization

### Optional (if not using localsend-rs)

- **Axum**: HTTP server framework
- **Reqwest**: HTTP client

## Status

This is a work in progress. Current implementation includes:

- ✅ Protocol definitions
- ✅ Device discovery
- ✅ HTTP server and client
- ✅ Basic UI framework
- ⏳ File picker integration
- ⏳ File upload/download with progress
- ⏳ Transfer management UI
- ⏳ HTTPS/TLS support

## License

[Add your license here]

## References

- [LocalSend Protocol](https://github.com/localsend/protocol)
- [LocalSend Application](https://github.com/localsend/localsend)
