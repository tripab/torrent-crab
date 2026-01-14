# A BitTorrent Client in Rust

A production-quality BitTorrent client implementation in Rust, built in a phased manner. This was inspired from the article [Building a BitTorrent client in Go](https://blog.jse.li/posts/torrent/).

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)](tests/)

## Phase 1 Objectives

- [x] Parse .torrent files (bencode format)
- [x] Calculate info hashes (SHA-1)
- [x] Communicate with HTTP trackers
- [x] Parse compact peer format
- [x] Discover and list available peers
- [x] Comprehensive tests and documentation
- [x] Production-ready error handling

## Quick Start

```bash
# Clone the repository
git clone https://github.com/tripab/torrent-crab
cd torrent-crab

# Build the project
cargo build --release

# Download a test torrent (Debian ISO - legal and fast!)
wget https://cdimage.debian.org/debian-cd/current/amd64/bt-cd/debian-13.3.0-amd64-netinst.iso.torrent

# Run the demo
cargo run --release -- --torrent debian-13.3.0-amd64-netinst.iso.torrent

# With debug logging
RUST_LOG=torrent_crab=debug cargo run --release -- --torrent debian-13.3.0-amd64-netinst.iso.torrent
```

## Demo Output

```
BitTorrent Client - Phase 1 Demo

Parsing torrent file: debian-13.3.0-amd64-netinst.iso.torrent

Torrent Information:
   Name: debian-13.3.0-amd64-netinst.iso
   Size: 658505728 bytes (628.00 MB)
   Pieces: 2512
   Piece length: 262144 bytes
   Info hash: 5a8062c076fa85e8056451c0d9aa04349ae27909
   Comment: "Debian CD from cdimage.debian.org"

Trackers:
   - http://bttracker.debian.org:6969/announce

Our Peer ID: -RS0100-

Contacting tracker...

Tracker Response:
   Interval: 1800 seconds
   Seeders: 45
   Leechers: 12

Discovered Peers (57):
   - 185.21.217.50:51413
   - 91.228.167.78:51413
   - 94.23.170.107:6881
   ... and 47 more

Phase 1 objectives complete!
```

## What's Implemented

### Core Modules

#### 1. **Bencode Parser** (`src/bencode/`)
- Full bencode format support (integers, strings, lists, dictionaries)
- Leverages `serde` for ergonomic deserialization
- Zero-copy where possible
- Comprehensive error handling

#### 2. **Metainfo Parser** (`src/metainfo.rs`)
- Parse single-file and multi-file torrents
- Calculate info hash (SHA-1 of info dict)
- Extract all metadata (name, size, piece length, comments)
- Support for announce-list (multiple trackers)

#### 3. **Tracker Client** (`src/tracker/`)
- HTTP tracker protocol implementation
- Proper URL encoding for binary data
- Parse compact peer format (6 bytes per peer)
- Extract seeder/leecher counts
- Handle tracker errors gracefully

#### 4. **Peer Utilities** (`src/peer/`)
- Generate spec-compliant peer IDs
- Peer address representation


## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture --test-threads=1

# Run specific module tests
cargo test --lib bencode
cargo test --lib metainfo
cargo test --lib tracker

# Run integration tests
cargo test --test integration_test

# Run benchmarks
cargo bench

# Generate coverage report (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

### Test Coverage

- **Unit tests**: added unit tests
- **Integration tests**: added an end-to-end workflow test
- **Property tests**: added tests for edge cases and error conditions
- **Benchmarks**: added performance benchmarking tests

## Documentation

```bash
# Generate and open documentation
cargo doc --open

# Documentation includes:
# - Module-level docs with examples
# - Function documentation with examples
# - Type documentation
# - Internal implementation notes
```

## Development

### Prerequisites

- Rust 1.75 or newer
- Cargo (comes with Rust)

### Adding Dependencies

```bash
# Add a dependency
cargo add <crate-name>

# Add a development dependency
cargo add --dev <crate-name>
```

### Code Style

This project follows standard Rust conventions:

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check without building
cargo check
```

## Learning Resources

### Understanding BitTorrent

1. **Official Specifications**:
   - [BEP 0003](http://www.bittorrent.org/beps/bep_0003.html) - The BitTorrent Protocol
   - [Unofficial Spec](https://wiki.theory.org/BitTorrentSpecification)

2. **Bencode Format**:
   - [Bencode Wikipedia](https://en.wikipedia.org/wiki/Bencode)
   - [50-line Python Parser](https://web.archive.org/web/20200105114449/https://effbot.org/zone/bencode.htm)

3. **Related Articles**:
   - [Building a BitTorrent client in Go](https://blog.jse.li/posts/torrent/)
