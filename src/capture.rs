use anyhow::{Context, Result};
use pcap::{Capture, Device};
use std::sync::mpsc;
use tokio::task;

pub struct PacketCapture {
    interface: String,
    filter: String,
}

#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub timestamp: std::time::SystemTime,
    pub size: u32,
}

impl PacketCapture {
    pub fn new(interface: String, filter: String) -> Self {
        Self { interface, filter }
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
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == interface)
            .context(format!("Interface '{}' not found", interface))?;

        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()?;

        cap.filter(&filter, true)
            .context("Failed to set packet filter")?;

        loop {
            match cap.next_packet() {
                Ok(packet) => {
                    let packet_info = PacketInfo {
                        timestamp: std::time::SystemTime::now(),
                        size: packet.header.caplen, // Use captured length, not just header
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