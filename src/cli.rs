use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "tcpgraph")]
#[command(about = "A terminal-based network bandwidth monitor")]
pub struct Args {
    #[arg(short, long, help = "Network interface to monitor")]
    pub interface: String,

    #[arg(short, long, help = "PCAP filter expression")]
    pub filter: String,

    #[arg(
        long,
        default_value = "1",
        help = "Graph update interval in seconds"
    )]
    pub interval: u64,

    #[arg(long, help = "Total monitoring duration in seconds")]
    pub duration: Option<u64>,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}