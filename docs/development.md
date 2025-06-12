# TCPGraph Development Guide

## Project Structure

```
tcpgraph/
├── src/
│   ├── main.rs              # Application entry point and orchestration
│   ├── cli.rs               # Command-line argument parsing (clap)
│   ├── capture.rs           # Packet capture and direction detection (pcap, pnet)
│   ├── bandwidth.rs         # Bandwidth calculation and smoothing
│   ├── ui.rs               # Terminal UI and graph rendering (ratatui)
│   └── lib.rs              # Library interface
├── tests/
│   └── integration_tests.rs # Integration tests
├── docs/                    # Documentation
├── Cargo.toml              # Dependencies and metadata
└── README.md               # User documentation
```

## Dependencies

### Core Dependencies
```toml
[dependencies]
pcap = "1.3"                # Network packet capture
clap = { version = "4.4", features = ["derive"] }  # CLI parsing
ratatui = "0.26"            # Terminal UI framework
crossterm = "0.28"          # Cross-platform terminal manipulation
tokio = { version = "1.0", features = ["full"] }   # Async runtime
anyhow = "1.0"              # Error handling
tokio-util = "0.7"          # Additional tokio utilities
pnet = "0.35"               # Network packet parsing
```

### Development Dependencies
- Standard Rust testing framework
- No additional dev dependencies currently

## Build Requirements

### System Dependencies
- **Linux**: `libpcap-dev` package
- **macOS**: libpcap (usually pre-installed)
- **Windows**: WinPcap or Npcap

### Installation Commands
```bash
# Ubuntu/Debian
sudo apt install libpcap-dev

# CentOS/RHEL
sudo yum install libpcap-devel

# macOS
brew install libpcap  # Usually not needed

# Arch Linux
sudo pacman -S libpcap
```

## Building

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Running with Permissions
```bash
# Method 1: Use sudo
sudo ./target/release/tcpgraph -i eth0 -f "tcp"

# Method 2: Set capabilities (Linux only)
sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/tcpgraph
./target/release/tcpgraph -i eth0 -f "tcp"
```

## Code Architecture

### Threading Model

#### Main Thread
- UI rendering with ratatui
- User input handling
- Application lifecycle management

#### Blocking Thread (Packet Capture)
```rust
task::spawn_blocking(move || {
    Self::capture_packets(interface, filter, payload_only, tx)
});
```
- Packet capture using pcap (blocking operations)
- MAC address-based direction detection
- Payload size calculation

#### Async Task (Bandwidth Calculation)
```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(update_interval);
    loop {
        interval.tick().await;
        // Process packets and calculate bandwidth
    }
});
```
- Periodic bandwidth calculation
- Moving average smoothing
- Data aggregation

### Data Flow

```rust
// Channel types
mpsc::Receiver<PacketInfo>           // Capture -> Bandwidth Calculator
mpsc::Receiver<DirectionalBandwidth> // Bandwidth Calculator -> UI
```

### Error Handling Strategy

#### Context-Rich Errors
```rust
use anyhow::{Context, Result};

packet_capture.start_capture().await
    .context("Failed to start packet capture")?;
```

#### Early Validation
```rust
fn validate_args(args: &Args) -> Result<()> {
    validate_interface(&args.interface)?;
    // ... other validations
}
```

#### Graceful Degradation
```rust
// Fallback to full packet size if header parsing fails
let size = if payload_only {
    Self::get_payload_size(&packet.data)
} else {
    packet.header.caplen
};
```

## Key Algorithms

### MAC Address Direction Detection

```rust
fn determine_direction(packet_data: &[u8], local_macs: &HashSet<MacAddr>) -> TrafficDirection {
    if let Some(eth_packet) = EthernetPacket::new(packet_data) {
        let src_mac = eth_packet.get_source();
        let dst_mac = eth_packet.get_destination();
        
        let src_is_local = local_macs.contains(&src_mac);
        let dst_is_local = local_macs.contains(&dst_mac);
        
        match (src_is_local, dst_is_local) {
            (true, false) => TrafficDirection::Outbound,
            (false, true) => TrafficDirection::Inbound,
            _ => TrafficDirection::Unknown,
        }
    }
}
```

### Payload Size Extraction

```rust
fn get_payload_size(packet_data: &[u8]) -> u32 {
    // Parse Ethernet -> IP -> TCP headers
    // Return only application data size
    if let Some(eth_packet) = EthernetPacket::new(packet_data) {
        match eth_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                // Calculate: total_length - ip_header - tcp_header
            }
            EtherTypes::Ipv6 => {
                // Calculate: payload_length - tcp_header  
            }
        }
    }
    // Fallback to full packet size
}
```

### Bandwidth Smoothing

```rust
fn calculate_bandwidth(&mut self) -> DirectionalBandwidth {
    // Calculate raw bandwidth
    let raw_bandwidth = DirectionalBandwidth { inbound_bps, outbound_bps };
    
    // Add to smoothing buffer
    self.smoothing_buffer.push_back(raw_bandwidth.clone());
    if self.smoothing_buffer.len() > self.smoothing_samples {
        self.smoothing_buffer.pop_front();
    }
    
    // Return moving average
    let smoothed_inbound = self.smoothing_buffer.iter()
        .map(|b| b.inbound)
        .sum::<f64>() / self.smoothing_buffer.len() as f64;
}
```

## Testing Strategy

### Unit Tests
Located in `tests/integration_tests.rs`:
- Bandwidth calculator functionality
- Edge cases (empty data, single packets, multiple packets)
- History management

### Manual Testing
```bash
# Test interface validation
cargo run -- -i nonexistent -f "tcp"

# Test help output
cargo run -- --help

# Test with minimal traffic
sudo cargo run -- -i lo -f "tcp"
```

### Integration Testing with Real Traffic
```bash
# Generate test traffic
ping google.com &
curl http://example.com &

# Monitor with tcpgraph
sudo cargo run -- -i wlan0 -f "icmp or tcp port 80"
```

## Performance Considerations

### Memory Management
- Bounded packet buffer (automatic cleanup)
- Limited bandwidth history (5 minutes)
- Efficient packet parsing (zero-copy where possible)

### CPU Optimization
- Configurable update intervals
- Selective packet filtering at pcap level
- Minimal string allocations in hot paths

### Network Performance
- Raw socket access for minimal overhead
- No deep packet inspection unless needed
- Efficient MAC address lookups (HashSet)

## Common Development Tasks

### Adding New CLI Options

1. **Update cli.rs**:
```rust
#[arg(long, help = "Description")]
pub new_option: bool,
```

2. **Update validation**:
```rust
fn validate_args(args: &Args) -> Result<()> {
    // Add validation for new option
}
```

3. **Pass through modules**:
```rust
// Update constructors and function calls
```

### Adding New Packet Parsing

1. **Extend get_payload_size()**:
```rust
match eth_packet.get_ethertype() {
    EtherTypes::NewProtocol => {
        // Add parsing logic
    }
}
```

2. **Add tests**:
```rust
#[test]
fn test_new_protocol_parsing() {
    // Test the new parsing logic
}
```

### Modifying UI Layout

1. **Update ui.rs layout**:
```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        // Modify constraints
    ])
    .split(f.size());
```

2. **Add new widgets**:
```rust
let new_widget = Paragraph::new("content")
    .block(Block::default().borders(Borders::ALL));
f.render_widget(new_widget, chunks[n]);
```

## Debugging Tips

### Packet Capture Issues
```bash
# Test with tcpdump first
sudo tcpdump -i eth0 tcp

# Check interface permissions
ls -la /dev/net/tun
```

### Direction Detection Issues
```bash
# Check interface MAC addresses
ip link show
cat /sys/class/net/eth0/address

# Test with known traffic
ping google.com  # Should show outbound
```

### Performance Profiling
```bash
# Build with debug symbols
cargo build --release

# Profile with perf (Linux)
sudo perf record ./target/release/tcpgraph -i eth0 -f "tcp"
perf report
```

## Code Style Guidelines

### Rust Conventions
- Follow standard Rust naming conventions
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` for linting

### Error Handling
- Prefer `anyhow::Result` for application errors
- Add context to errors: `.context("What failed")?`
- Validate inputs early and provide helpful messages

### Documentation
- Document public APIs with `///` comments
- Include examples in doc comments
- Update README.md for user-facing changes

### Testing
- Write tests for new functionality
- Include edge cases and error conditions
- Test with real network traffic when possible

## Release Process

### Version Bumping
1. Update `Cargo.toml` version
2. Update README.md if needed
3. Create git tag: `git tag v0.2.0`

### Testing Checklist
- [ ] Unit tests pass: `cargo test`
- [ ] Builds on target platforms
- [ ] Manual testing with real interfaces
- [ ] Help output is correct
- [ ] Interface validation works
- [ ] Permission handling works

### Documentation Updates
- [ ] Update README.md
- [ ] Update docs/ folder
- [ ] Update code comments
- [ ] Update changelog (if maintained)

## Future Development Ideas

### High Priority
1. **VLAN Support**: Parse 802.1Q tags
2. **IPv6 Improvements**: Better IPv6 header handling
3. **Export Features**: CSV/JSON data export
4. **Configuration File**: Save common settings

### Medium Priority
1. **Advanced Filtering**: GUI filter builder
2. **Historical Data**: Long-term storage and analysis
3. **Alerting**: Threshold-based notifications
4. **Protocol Analysis**: Application-layer insights

### Low Priority
1. **Web Interface**: Browser-based monitoring
2. **Clustering**: Multi-host monitoring
3. **Machine Learning**: Anomaly detection
4. **Plugin System**: Extensible analysis modules