#!/usr/bin/env python3
"""
Create test .torrent files for development and testing.
Requires: pip install bencodepy
"""

import bencodepy
import hashlib
import sys
from pathlib import Path

def create_single_file_torrent(name="test.txt", size=1024, piece_length=512):
    """Create a minimal single-file torrent."""
    num_pieces = (size + piece_length - 1) // piece_length
    pieces = b"".join(hashlib.sha1(f"piece{i}".encode()).digest() 
                      for i in range(num_pieces))
    
    torrent = {
        b"announce": b"http://tracker.test:6969/announce",
        b"info": {
            b"name": name.encode(),
            b"length": size,
            b"piece length": piece_length,
            b"pieces": pieces,
        },
        b"comment": b"Test torrent for development",
        b"creation date": 1234567890,
    }
    
    return bencodepy.encode(torrent)

def create_multi_file_torrent(name="testdir", files=None):
    """Create a multi-file torrent."""
    if files is None:
        files = [
            {"path": ["file1.txt"], "length": 1000},
            {"path": ["subdir", "file2.txt"], "length": 2000},
        ]
    
    total_size = sum(f["length"] for f in files)
    piece_length = 512
    num_pieces = (total_size + piece_length - 1) // piece_length
    pieces = b"".join(hashlib.sha1(f"piece{i}".encode()).digest() 
                      for i in range(num_pieces))
    
    torrent = {
        b"announce": b"http://tracker.test:6969/announce",
        b"info": {
            b"name": name.encode(),
            b"files": [
                {
                    b"path": [p.encode() for p in f["path"]],
                    b"length": f["length"]
                }
                for f in files
            ],
            b"piece length": piece_length,
            b"pieces": pieces,
        },
        b"comment": b"Multi-file test torrent",
    }
    
    return bencodepy.encode(torrent)

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 create_test_torrent.py [single|multi]")
        sys.exit(1)
    
    mode = sys.argv[1]
    
    if mode == "single":
        data = create_single_file_torrent()
        filename = "test-single.torrent"
    elif mode == "multi":
        data = create_multi_file_torrent()
        filename = "test-multi.torrent"
    else:
        print(f"Unknown mode: {mode}")
        sys.exit(1)
    
    Path(filename).write_bytes(data)
    print(f"✅ Created {filename}")
    
    # Show info
    torrent = bencodepy.decode(data)
    info = torrent[b"info"]
    print(f"   Name: {info[b'name'].decode()}")
    if b"length" in info:
        print(f"   Size: {info[b'length']} bytes")
    else:
        total = sum(f[b"length"] for f in info[b"files"])
        print(f"   Total size: {total} bytes ({len(info[b'files'])} files)")
    print(f"   Pieces: {len(info[b'pieces']) // 20}")

if __name__ == "__main__":
    main()
