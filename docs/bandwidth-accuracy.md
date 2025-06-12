# Bandwidth Calculation Accuracy

## Problem Statement

When comparing tcpgraph with commercial speed testing tools like Ookla speedtest, users observed:
- **10-20% higher readings** in tcpgraph
- **Spikes and drops** instead of stable measurements
- **Inconsistent results** compared to established benchmarks

## Root Cause Analysis

### 1. Header Overhead (Primary Cause)

**Issue**: tcpgraph counted entire Ethernet frames including protocol headers.

**Breakdown of Overhead**:
```
┌─────────────────┬─────────────────┬─────────────────┬─────────────────┐
│ Ethernet Header │    IP Header    │   TCP Header    │   Application   │
│    14 bytes     │   20-40 bytes   │   20+ bytes     │      Data       │
└─────────────────┴─────────────────┴─────────────────┴─────────────────┘
      ^                                                        ^
      └─── tcpgraph (standard) counts everything ──────────────┘
                                                               │
      └─── Ookla/tcpgraph (payload-only) counts only this ────┘
```

**Impact**: For typical TCP traffic, headers represent 8-15% overhead.

### 2. Measurement Methodology Differences

| Aspect | Ookla Speedtest | tcpgraph (Standard) | tcpgraph (Payload-Only) |
|--------|-----------------|---------------------|-------------------------|
| **Data Counted** | Application payload | Full Ethernet frame | Application payload |
| **Control Traffic** | Excluded | Included (ACKs, etc.) | Minimal impact |
| **Timing Method** | Controlled transfer | Real-time packet capture | Real-time packet capture |
| **Smoothing** | Built-in averaging | Raw per-second | Configurable moving average |

### 3. Network Stack Behavior

**TCP Acknowledgments**: Standard mode counts ACK packets (typically 40-54 bytes each).

**Retransmissions**: Network layer retransmissions inflate bandwidth readings.

**Window Management**: TCP flow control packets add overhead.

## Solution Implementation

### Payload-Only Mode (`--payload-only`)

**Header Stripping Logic**:
```rust
fn get_payload_size(packet_data: &[u8]) -> u32 {
    // Parse Ethernet frame
    if let Some(eth_packet) = EthernetPacket::new(packet_data) {
        match eth_packet.get_ethertype() {
            EtherTypes::Ipv4 => {
                // Parse IPv4 header
                let ip_header_len = (ipv4.get_header_length() as u32) * 4;
                if protocol == TCP {
                    // Parse TCP header
                    let tcp_header_len = (tcp.get_data_offset() as u32) * 4;
                    return total_length - ip_header_len - tcp_header_len;
                }
            }
            // Similar logic for IPv6...
        }
    }
}
```

### Smoothing Algorithm (`--smoothing N`)

**Moving Average Implementation**:
```rust
// Maintain sliding window of bandwidth samples
smoothing_buffer.push_back(raw_bandwidth);
if smoothing_buffer.len() > smoothing_samples {
    smoothing_buffer.pop_front();
}

// Return average of samples
let smoothed = smoothing_buffer.iter().sum() / smoothing_buffer.len();
```

## Validation Results

### Expected Behavior Comparison

| Scenario | Ookla Reading | tcpgraph Standard | tcpgraph Payload-Only |
|----------|---------------|-------------------|----------------------|
| 150 Mbps Download | 150.0 Mbps | ~165-180 Mbps | ~150-155 Mbps |
| 50 Mbps Upload | 50.0 Mbps | ~55-60 Mbps | ~50-52 Mbps |
| Gigabit Connection | 940 Mbps | ~1050+ Mbps | ~940-960 Mbps |

### Smoothing Effect

| Smoothing Level | Responsiveness | Stability | Use Case |
|-----------------|----------------|-----------|----------|
| `--smoothing 1` | High | Low | Debug/analysis |
| `--smoothing 3` (default) | Medium | Medium | General use |
| `--smoothing 5` | Medium | High | Ookla comparison |
| `--smoothing 10` | Low | Very High | Long-term trends |

## Best Practices

### For Speed Test Comparison
```bash
tcpgraph -i eth0 -f "tcp" --payload-only --smoothing 5
```

### For Network Utilization Analysis
```bash
tcpgraph -i eth0 -f "ip" --smoothing 3
```

### For Router/Firewall Monitoring
```bash
tcpgraph -i any -f "ip" --payload-only
```

## Technical Limitations

### Header Parsing Edge Cases
- **Malformed packets**: Fall back to full packet size
- **Unknown protocols**: Count entire payload after IP header
- **Fragmented packets**: May not parse TCP header correctly

### Router Scenarios
- **Transit traffic**: Unknown direction packets split 50/50
- **VLAN tags**: Not currently parsed (counted as payload)
- **Tunneled traffic**: Inner headers not stripped

## Future Improvements

1. **VLAN Support**: Parse 802.1Q tags
2. **Tunnel Awareness**: Handle GRE, IPSec, etc.
3. **Application Layer**: HTTP/HTTPS content parsing
4. **Advanced Smoothing**: Exponential moving average options
5. **Calibration Mode**: Auto-adjust based on known traffic patterns