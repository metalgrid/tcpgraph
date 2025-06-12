use anyhow::{Context, Result};
use pcap::{Capture, Device};
use pnet::datalink;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::Packet;
use pnet::util::MacAddr;
use std::collections::HashSet;
use std::sync::mpsc;
use tokio::task;

pub struct PacketCapture {
    interface: String,
    filter: String,
    payload_only: bool,
}

#[derive(Debug, Clone)]
pub enum TrafficDirection {
    Inbound,
    Outbound,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub timestamp: std::time::SystemTime,
    pub size: u32,
    pub direction: TrafficDirection,
}

impl PacketCapture {
    pub fn new(interface: String, filter: String, payload_only: bool) -> Self {
        Self { interface, filter, payload_only }
    }

    fn get_local_macs(interface_name: &str) -> HashSet<MacAddr> {
        let mut local_macs = HashSet::new();
        
        // Get all network interfaces
        let interfaces = datalink::interfaces();
        
        for iface in interfaces {
            // If specific interface requested, only use that interface
            // If "any" interface, include all interfaces
            if interface_name == "any" || iface.name == interface_name {
                if let Some(mac) = iface.mac {
                    local_macs.insert(mac);
                }
            }
        }
        
        local_macs
    }

    fn get_payload_size(packet_data: &[u8]) -> u32 {
        if let Some(eth_packet) = EthernetPacket::new(packet_data) {
            match eth_packet.get_ethertype() {
                EtherTypes::Ipv4 => {
                    if let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) {
                        let total_length = ipv4_packet.get_total_length() as u32;
                        let header_length = (ipv4_packet.get_header_length() as u32) * 4;
                        
                        // For TCP, subtract TCP header as well
                        if ipv4_packet.get_next_level_protocol() == IpNextHeaderProtocols::Tcp {
                            if let Some(tcp_packet) = TcpPacket::new(ipv4_packet.payload()) {
                                let tcp_header_length = (tcp_packet.get_data_offset() as u32) * 4;
                                return total_length.saturating_sub(header_length + tcp_header_length);
                            }
                        }
                        
                        // For other protocols, just subtract IP header
                        return total_length.saturating_sub(header_length);
                    }
                }
                EtherTypes::Ipv6 => {
                    if let Some(ipv6_packet) = Ipv6Packet::new(eth_packet.payload()) {
                        let payload_length = ipv6_packet.get_payload_length() as u32;
                        
                        // For TCP, subtract TCP header
                        if ipv6_packet.get_next_header() == IpNextHeaderProtocols::Tcp {
                            if let Some(tcp_packet) = TcpPacket::new(ipv6_packet.payload()) {
                                let tcp_header_length = (tcp_packet.get_data_offset() as u32) * 4;
                                return payload_length.saturating_sub(tcp_header_length);
                            }
                        }
                        
                        return payload_length;
                    }
                }
                _ => {}
            }
        }
        
        // Fallback to full packet size if we can't parse headers
        packet_data.len() as u32
    }

    fn determine_direction(packet_data: &[u8], local_macs: &HashSet<MacAddr>) -> TrafficDirection {
        if let Some(eth_packet) = EthernetPacket::new(packet_data) {
            let src_mac = eth_packet.get_source();
            let dst_mac = eth_packet.get_destination();
            
            let src_is_local = local_macs.contains(&src_mac);
            let dst_is_local = local_macs.contains(&dst_mac);
            
            // Check for broadcast/multicast destinations
            let is_broadcast = dst_mac == MacAddr::broadcast();
            let is_multicast = dst_mac.is_multicast();
            
            match (src_is_local, dst_is_local, is_broadcast, is_multicast) {
                // Source is our interface -> outbound traffic
                (true, false, _, _) => TrafficDirection::Outbound,
                // Destination is our interface -> inbound traffic  
                (false, true, _, _) => TrafficDirection::Inbound,
                // Broadcast/multicast from our interface -> outbound
                (true, _, true, _) | (true, _, _, true) => TrafficDirection::Outbound,
                // Broadcast/multicast to us -> inbound
                (false, _, true, _) | (false, _, _, true) => TrafficDirection::Inbound,
                // Internal traffic (both local) or external (neither local) -> unknown
                _ => TrafficDirection::Unknown,
            }
        } else {
            TrafficDirection::Unknown
        }
    }

    pub async fn start_capture(&self) -> Result<mpsc::Receiver<PacketInfo>> {
        let (tx, rx) = mpsc::channel();
        let interface = self.interface.clone();
        let filter = self.filter.clone();

        let payload_only = self.payload_only;
        task::spawn_blocking(move || {
            Self::capture_packets(interface, filter, payload_only, tx)
        });

        Ok(rx)
    }

    fn capture_packets(
        interface: String,
        filter: String,
        payload_only: bool,
        tx: mpsc::Sender<PacketInfo>,
    ) -> Result<()> {
        let device = if interface == "any" {
            // For "any" interface, we need to handle it specially
            Device::lookup()?.unwrap_or_else(|| {
                Device::list().unwrap_or_default()
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Device {
                        name: "any".to_string(),
                        desc: Some("Pseudo-device that captures on all interfaces".to_string()),
                        addresses: vec![],
                        flags: pcap::DeviceFlags::empty(),
                    })
            })
        } else {
            Device::list()?
                .into_iter()
                .find(|d| d.name == interface)
                .context(format!("Interface '{}' not found", interface))?
        };

        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        cap.filter(&filter, true)
            .context("Failed to set packet filter")?;

        let local_macs = Self::get_local_macs(&interface);

        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    let direction = Self::determine_direction(&packet.data, &local_macs);
                    
                    let size = if payload_only {
                        Self::get_payload_size(&packet.data)
                    } else {
                        packet.header.caplen
                    };
                    
                    let packet_info = PacketInfo {
                        timestamp: std::time::SystemTime::now(),
                        size,
                        direction,
                    };

                    if tx.send(packet_info).is_err() {
                        break;
                    }
                }
                Err(pcap::Error::TimeoutExpired) => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }
}