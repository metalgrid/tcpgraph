use crate::capture::PacketInfo;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct BandwidthData {
    pub timestamp: SystemTime,
    pub bytes_per_second: f64,
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

    pub fn calculate_bandwidth(&mut self) -> f64 {
        let now = SystemTime::now();
        let cutoff_time = now - self.window_duration;

        let total_bytes: u64 = self.packet_buffer
            .iter()
            .filter(|packet| packet.timestamp >= cutoff_time)
            .map(|packet| packet.size as u64)
            .sum();

        let bytes_per_second = total_bytes as f64 / self.window_duration.as_secs_f64();

        let bandwidth_data = BandwidthData {
            timestamp: now,
            bytes_per_second,
        };

        self.bandwidth_history.push_back(bandwidth_data);

        if self.bandwidth_history.len() > self.max_history {
            self.bandwidth_history.pop_front();
        }

        bytes_per_second
    }

    pub fn get_history(&self) -> &VecDeque<BandwidthData> {
        &self.bandwidth_history
    }

    pub fn get_chart_data(&self) -> Vec<(f64, f64)> {
        self.bandwidth_history
            .iter()
            .enumerate()
            .map(|(i, data)| (i as f64, data.bytes_per_second / 1024.0))
            .collect()
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
) -> mpsc::Receiver<f64> {
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