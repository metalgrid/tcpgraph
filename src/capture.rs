use anyhow::{Context, Result};
use pcap::{Capture, Device};
use pnet::datalink;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::mpsc;
use tokio::task;

pub struct PacketCapture {
    interface: String,
    filter: String,
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
    pub fn new(interface: String, filter: String) -> Self {
        Self { interface, filter }
    }

    fn get_local_ips(interface_name: &str) -> HashSet<IpAddr> {
        let mut local_ips = HashSet::new();
        
        // Always include localhost
        local_ips.insert(IpAddr::V4(Ipv4Addr::LOCALHOST));
        local_ips.insert(IpAddr::V6(Ipv6Addr::LOCALHOST));
        
        // Get all network interfaces
        let interfaces = datalink::interfaces();
        
        for iface in interfaces {
            // If specific interface requested, only use that interface
            // If "any" interface, include all interfaces
            if interface_name == "any" || iface.name == interface_name {
                for ip_network in &iface.ips {
                    local_ips.insert(ip_network.ip());
                }
            }
        }
        
        local_ips
    }

    fn determine_direction(packet_data: &[u8], local_ips: &HashSet<IpAddr>) -> TrafficDirection {
        if let Some(eth_packet) = EthernetPacket::new(packet_data) {
            match eth_packet.get_ethertype() {
                EtherTypes::Ipv4 => {
                    if let Some(ipv4_packet) = Ipv4Packet::new(eth_packet.payload()) {
                        let src_ip = IpAddr::V4(ipv4_packet.get_source());
                        let dst_ip = IpAddr::V4(ipv4_packet.get_destination());
                        
                        let src_local = local_ips.contains(&src_ip);
                        let dst_local = local_ips.contains(&dst_ip);
                        
                        match (src_local, dst_local) {
                            (true, false) => TrafficDirection::Outbound,
                            (false, true) => TrafficDirection::Inbound,
                            _ => TrafficDirection::Unknown,
                        }
                    } else {
                        TrafficDirection::Unknown
                    }
                }
                EtherTypes::Ipv6 => {
                    if let Some(ipv6_packet) = Ipv6Packet::new(eth_packet.payload()) {
                        let src_ip = IpAddr::V6(ipv6_packet.get_source());
                        let dst_ip = IpAddr::V6(ipv6_packet.get_destination());
                        
                        let src_local = local_ips.contains(&src_ip);
                        let dst_local = local_ips.contains(&dst_ip);
                        
                        match (src_local, dst_local) {
                            (true, false) => TrafficDirection::Outbound,
                            (false, true) => TrafficDirection::Inbound,
                            _ => TrafficDirection::Unknown,
                        }
                    } else {
                        TrafficDirection::Unknown
                    }
                }
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

        task::spawn_blocking(move || {
            Self::capture_packets(interface, filter, tx)
        });

        Ok(rx)
    }

    fn capture_packets(
        interface: String,
        filter: String,
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

        let local_ips = Self::get_local_ips(&interface);

        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    let direction = Self::determine_direction(&packet.data, &local_ips);
                    
                    let packet_info = PacketInfo {
                        timestamp: std::time::SystemTime::now(),
                        size: packet.header.caplen,
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