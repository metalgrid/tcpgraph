use tcpgraph::bandwidth::BandwidthCalculator;
use tcpgraph::capture::PacketInfo;
use std::time::{Duration, SystemTime};

#[test]
fn test_bandwidth_calculator_empty() {
    let mut calc = BandwidthCalculator::new(Duration::from_secs(1), 100);
    let bandwidth = calc.calculate_bandwidth();
    assert_eq!(bandwidth, 0.0);
}

#[test]
fn test_bandwidth_calculator_single_packet() {
    let mut calc = BandwidthCalculator::new(Duration::from_secs(1), 100);
    
    let packet = PacketInfo {
        timestamp: SystemTime::now(),
        size: 1000,
    };
    
    calc.add_packet(packet);
    let bandwidth = calc.calculate_bandwidth();
    
    assert!(bandwidth > 0.0);
    assert!(bandwidth <= 1000.0);
}

#[test]
fn test_bandwidth_calculator_multiple_packets() {
    let mut calc = BandwidthCalculator::new(Duration::from_secs(1), 100);
    let now = SystemTime::now();
    
    for _ in 0..5 {
        let packet = PacketInfo {
            timestamp: now,
            size: 200,
        };
        calc.add_packet(packet);
    }
    
    let bandwidth = calc.calculate_bandwidth();
    assert!(bandwidth >= 1000.0); // 5 packets * 200 bytes = 1000 bytes/sec
}

#[test]
fn test_bandwidth_calculator_history_limit() {
    let mut calc = BandwidthCalculator::new(Duration::from_secs(1), 2);
    
    for _ in 0..5 {
        calc.calculate_bandwidth();
    }
    
    assert!(calc.get_history().len() <= 2);
}