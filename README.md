# TCPGraph

A terminal-based network bandwidth monitor that captures packets using pcap filters and displays real-time bandwidth graphs.

## Features

- Real-time packet capture with pcap filtering
- Live bandwidth visualization in the terminal with grid lines
- Intelligent speed bucket scaling (auto-adjusts from 0-10 Mbps to 1000+ Mbps)
- Speed category indicators (Very Slow, Slow, Basic, Fast, Very Fast, Superfast, Ultra Fast)
- Configurable update intervals 
- Supports any network interface
- Graceful shutdown with Ctrl+C
- Cross-platform terminal UI

## Installation

### Prerequisites

- Rust (1.70 or later)
- libpcap development libraries:
  - **Ubuntu/Debian**: `sudo apt install libpcap-dev`
  - **CentOS/RHEL**: `sudo yum install libpcap-devel`
  - **macOS**: `brew install libpcap` (usually pre-installed)
  - **Windows**: WinPcap or Npcap

### Build from source

```bash
git clone <repository-url>
cd tcpgraph
cargo build --release
```

## Usage

### Basic Usage

```bash
tcpgraph --interface eth0 --filter "tcp port 80"
```

### Options

- `-i, --interface <INTERFACE>`: Network interface to monitor (required)
- `-f, --filter <FILTER>`: PCAP filter expression (required)
- `--interval <SECONDS>`: Graph update interval in seconds (default: 1)
- `--duration <SECONDS>`: Total monitoring duration in seconds (optional)

### Examples

**Monitor HTTP traffic on eth0:**
```bash
tcpgraph -i eth0 -f "tcp port 80"
```

**Monitor all TCP traffic with 2-second updates:**
```bash
tcpgraph -i wlan0 -f "tcp" --interval 2
```

**Monitor SSH traffic for 60 seconds:**
```bash
tcpgraph -i eth0 -f "tcp port 22" --duration 60
```

**Monitor specific host traffic:**
```bash
tcpgraph -i eth0 -f "host 192.168.1.100"
```

**Monitor DNS queries:**
```bash
tcpgraph -i eth0 -f "udp port 53"
```

### PCAP Filter Examples

- `tcp`: All TCP traffic
- `udp`: All UDP traffic  
- `tcp port 80`: HTTP traffic
- `tcp port 443`: HTTPS traffic
- `host 192.168.1.1`: Traffic to/from specific IP
- `net 192.168.1.0/24`: Traffic to/from subnet
- `tcp and port 80`: TCP traffic on port 80
- `not tcp`: All non-TCP traffic

## Interface

The application displays:
- **Top panel**: Interface name and current filter
- **Middle panel**: Real-time bandwidth graph (Mbps over time) with intelligent scaling and grid lines
- **Bottom panel**: Current bandwidth with speed category, maximum recorded bandwidth

### Speed Categories
- **Very Slow**: < 1 Mbps (Red)
- **Slow**: 1-10 Mbps (Yellow) 
- **Basic**: 10-25 Mbps (Blue)
- **Fast**: 25-50 Mbps (Green)
- **Very Fast**: 50-100 Mbps (Cyan)
- **Superfast**: 100-250 Mbps (Magenta)
- **Ultra Fast**: 250+ Mbps (White)

### Controls

- **q** or **Esc**: Quit the application
- **Ctrl+C**: Graceful shutdown

## Permissions

Network packet capture typically requires elevated privileges:

```bash
# Run with sudo (Linux/macOS)
sudo tcpgraph -i eth0 -f "tcp"

# Or add capabilities (Linux only)
sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/tcpgraph
```

## Troubleshooting

### Common Issues

**"Permission denied" errors:**
- Run with `sudo` or set appropriate capabilities
- Ensure your user has access to network interfaces

**"Interface not found" errors:**
- List available interfaces: `ip link show` (Linux) or `ifconfig` (macOS)
- Use the exact interface name (e.g., `eth0`, `wlan0`, `en0`)

**"Failed to set packet filter" errors:**
- Verify your pcap filter syntax
- Test filters with tools like `tcpdump` first

### Performance

- Higher update intervals (--interval) reduce CPU usage
- Complex filters may impact performance
- Large amounts of traffic may require filtering to specific ports/hosts

## License

This project is licensed under the MIT License.