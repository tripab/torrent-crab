//! Torrent metainfo file (.torrent) parsing
//!
//! A .torrent file contains all the information needed to download a file:
//! - Tracker URL(s)
//! - File information (name, length, piece hashes)
//! - Optional metadata (creation date, comments)

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

/// Parsed .torrent file
#[derive(Debug, Clone)]
pub struct Metainfo {
    /// Primary tracker URL
    pub announce: String,
    /// List of backup trackers
    pub announce_list: Vec<Vec<String>>,
    /// SHA-1 hash of the info dictionary (identifies the torrent)
    pub info_hash: [u8; 20],
    /// Detailed file information
    pub info: Info,
    /// Optional creation timestamp
    pub creation_date: Option<i64>,
    /// Optional comment
    pub comment: Option<String>,
    /// Optional creator
    pub created_by: Option<String>,
}

/// File information from the info dictionary
#[derive(Debug, Clone)]
pub struct Info {
    /// Suggested name for the file/directory
    pub name: String,
    /// Length of each piece in bytes (typically 256KB or 512KB)
    pub piece_length: u64,
    /// Concatenated SHA-1 hashes of all pieces
    pub pieces: Vec<[u8; 20]>,
    /// Single file or multiple files
    pub files: FileInfo,
}

/// File layout - either single file or multiple files
#[derive(Debug, Clone)]
pub enum FileInfo {
    /// Single file torrent
    Single {
        /// Length in bytes
        length: u64,
    },
    /// Multi-file torrent
    Multi {
        /// List of files with paths and lengths
        files: Vec<FileEntry>,
    },
}

/// A single file in a multi-file torrent
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path components (e.g., ["subdir", "file.txt"])
    pub path: Vec<String>,
    /// Length in bytes
    pub length: u64,
}

// Internal structures for deserializing bencode
#[derive(Deserialize)]
struct BencodeTorrent {
    announce: String,
    #[serde(rename = "announce-list", default)]
    announce_list: Vec<Vec<String>>,
    info: BencodeInfo,
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    comment: Option<String>,
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct BencodeInfo {
    name: String,
    #[serde(rename = "piece length")]
    piece_length: u64,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
    #[serde(flatten)]
    file_info: BencodeFileInfo,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum BencodeFileInfo {
    Single { length: u64 },
    Multi { files: Vec<BencodeFile> },
}

#[derive(Deserialize, Serialize)]
struct BencodeFile {
    path: Vec<String>,
    length: u64,
}

impl Metainfo {
    /// Parse a .torrent file
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let bytes = fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    /// Parse .torrent data from bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        let torrent: BencodeTorrent = serde_bencode::from_bytes(bytes)
            .map_err(|e| crate::Error::InvalidMetainfo(e.to_string()))?;

        // Calculate info_hash by re-encoding the info dict
        let info_bytes = serde_bencode::to_bytes(&torrent.info)
            .map_err(|e| crate::Error::InvalidMetainfo(e.to_string()))?;
        let mut hasher = Sha1::new();
        hasher.update(&info_bytes);
        let info_hash: [u8; 20] = hasher.finalize().into();

        // Parse piece hashes
        if torrent.info.pieces.len() % 20 != 0 {
            return Err(crate::Error::InvalidMetainfo(
                "Pieces length must be multiple of 20".to_string(),
            ));
        }
        let pieces: Vec<[u8; 20]> = torrent
            .info
            .pieces
            .chunks_exact(20)
            .map(|chunk| {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(chunk);
                hash
            })
            .collect();

        // Convert file info
        let files = match torrent.info.file_info {
            BencodeFileInfo::Single { length } => FileInfo::Single { length },
            BencodeFileInfo::Multi { files } => FileInfo::Multi {
                files: files
                    .into_iter()
                    .map(|f| FileEntry {
                        path: f.path,
                        length: f.length,
                    })
                    .collect(),
            },
        };

        Ok(Metainfo {
            announce: torrent.announce,
            announce_list: torrent.announce_list,
            info_hash,
            info: Info {
                name: torrent.info.name,
                piece_length: torrent.info.piece_length,
                pieces,
                files,
            },
            creation_date: torrent.creation_date,
            comment: torrent.comment,
            created_by: torrent.created_by,
        })
    }

    /// Get total size of all files in bytes
    pub fn total_size(&self) -> u64 {
        match &self.info.files {
            FileInfo::Single { length } => *length,
            FileInfo::Multi { files } => files.iter().map(|f| f.length).sum(),
        }
    }

    /// Get number of pieces
    pub fn num_pieces(&self) -> usize {
        self.info.pieces.len()
    }

    /// Get all tracker URLs (primary + backups)
    pub fn all_trackers(&self) -> Vec<String> {
        let mut trackers = vec![self.announce.clone()];
        for tier in &self.announce_list {
            trackers.extend(tier.clone());
        }
        trackers.sort();
        trackers.dedup();
        trackers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_torrent() -> Vec<u8> {
        // Minimal valid .torrent file
        let torrent = r#"d8:announce9:localhost4:infod6:lengthi1000e4:name8:test.txt12:piece lengthi512e6:pieces20:12345678901234567890ee"#;
        torrent.as_bytes().to_vec()
    }

    #[test]
    fn test_parse_torrent() {
        let data = create_test_torrent();
        let metainfo = Metainfo::from_bytes(&data).unwrap();

        assert_eq!(metainfo.announce, "localhost");
        assert_eq!(metainfo.info.name, "test.txt");
        assert_eq!(metainfo.total_size(), 1000);
        assert_eq!(metainfo.num_pieces(), 1);
    }

    #[test]
    fn test_info_hash_consistency() {
        let data = create_test_torrent();
        let metainfo1 = Metainfo::from_bytes(&data).unwrap();
        let metainfo2 = Metainfo::from_bytes(&data).unwrap();

        assert_eq!(metainfo1.info_hash, metainfo2.info_hash);
    }

    #[test]
    fn test_invalid_piece_length() {
        // Pieces must be multiple of 20
        let torrent_data = b"d8:announce9:localhost4:infod6:lengthi1000e4:name4:test12:piece lengthi512e6:pieces19:1234567890123456789ee";

        let result = Metainfo::from_bytes(torrent_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_piece_calculation() {
        let data = create_test_torrent();
        let metainfo = Metainfo::from_bytes(&data).unwrap();

        // 1000 bytes / 512 piece_length = 2 pieces (rounded up)
        assert_eq!(metainfo.num_pieces(), 1);
    }

    #[test]
    fn test_all_trackers_deduplication() {
        let torrent = "d8:announce9:tracker-113:announce-listll9:tracker-1e\
                       l9:tracker-2el9:tracker-1ee\
                       4:infod6:lengthi1000e4:name4:test12:piece lengthi512e\
                       6:pieces20:12345678901234567890ee";

        let metainfo = Metainfo::from_bytes(torrent.as_bytes()).unwrap();
        let trackers = metainfo.all_trackers();

        // Should deduplicate tracker-1
        assert_eq!(trackers.len(), 2);
    }
}
