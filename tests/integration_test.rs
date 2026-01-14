use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use torrent_crab::Metainfo;

/// Helper to create a minimal valid .torrent file
fn create_test_torrent_file(dir: &TempDir) -> std::path::PathBuf {
    let torrent_path = dir.path().join("test.torrent");
    let torrent_data = create_minimal_torrent();

    let mut file = File::create(&torrent_path).unwrap();
    file.write_all(&torrent_data).unwrap();

    torrent_path
}

fn create_minimal_torrent() -> Vec<u8> {
    // A valid minimal .torrent file in bencode format
    let data = "d8:announce23:http://tracker.test:6969\
                 4:infod6:lengthi1048576e\
                 4:name9:test.file\
                 12:piece lengthi262144e\
                 6:pieces80:";

    let mut result = data.as_bytes().to_vec();
    // Add 4 piece hashes (4 * 20 = 80 bytes)
    for _ in 0..4 {
        result.extend_from_slice(&[0u8; 20]);
    }
    result.push(b'e');
    result.push(b'e');
    result
}

#[test]
fn test_parse_single_file_torrent() {
    let temp_dir = TempDir::new().unwrap();
    let torrent_path = create_test_torrent_file(&temp_dir);

    let metainfo = Metainfo::from_file(&torrent_path).unwrap();

    assert_eq!(metainfo.info.name, "test.file");
    assert_eq!(metainfo.total_size(), 1048576);
    assert_eq!(metainfo.num_pieces(), 4);
    assert_eq!(metainfo.info.piece_length, 262144);
}

#[test]
fn test_info_hash_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let torrent_path = create_test_torrent_file(&temp_dir);

    let metainfo1 = Metainfo::from_file(&torrent_path).unwrap();
    let metainfo2 = Metainfo::from_file(&torrent_path).unwrap();

    assert_eq!(metainfo1.info_hash, metainfo2.info_hash);
}

#[test]
fn test_multi_file_torrent() {
    // Create a multi-file torrent
    let torrent_data = "d8:announce23:http://tracker.test:6969\
                        4:infod5:filesl\
                        d6:lengthi1000e4:pathl5:file1ee\
                        d6:lengthi2000e4:pathl5:file2ee\
                        e\
                        4:name7:testdir\
                        12:piece lengthi512e\
                        6:pieces60:";

    let mut data = torrent_data.as_bytes().to_vec();
    // Add 3 piece hashes
    for _ in 0..3 {
        data.extend_from_slice(&[0u8; 20]);
    }
    data.extend_from_slice(b"ee");

    let metainfo = Metainfo::from_bytes(&data).unwrap();

    assert_eq!(metainfo.total_size(), 3000);
    assert_eq!(metainfo.info.name, "testdir");

    if let torrent_crab::metainfo::FileInfo::Multi { files } = &metainfo.info.files {
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].length, 1000);
        assert_eq!(files[1].length, 2000);
    } else {
        panic!("Expected multi-file torrent");
    }
}

#[test]
fn test_tracker_url_building() {
    use torrent_crab::tracker::TrackerRequest;

    let info_hash = [1u8; 20];
    let peer_id = [2u8; 20];

    let request = TrackerRequest::new_started(info_hash, peer_id, 6881, 1000000);

    // The tracker would build a URL like:
    // http://tracker.test:6969/announce?info_hash=%01%01...&peer_id=%02%02...
    assert_eq!(request.port, 6881);
    assert_eq!(request.left, 1000000);
    assert!(request.compact);
}
