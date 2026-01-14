//! Peer-related utilities

use rand::Rng;

/// Generate a random 20-byte peer ID
///
/// Real clients use a format like: -TR2940-k8hj0wgej6ch
/// where TR2940 identifies Transmission 2.94.0
pub fn generate_peer_id() -> [u8; 20] {
    let mut rng = rand::thread_rng();
    let mut peer_id = [0u8; 20];

    // Use a prefix to identify our client
    peer_id[0..8].copy_from_slice(b"-RS0100-");

    // Fill rest with random bytes
    rng.fill(&mut peer_id[8..]);

    peer_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_peer_id() {
        let peer_id = generate_peer_id();
        assert_eq!(peer_id.len(), 20);
        assert_eq!(&peer_id[0..8], b"-RS0100-");
    }
}
