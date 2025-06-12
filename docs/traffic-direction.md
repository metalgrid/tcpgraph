# Traffic Direction Detection

## Overview

TCPGraph uses MAC address-based traffic direction detection for accurate bidirectional bandwidth monitoring, especially in complex network scenarios like routers and multi-interface systems.

## Evolution of Direction Detection

### Initial Approach: IP-Based Detection
**Problems**:
- Failed on routers where traffic neither originates nor terminates locally
- Ambiguous for multi-homed hosts
- Didn't handle the "any" interface properly

### Current Approach: MAC Address Analysis
**Benefits**:
- Works correctly on routers, firewalls, and bridge devices
- Accurate for broadcast/multicast traffic
- Proper handling of forwarded traffic

## MAC Address Detection Logic

### Basic Algorithm
```rust
let src_is_local = local_macs.contains(&src_mac);
let dst_is_local = local_macs.contains(&dst_mac);

match (src_is_local, dst_is_local) {
    (true, false) => Outbound,   // Our interface is sending
    (false, true) => Inbound,    // Our interface is receiving
    _ => Unknown,                // Transit or internal traffic
}
```

### MAC Address Collection
```rust
fn get_local_macs(interface_name: &str) -> HashSet<MacAddr> {
    let interfaces = datalink::interfaces();
    
    for iface in interfaces {
        if interface_name == "any" || iface.name == interface_name {
            if let Some(mac) = iface.mac {
                local_macs.insert(mac);
            }
        }
    }
}
```

## Scenario Handling

### 1. Standard Host Monitoring
**Setup**: Laptop/desktop with single active interface
```
Interface: wlan0 (MAC: AA:BB:CC:DD:EE:FF)

Packet 1: src=AA:BB:CC:DD:EE:FF dst=11:22:33:44:55:66 → Outbound
Packet 2: src=11:22:33:44:55:66 dst=AA:BB:CC:DD:EE:FF → Inbound
```

### 2. Router/Firewall Monitoring
**Setup**: Router with WAN interface eth0
```
Interface: eth0 (MAC: AA:BB:CC:DD:EE:FF)

Packet 1: src=AA:BB:CC:DD:EE:FF dst=11:22:33:44:55:66 → Outbound (to internet)
Packet 2: src=11:22:33:44:55:66 dst=AA:BB:CC:DD:EE:FF → Inbound (from internet)
Packet 3: src=22:33:44:55:66:77 dst=33:44:55:66:77:88 → Unknown (forwarded)
```

**Traffic Interpretation**:
- **Outbound**: Router-generated traffic (management, updates)
- **Inbound**: Traffic destined for router (SSH, web interface)  
- **Unknown**: Client traffic being routed/forwarded

### 3. Multi-Interface ("any") Monitoring
**Setup**: System with multiple interfaces
```
Interfaces: 
- eth0 (MAC: AA:BB:CC:DD:EE:FF)
- wlan0 (MAC: 11:22:33:44:55:66)
- docker0 (MAC: 22:33:44:55:66:77)

Packet Analysis:
- src=AA:BB:CC:DD:EE:FF → Outbound (ethernet)
- dst=11:22:33:44:55:66 → Inbound (wireless)
- src=22:33:44:55:66:77 → Outbound (docker)
```

## Edge Cases and Special Handling

### Broadcast Traffic
```rust
let is_broadcast = dst_mac == MacAddr::broadcast();
let is_multicast = dst_mac.is_multicast();

match (src_is_local, is_broadcast, is_multicast) {
    (true, true, _) | (true, _, true) => Outbound,  // We're broadcasting
    (false, true, _) | (false, _, true) => Inbound, // Receiving broadcast
    // ... normal logic for unicast
}
```

**Examples**:
- ARP requests from our interface → Outbound
- DHCP offers to broadcast → Inbound
- Multicast DNS from our interface → Outbound

### VLAN Tagged Traffic
**Current Limitation**: VLAN tags not parsed, treated as normal Ethernet frames.

**Future Enhancement**: Parse 802.1Q headers before determining direction.

### Bridge/Switch Scenarios
**Problem**: Bridge interfaces may see traffic with multiple local MACs.

**Solution**: Include all bridge member interfaces in MAC collection.

## Interface-Specific Behavior

### Standard Interface (e.g., eth0, wlan0)
```bash
tcpgraph -i eth0 -f "tcp"
```
- Uses only eth0's MAC address
- Clean separation of interface-specific traffic
- Ideal for interface-specific monitoring

### "any" Pseudo-Interface
```bash
tcpgraph -i any -f "tcp"
```
- Aggregates MACs from all active interfaces
- Shows system-wide traffic patterns
- Useful for comprehensive monitoring

### Loopback Interface
```bash
tcpgraph -i lo -f "tcp"
```
- Both source and destination are local
- All traffic typically classified as "Unknown"
- Useful for monitoring inter-process communication

## Router Use Cases

### WAN Interface Monitoring
```bash
# Monitor internet-facing traffic
tcpgraph -i eth0 -f "ip"
```
**Interpretation**:
- **Inbound**: Traffic from internet to router/LAN
- **Outbound**: Traffic from router/LAN to internet
- **Unknown**: Routed traffic (counted as transit)

### LAN Interface Monitoring
```bash
# Monitor local network traffic
tcpgraph -i br0 -f "tcp"
```
**Interpretation**:
- **Inbound**: Traffic from LAN devices to router
- **Outbound**: Traffic from router to LAN devices
- **Unknown**: Inter-device traffic being bridged

### Complete Router View
```bash
# Monitor all router interfaces
tcpgraph -i any -f "ip"
```
**Benefits**:
- See total router throughput
- Understand traffic distribution across interfaces
- Monitor both routing and local traffic

## Troubleshooting Direction Detection

### Common Issues

1. **All Traffic Shows as Unknown**
   - **Cause**: Interface MAC not detected properly
   - **Solution**: Check interface is up and has assigned MAC

2. **Incorrect Direction Classification**
   - **Cause**: Monitoring wrong interface
   - **Solution**: Verify interface name with `ip link show`

3. **Missing Traffic in "any" Mode**
   - **Cause**: Some interfaces not included
   - **Solution**: Check interface flags and permissions

### Debugging Commands
```bash
# List available interfaces with MACs
ip link show

# Verify interface has MAC address
cat /sys/class/net/eth0/address

# Test with specific interface first
tcpgraph -i eth0 -f "icmp"  # ping traffic for testing
```

## Performance Considerations

### MAC Lookup Efficiency
- HashSet provides O(1) MAC address lookups
- Minimal performance impact even with many interfaces

### Packet Processing Overhead
- MAC extraction from Ethernet header is fast
- No deep packet inspection required for direction detection

### Memory Usage
- Small memory footprint (6 bytes per MAC address)
- Bounded by number of network interfaces on system