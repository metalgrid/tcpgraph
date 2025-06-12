use crate::capture::{PacketInfo, TrafficDirection};
use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct BandwidthData {
    pub timestamp: SystemTime,
    pub inbound_bps: f64,
    pub outbound_bps: f64,
}

#[derive(Debug, Clone)]
pub struct DirectionalBandwidth {
    pub inbound: f64,
    pub outbound: f64,
}

pub struct BandwidthCalculator {
    packet_buffer: VecDeque<PacketInfo>,
    bandwidth_history: VecDeque<BandwidthData>,
    max_history: usize,
    window_duration: Duration,
}

impl BandwidthCalculator {
    pub fn new(window_duration: Duration, max_history: usize) -> Self {
        Self {
            packet_buffer: VecDeque::new(),
            bandwidth_history: VecDeque::new(),
            max_history,
            window_duration,
        }
    }

    pub fn add_packet(&mut self, packet: PacketInfo) {
        self.packet_buffer.push_back(packet);
        self.cleanup_old_packets();
    }

    pub fn calculate_bandwidth(&mut self) -> DirectionalBandwidth {
        let now = SystemTime::now();
        let cutoff_time = now - self.window_duration;

        let (inbound_bytes, outbound_bytes): (u64, u64) = self.packet_buffer
            .iter()
            .filter(|packet| packet.timestamp >= cutoff_time)
            .fold((0, 0), |(in_acc, out_acc), packet| {
                match packet.direction {
                    TrafficDirection::Inbound => (in_acc + packet.size as u64, out_acc),
                    TrafficDirection::Outbound => (in_acc, out_acc + packet.size as u64),
                    TrafficDirection::Unknown => {
                        // For router scenarios, unknown traffic (neither source nor dest MAC is ours)
                        // represents forwarded traffic. We'll count it as transit traffic.
                        // For now, we'll split it to show total network activity.
                        let half_size = packet.size as u64 / 2;
                        (in_acc + half_size, out_acc + half_size)
                    }
                }
            });

        let inbound_bps = inbound_bytes as f64 / self.window_duration.as_secs_f64();
        let outbound_bps = outbound_bytes as f64 / self.window_duration.as_secs_f64();

        let bandwidth_data = BandwidthData {
            timestamp: now,
            inbound_bps,
            outbound_bps,
        };

        self.bandwidth_history.push_back(bandwidth_data);

        if self.bandwidth_history.len() > self.max_history {
            self.bandwidth_history.pop_front();
        }

        DirectionalBandwidth {
            inbound: inbound_bps,
            outbound: outbound_bps,
        }
    }

    pub fn get_history(&self) -> &VecDeque<BandwidthData> {
        &self.bandwidth_history
    }

    pub fn get_chart_data(&self) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let inbound: Vec<(f64, f64)> = self.bandwidth_history
            .iter()
            .enumerate()
            .map(|(i, data)| (i as f64, data.inbound_bps / 1024.0))
            .collect();
        
        let outbound: Vec<(f64, f64)> = self.bandwidth_history
            .iter()
            .enumerate()
            .map(|(i, data)| (i as f64, data.outbound_bps / 1024.0))
            .collect();
            
        (inbound, outbound)
    }

    fn cleanup_old_packets(&mut self) {
        let cutoff_time = SystemTime::now() - self.window_duration * 2;
        
        while let Some(packet) = self.packet_buffer.front() {
            if packet.timestamp < cutoff_time {
                self.packet_buffer.pop_front();
            } else {
                break;
            }
        }
    }
}

pub async fn start_bandwidth_monitor(
    packet_rx: mpsc::Receiver<PacketInfo>,
    update_interval: Duration,
) -> mpsc::Receiver<DirectionalBandwidth> {
    let (tx, rx) = mpsc::channel();
    let mut calculator = BandwidthCalculator::new(
        Duration::from_secs(1),
        300, // Keep 5 minutes of history
    );

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(update_interval);
        
        loop {
            interval.tick().await;

            while let Ok(packet) = packet_rx.try_recv() {
                calculator.add_packet(packet);
            }

            let bandwidth = calculator.calculate_bandwidth();
            
            if tx.send(bandwidth).is_err() {
                break;
            }
        }
    });

    rx
}