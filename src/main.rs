//! - Parsing .torrent files
//! - Communicating with trackers
//! - Discovering peers

use clap::Parser;
use std::path::PathBuf;
use torrent_crab::metainfo::FileInfo;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "torrent-crab")]
#[command(
    about = "A BitTorrent client in Rust - A demo showcasing parsing torrent files, tracker communication, and discovering peers"
)]
struct Cli {
    /// Path to .torrent file
    #[arg(short, long)]
    torrent: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "6881")]
    port: u16,
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("torrent-crab=debug")
        .init();

    let cli = Cli::parse();

    println!("BitTorrent Client - A demo showcasing parsing torrent files, tracker communication, and discovering peers\n");

    // Parse .torrent file
    println!("Parsing torrent file: {}", cli.torrent.display());
    let metainfo = torrent_crab::Metainfo::from_file(&cli.torrent)?;

    // Display torrent information
    println!("\nTorrent Information:");
    println!("   Name: {}", metainfo.info.name);
    println!(
        "   Size: {} bytes ({:.2} MB)",
        metainfo.total_size(),
        metainfo.total_size() as f64 / 1_048_576.0
    );
    println!("   Pieces: {}", metainfo.num_pieces());
    println!("   Piece length: {} bytes", metainfo.info.piece_length);
    println!("   Info hash: {}", hex::encode(metainfo.info_hash));

    if let Some(comment) = &metainfo.comment {
        println!("   Comment: {}", comment);
    }

    // Display file structure
    match &metainfo.info.files {
        FileInfo::Single { length } => {
            println!("   Single file: {} bytes", length);
        }
        FileInfo::Multi { files } => {
            println!("   Multiple files: {}", files.len());
            for file in files.iter().take(5) {
                println!("      - {}: {} bytes", file.path.join("/"), file.length);
            }
            if files.len() > 5 {
                println!("      ... and {} more", files.len() - 5);
            }
        }
    }

    // Display trackers
    println!("\nTrackers:");
    for tracker_url in metainfo.all_trackers().iter().take(3) {
        println!("   - {}", tracker_url);
    }

    // Generate peer ID
    let peer_id = torrent_crab::peer::generate_peer_id();
    println!("\nOur Peer ID: {}", String::from_utf8_lossy(&peer_id[0..8]));

    // Contact tracker
    println!("\nContacting tracker...");
    let tracker = torrent_crab::Tracker::new(metainfo.announce.clone());
    let request = torrent_crab::tracker::TrackerRequest::new_started(
        metainfo.info_hash,
        peer_id,
        cli.port,
        metainfo.total_size(),
    );

    match tracker.announce(&request) {
        Ok(response) => {
            println!("\nTracker Response:");
            println!("   Interval: {} seconds", response.interval);

            if let Some(seeders) = response.seeders {
                println!("   Seeders: {}", seeders);
            }
            if let Some(leechers) = response.leechers {
                println!("   Leechers: {}", leechers);
            }

            println!("\nDiscovered Peers ({}):", response.peers.len());
            for peer in response.peers.iter().take(10) {
                println!("   - {}", peer);
            }
            if response.peers.len() > 10 {
                println!("   ... and {} more", response.peers.len() - 10);
            }

            println!("   ✓ Parsed .torrent file");
            println!("   ✓ Calculated info hash");
            println!("   ✓ Contacted tracker");
            println!("   ✓ Discovered {} peers", response.peers.len());
        }
        Err(e) => {
            eprintln!("\n❌ Tracker error: {}", e);
            eprintln!("   This might be because:");
            eprintln!("   - The tracker is down");
            eprintln!("   - Network connectivity issues");
            eprintln!("   - Invalid tracker URL in .torrent file");
        }
    }

    Ok(())
}
