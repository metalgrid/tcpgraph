# TCPGraph Architecture

## Overview

TCPGraph is a terminal-based network bandwidth monitor built in Rust that provides real-time visualization of bidirectional traffic using pcap packet capture and ratatui terminal UI.

## Core Components

### 1. CLI Module (`src/cli.rs`)
- **Purpose**: Command-line argument parsing using clap
- **Key Features**:
  - Interface selection (including "any" pseudo-interface)
  - PCAP filter expressions
  - Configurable update intervals
  - Duration limits
  - Bandwidth calculation modes (`--payload-only`, `--smoothing`)

### 2. Packet Capture Module (`src/capture.rs`)
- **Purpose**: Network packet capture and traffic direction detection
- **Key Features**:
  - MAC address-based direction detection
  - Payload size calculation (strips headers)
  - Router-friendly operation
  - Multi-interface support via "any" interface

#### Traffic Direction Logic
```rust
match (src_is_local, dst_is_local) {
    (true, false) => Outbound,   // Our MAC is source -> we're sending
    (false, true) => Inbound,    // Our MAC is dest -> we're receiving  
    _ => Unknown,                // Transit traffic (router scenario)
}
```

### 3. Bandwidth Calculation Module (`src/bandwidth.rs`)
- **Purpose**: Real-time bandwidth calculation with smoothing
- **Key Features**:
  - Bidirectional bandwidth tracking
  - Moving average smoothing
  - Configurable time windows
  - Header-aware payload extraction

#### Calculation Methods
- **Standard Mode**: Counts entire Ethernet frames (includes all headers)
- **Payload-Only Mode**: Strips L2/L3/L4 headers, counts only application data

### 4. Terminal UI Module (`src/ui.rs`)
- **Purpose**: Real-time graph visualization using ratatui
- **Key Features**:
  - Dual-line graphs (green=inbound, red=outbound)
  - Intelligent scaling (10 Mbps to 1000+ Mbps ranges)
  - Real-time statistics display
  - Keyboard controls (q to quit)

## Data Flow

```
Network Interface
       ↓
  Packet Capture (pcap)
       ↓
  MAC-based Direction Detection
       ↓
  Payload Size Calculation
       ↓
  Bandwidth Calculator (with smoothing)
       ↓
  Terminal UI Rendering
```

## Threading Model

- **Main Thread**: UI rendering and user input
- **Blocking Thread**: Packet capture (pcap operations)
- **Async Task**: Bandwidth calculation timer
- **Channel Communication**: mpsc channels for packet data flow

## Key Design Decisions

### MAC Address-Based Direction Detection
**Why**: More reliable than IP-based detection, especially for routers and complex network topologies.

**Benefits**:
- Works correctly on routers/firewalls where traffic is forwarded
- Handles broadcast/multicast traffic properly
- Accurate for "any" interface monitoring

### Payload-Only Calculation Mode
**Why**: Address 10-20% bandwidth discrepancies with tools like Ookla speedtest.

**Implementation**: Parses Ethernet → IP → TCP headers to extract pure application data.

### Smoothing with Moving Average
**Why**: Reduce spikes and drops from bursty packet arrival patterns.

**Implementation**: Maintains sliding window of bandwidth samples, returns average.

## Error Handling

- **Interface Validation**: Checks interface exists on startup, lists available options
- **Graceful Shutdown**: Handles Ctrl+C and 'q' key properly
- **Permission Errors**: Clear messages about requiring elevated privileges
- **Network Errors**: Contextual error messages for packet capture failures

## Performance Considerations

- **Packet Buffer Management**: Automatic cleanup of old packets
- **History Limits**: Bounded memory usage (5 minutes of bandwidth history)
- **Efficient Parsing**: Zero-copy packet parsing where possible
- **Update Intervals**: Configurable to balance responsiveness vs CPU usage