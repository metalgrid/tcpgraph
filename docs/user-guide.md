# TCPGraph User Guide

## Quick Start

### Basic Usage
```bash
# Monitor HTTP traffic on ethernet interface
sudo tcpgraph -i eth0 -f "tcp port 80"

# Monitor all TCP traffic on wireless interface  
sudo tcpgraph -i wlan0 -f "tcp"

# Monitor all interfaces simultaneously
sudo tcpgraph -i any -f "tcp"
```

### Why sudo?
Network packet capture requires elevated privileges to access raw network interfaces.

## Command Line Options

### Required Arguments
- `-i, --interface <INTERFACE>`: Network interface to monitor
- `-f, --filter <FILTER>`: PCAP filter expression

### Optional Arguments
- `--interval <SECONDS>`: Graph update interval (default: 1)
- `--duration <SECONDS>`: Total monitoring duration
- `--payload-only`: Count only payload data (more accurate for speed comparisons)
- `--smoothing <N>`: Number of samples for smoothing (default: 3)

## Interface Selection

### Finding Available Interfaces
If you specify an invalid interface, tcpgraph will list all available options:
```bash
$ tcpgraph -i invalid -f "tcp"
Error: Interface 'invalid' not found.

Available interfaces:
  - any (Pseudo-device that captures on all interfaces)
  - eth0 (Ethernet)
  - wlan0 (Wireless)
  - lo (Loopback)
```

### Common Interface Names
- **Linux**: eth0, eth1, wlan0, ens33, enp0s3
- **macOS**: en0, en1, en2
- **Windows**: varies, check with interface listing

### Special Interfaces
- **any**: Monitors all interfaces simultaneously
- **lo**: Loopback interface for inter-process communication
- **docker0**: Docker bridge interface
- **br0**: Bridge interfaces

## PCAP Filter Examples

### Protocol Filters
```bash
# All TCP traffic
tcpgraph -i eth0 -f "tcp"

# All UDP traffic  
tcpgraph -i eth0 -f "udp"

# All IP traffic
tcpgraph -i eth0 -f "ip"
```

### Port-Based Filters
```bash
# HTTP traffic
tcpgraph -i eth0 -f "tcp port 80"

# HTTPS traffic
tcpgraph -i eth0 -f "tcp port 443"

# SSH traffic
tcpgraph -i eth0 -f "tcp port 22"

# DNS traffic
tcpgraph -i eth0 -f "udp port 53"

# Multiple ports
tcpgraph -i eth0 -f "tcp port 80 or tcp port 443"
```

### Host-Based Filters
```bash
# Traffic to/from specific host
tcpgraph -i eth0 -f "host 192.168.1.100"

# Traffic to/from subnet
tcpgraph -i eth0 -f "net 192.168.1.0/24"

# Outbound traffic only
tcpgraph -i eth0 -f "src host 192.168.1.100"

# Inbound traffic only  
tcpgraph -i eth0 -f "dst host 192.168.1.100"
```

### Complex Filters
```bash
# HTTP/HTTPS traffic to specific host
tcpgraph -i eth0 -f "host example.com and (port 80 or port 443)"

# All traffic except SSH
tcpgraph -i eth0 -f "not port 22"

# Large packets only
tcpgraph -i eth0 -f "tcp and greater 1000"
```

## Usage Scenarios

### Speed Test Comparison
For results comparable to Ookla speedtest:
```bash
tcpgraph -i eth0 -f "tcp" --payload-only --smoothing 5
```

### Network Troubleshooting
Monitor specific services:
```bash
# Web server monitoring
tcpgraph -i eth0 -f "tcp port 80 or tcp port 443"

# Database monitoring
tcpgraph -i eth0 -f "tcp port 3306"  # MySQL
tcpgraph -i eth0 -f "tcp port 5432"  # PostgreSQL
```

### Router/Firewall Monitoring
```bash
# WAN interface monitoring
tcpgraph -i eth0 -f "ip"

# LAN interface monitoring  
tcpgraph -i br0 -f "tcp"

# Complete router view
tcpgraph -i any -f "ip"
```

### Development/Testing
```bash
# Monitor localhost traffic
tcpgraph -i lo -f "tcp"

# Monitor Docker container traffic
tcpgraph -i docker0 -f "tcp"

# Monitor specific application port
tcpgraph -i eth0 -f "tcp port 8080"
```

## Understanding the Interface

### Graph Display
- **Green line**: Inbound traffic (downloads/received data)
- **Red line**: Outbound traffic (uploads/sent data)
- **X-axis**: Time (last 100 seconds)
- **Y-axis**: Bandwidth in Mbps with intelligent scaling

### Status Information
- **↓ In**: Current inbound speed
- **↑ Out**: Current outbound speed  
- **Max**: Maximum recorded speeds for each direction

### Controls
- **q** or **Esc**: Quit application
- **Ctrl+C**: Graceful shutdown

## Bandwidth Calculation Modes

### Standard Mode (Default)
```bash
tcpgraph -i eth0 -f "tcp"
```
- Counts entire Ethernet frames including headers
- Shows total network utilization
- Typically 10-20% higher than application-layer measurements

### Payload-Only Mode
```bash
tcpgraph -i eth0 -f "tcp" --payload-only
```
- Strips protocol headers (Ethernet, IP, TCP)
- Counts only application data
- More comparable to speed test tools
- Better for measuring actual data transfer rates

### Smoothing Options
```bash
# Responsive but potentially spiky
tcpgraph -i eth0 -f "tcp" --smoothing 1

# Balanced (default)
tcpgraph -i eth0 -f "tcp" --smoothing 3

# Smooth but less responsive
tcpgraph -i eth0 -f "tcp" --smoothing 10
```

## Common Use Cases

### 1. Internet Speed Monitoring
```bash
# Compare with speed test results
sudo tcpgraph -i wlan0 -f "tcp" --payload-only --smoothing 5

# Monitor during large downloads
sudo tcpgraph -i eth0 -f "tcp port 80 or tcp port 443"
```

### 2. Server Monitoring
```bash
# Web server load
sudo tcpgraph -i eth0 -f "tcp port 80 or tcp port 443"

# Database traffic
sudo tcpgraph -i eth0 -f "tcp port 3306"

# All server traffic
sudo tcpgraph -i eth0 -f "tcp and dst port 1024:65535"
```

### 3. Network Diagnostics
```bash
# Identify bandwidth hogs
sudo tcpgraph -i any -f "ip"

# Monitor specific protocols
sudo tcpgraph -i eth0 -f "udp"  # For streaming, VoIP

# Check for unusual traffic
sudo tcpgraph -i eth0 -f "not (tcp or udp)"
```

### 4. Development Testing
```bash
# API testing
sudo tcpgraph -i lo -f "tcp port 8080"

# Microservices communication
sudo tcpgraph -i docker0 -f "tcp"

# Load testing monitoring
sudo tcpgraph -i eth0 -f "tcp port 80" --smoothing 1
```

## Troubleshooting

### Permission Errors
```bash
# Error: Permission denied
# Solution: Run with sudo
sudo tcpgraph -i eth0 -f "tcp"

# Or set capabilities (Linux only)
sudo setcap cap_net_raw,cap_net_admin=eip ./tcpgraph
```

### Interface Not Found
```bash
# Error: Interface 'eth1' not found
# Solution: List available interfaces
tcpgraph -i nonexistent -f "tcp"  # Will show available interfaces

# Or use system commands
ip link show        # Linux
ifconfig           # macOS
```

### No Traffic Visible
**Common Causes**:
1. **Wrong interface**: Use `ip link show` to verify
2. **No matching traffic**: Broaden filter (try "ip" instead of "tcp")
3. **Permission issues**: Ensure running with sudo
4. **Interface down**: Check interface status

**Solutions**:
```bash
# Test with broad filter
sudo tcpgraph -i any -f "ip"

# Test with ping traffic
sudo tcpgraph -i eth0 -f "icmp"  # Then ping something

# Check interface status
ip link show eth0
```

### Unexpected Readings
**For speed test comparison**:
```bash
# Use payload-only mode with smoothing
sudo tcpgraph -i eth0 -f "tcp" --payload-only --smoothing 5
```

**For network utilization**:
```bash
# Use standard mode
sudo tcpgraph -i eth0 -f "ip"
```

## Performance Tips

### Reducing CPU Usage
```bash
# Increase update interval
tcpgraph -i eth0 -f "tcp" --interval 2

# Use more specific filters
tcpgraph -i eth0 -f "tcp port 80"  # Instead of "ip"
```

### Monitoring High-Speed Links
```bash
# Reduce smoothing for responsiveness
tcpgraph -i eth0 -f "tcp" --smoothing 1

# Use payload-only to reduce processing
tcpgraph -i eth0 -f "tcp" --payload-only
```

### Long-Term Monitoring
```bash
# Higher smoothing for stability
tcpgraph -i eth0 -f "tcp" --smoothing 10 --interval 5
```