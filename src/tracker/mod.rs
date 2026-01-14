//! BitTorrent tracker communication
//!
//! Trackers help peers find each other. The client announces its presence
//! and receives a list of peers that have the same torrent.

use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use url::Url;

/// HTTP tracker client
pub struct Tracker {
    announce_url: String,
}

/// Request sent to tracker
#[derive(Debug, Clone)]
pub struct TrackerRequest {
    /// Info hash identifying the torrent
    pub info_hash: [u8; 20],
    /// Our peer ID
    pub peer_id: [u8; 20],
    /// Port we're listening on
    pub port: u16,
    /// Bytes uploaded this session
    pub uploaded: u64,
    /// Bytes downloaded this session
    pub downloaded: u64,
    /// Bytes left to download
    pub left: u64,
    /// Use compact peer format
    pub compact: bool,
    /// Event (started, stopped, completed)
    pub event: Option<TrackerEvent>,
}

#[derive(Debug, Clone)]
pub enum TrackerEvent {
    Started,
    Stopped,
    Completed,
}

/// Response from tracker
#[derive(Debug, Clone)]
pub struct TrackerResponse {
    /// Interval to wait before announcing again (seconds)
    pub interval: u32,
    /// List of peer addresses
    pub peers: Vec<SocketAddr>,
    /// Number of seeders (optional)
    pub seeders: Option<u32>,
    /// Number of leechers (optional)
    pub leechers: Option<u32>,
}

#[derive(Deserialize)]
struct BencodeTrackerResponse {
    interval: i64,
    #[serde(default)]
    complete: Option<i64>,
    #[serde(default)]
    incomplete: Option<i64>,
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}

impl Tracker {
    /// Create a new tracker client
    pub fn new(announce_url: String) -> Self {
        Self { announce_url }
    }

    /// Announce to tracker and get peer list
    pub fn announce(&self, request: &TrackerRequest) -> crate::Result<TrackerResponse> {
        let url = self.build_url(request)?;

        tracing::debug!("Announcing to tracker: {}", url);

        let response = reqwest::blocking::get(&url)?;
        let body = response.bytes()?;

        // Parse bencode response
        let tracker_response: BencodeTrackerResponse =
            serde_bencode::from_bytes(&body).map_err(|e| crate::Error::Tracker(e.to_string()))?;

        // Parse compact peer format
        let peers = Self::parse_compact_peers(&tracker_response.peers)?;

        tracing::info!("Received {} peers from tracker", peers.len());

        Ok(TrackerResponse {
            interval: tracker_response.interval as u32,
            peers,
            seeders: tracker_response.complete.map(|n| n as u32),
            leechers: tracker_response.incomplete.map(|n| n as u32),
        })
    }

    fn build_url(&self, req: &TrackerRequest) -> crate::Result<String> {
        let mut url = Url::parse(&self.announce_url)?;

        // Add query parameters
        url.query_pairs_mut()
            .append_pair("info_hash", &Self::url_encode_bytes(&req.info_hash))
            .append_pair("peer_id", &Self::url_encode_bytes(&req.peer_id))
            .append_pair("port", &req.port.to_string())
            .append_pair("uploaded", &req.uploaded.to_string())
            .append_pair("downloaded", &req.downloaded.to_string())
            .append_pair("left", &req.left.to_string())
            .append_pair("compact", if req.compact { "1" } else { "0" });

        if let Some(event) = &req.event {
            let event_str = match event {
                TrackerEvent::Started => "started",
                TrackerEvent::Stopped => "stopped",
                TrackerEvent::Completed => "completed",
            };
            url.query_pairs_mut().append_pair("event", event_str);
        }

        Ok(url.to_string())
    }

    /// URL encode binary data (special encoding for info_hash and peer_id)
    fn url_encode_bytes(bytes: &[u8]) -> String {
        bytes.iter().map(|&b| format!("%{:02x}", b)).collect()
    }

    /// Parse compact peer format (6 bytes per peer: 4 for IP, 2 for port)
    fn parse_compact_peers(data: &[u8]) -> crate::Result<Vec<SocketAddr>> {
        const PEER_SIZE: usize = 6;

        if data.len() % PEER_SIZE != 0 {
            return Err(crate::Error::Tracker(
                "Invalid compact peer data length".to_string(),
            ));
        }

        let peers = data
            .chunks_exact(PEER_SIZE)
            .map(|chunk| {
                let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                SocketAddr::new(IpAddr::V4(ip), port)
            })
            .collect();

        Ok(peers)
    }
}

impl TrackerRequest {
    /// Create a new tracker request for starting a download
    pub fn new_started(info_hash: [u8; 20], peer_id: [u8; 20], port: u16, total_size: u64) -> Self {
        Self {
            info_hash,
            peer_id,
            port,
            uploaded: 0,
            downloaded: 0,
            left: total_size,
            compact: true,
            event: Some(TrackerEvent::Started),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compact_peers() {
        // 192.168.1.1:6881 and 192.168.1.2:6882
        let data = vec![
            192, 168, 1, 1, 0x1A, 0xE1, // 6881 = 0x1AE1
            192, 168, 1, 2, 0x1A, 0xE2, // 6882 = 0x1AE2
        ];

        let peers = Tracker::parse_compact_peers(&data).unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].port(), 6881);
        assert_eq!(peers[1].port(), 6882);
    }

    #[test]
    fn test_url_encode_bytes() {
        let bytes = [0x12, 0x34, 0xAB, 0xCD];
        let encoded = Tracker::url_encode_bytes(&bytes);
        assert_eq!(encoded, "%12%34%ab%cd");
    }
}
