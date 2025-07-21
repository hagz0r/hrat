# Remote Access Tool (RAT) Client

This is the client component of a remote access tool that connects to a control server via WebSocket.

## Features

- **Remote Command Execution**: Execute shell commands on the target machine
- **File System Access**: Browse, download, upload, and manage files
- **Screen Capture**: Take screenshots and stream desktop video
- **Webcam Access**: Capture photos and stream video from webcam
- **System Information**: Collect detailed system specifications
- **Secure Communication**: Encrypted WebSocket connections with TLS support

## Configuration

The client uses environment variables for configuration:

### Connection Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `RAT_HOST_IP` | `127.0.0.1` | IP address of the control server |
| `RAT_HOST_PORT` | `8080` | Port number of the control server |
| `RAT_USE_TLS` | `true` | Enable TLS encryption for WebSocket connections |

### TLS/Encryption Settings

- **`RAT_USE_TLS=true`** (default): Uses secure WebSocket connections (`wss://`) with TLS encryption
- **`RAT_USE_TLS=false`**: Uses plain WebSocket connections (`ws://`) without encryption

**Security Note**: It's strongly recommended to keep TLS enabled (`RAT_USE_TLS=true`) for production use to ensure encrypted communication between client and server.

## Usage

### Basic Usage
```bash
cargo run --release
```

### Custom Configuration
```bash
# Connect to remote server with TLS
RAT_HOST_IP=192.168.1.100 RAT_HOST_PORT=9443 RAT_USE_TLS=true cargo run --release

# Connect to local development server without TLS
RAT_HOST_IP=localhost RAT_HOST_PORT=8080 RAT_USE_TLS=false cargo run --release
```

### Development Mode
```bash
# Enable debug logging
cargo run --features dev-logs
```

## Supported Modules

The client supports the following command modules:

- **RSH** - Remote Shell execution
- **FS** - File System operations
- **RS** - Remote Screen capture/streaming
- **WC** - Webcam capture/streaming
- **TM** - Task Manager (process management)
- **KL** - Keylogger
- **TRL** - Trolling/pranks
- **CH** - Chat functionality
- **AD** - Audio recording
- **RCE** - Remote Code Execution

## Build Options

### Release Build (Optimized)
```bash
cargo build --release
```

The release profile includes aggressive optimizations:
- Strip debug symbols
- Link-time optimization (LTO)
- Single codegen unit
- Panic abort (no unwinding)

### Development Build
```bash
cargo build --features dev-logs
```

## Security Considerations

1. **TLS Encryption**: Always use TLS in production (`RAT_USE_TLS=true`)
2. **Network Security**: Ensure the control server uses proper authentication
3. **Firewall**: Configure appropriate firewall rules for the connection
4. **Certificate Validation**: The client validates server certificates using system root CAs

## Dependencies

Key dependencies include:
- `tokio-tungstenite`: WebSocket client with TLS support
- `nokhwa`: Cross-platform camera access
- `scap`: Screen capture functionality
- `sysinfo`: System information gathering
- `image`: Image processing and compression

## Platform Support

- Windows (primary target)
- Linux (partial support)
- macOS (partial support)

## Troubleshooting

### Connection Issues
- Verify server is running and accessible
- Check firewall settings
- Ensure correct IP and port configuration
- For TLS issues, verify server certificate is valid

### TLS/SSL Errors
- Ensure server supports TLS if `RAT_USE_TLS=true`
- Check system date/time accuracy
- Verify certificate authority is trusted by the system

### Permission Issues
- Some features may require elevated privileges
- Screen capture may need accessibility permissions on macOS
- Camera access requires appropriate system permissions