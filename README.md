# hrat - Hagz0r's Remote Access Tool

![Project Status: Alpha](https://img.shields.io/badge/status-alpha-red)
![Cross-Platform](https://img.shields.io/badge/platform-Windows%20|%20Linux%20|%20macOS-blue)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A cross-platform remote access tool with advanced monitoring and control capabilities. Designed for ethical hacking and remote system management.

## Key Features

### Core Capabilities

- **Cross-Platform Operation**: Full support for Windows, Linux, and macOS
- **Secure Communication**: TLS-encrypted WebSocket connections
- **Polymorphic Techniques**: AV evasion through code obfuscation and payload variation

### Remote Monitoring

- **Screen Capture**: Real-time desktop streaming & screenshot capture
- **Webcam Access**: Photo capture and video streaming
- **Process Management**: Live process monitoring and termination
- **Keylogging**: Keystroke capture and analysis
- **Audio Monitoring**: Microphone recording capabilities

### System Interaction

- **Remote Shell**: Full terminal access with PowerShell/Bash support
- **File System Explorer**:
  - File upload/download
  - Directory traversal
  - File execution
- **Registry Editing** (Windows): Remote registry manipulation

### Data Collection

- **Credential Harvesting**:
  - Browser password extraction
  - Cookie database retrieval
  - Credit card data capture
- **System Intelligence**:
  - Hardware inventory
  - Network configuration
  - Geolocation tracking

### Additional Modules

- **Chat System**: Bidirectional communication channel
- **Trolling Toolkit**:
  - Fake error messages
  - Website redirection
  - Clipboard manipulation
- **Persistence Mechanisms**: Startup registration and service installation

## Web Interface

![Web Interface Preview](https://i.ibb.co/FLHbkPt6/chrome-o-Vg9fw58-UA.png)
![All Functions menu preview](https://i.ibb.co/TMRBmQX4/chrome-yjb-FLxp3i6.png)
![Builder preview](https://i.ibb.co/r9ZVm8N/chrome-D0pe-Qev-UAq.png)
Showing Super and Client builder pages,

Access through `https://[SERVER_IP]:[SERVER_PORT]` featuring:

- Real-time system dashboards
- Interactive file browser
- Live screen/view control
- Chat interface
- Task management console

## Installation

### Server Setup

```bash
# Clone repository
git clone https://github.com/hagz0r/hrat.git

# Install dependencies
pip install -r requirements.txt

# Start server
uvicorn main:app --host 0.0.0.0 --port 8443 --ssl-keyfile key.pem --ssl-certfile cert.pem
```

### Client Deployment

```bash
# Build optimized binary
cargo build --release

# Run with custom configuration
RAT_HOST_IP=your.server.ip RAT_USE_TLS=true ./target/release/client
```

## Configuration

### Essential Environment Variables

| Variable        | Default   | Description               |
| --------------- | --------- | ------------------------- |
| `RAT_HOST_IP`   | 127.0.0.1 | Control server IP address |
| `RAT_HOST_PORT` | 8443      | Server port number        |
| `RAT_USE_TLS`   | true      | Enable TLS encryption     |

### Advanced Options

```bash
# Debug mode with verbose logging
RAT_DEBUG=1 cargo run --features dev-logs

# Custom TLS certificates
RAT_CA_BUNDLE=/path/to/ca.pem cargo run
```

## Security Considerations

⚠️ **Ethical Use Warning**: This tool should only be used on systems you own or have explicit permission to access.

- All network communication uses TLS 1.3 encryption
- Certificate pinning for server authentication
- Memory encryption for sensitive operations
- Automatic connection obfuscation techniques

## Contributing

We welcome contributions! Please see our [Contribution Guidelines](CONTRIBUTING.md) for details.

## License

MIT License
