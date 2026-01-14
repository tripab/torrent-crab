use criterion::{black_box, criterion_group, criterion_main, Criterion};
use torrent_client::Metainfo;

fn create_large_torrent() -> Vec<u8> {
    let mut data = b"d8:announce9:localhost4:infod6:lengthi1073741824e4:name8:big.file12:piece lengthi262144e6:pieces".to_vec();

    // 1GB file with 256KB pieces = 4096 pieces = 81920 bytes of hashes
    let num_pieces = 4096;
    let hash_bytes = num_pieces * 20;
    data.extend_from_slice(&hash_bytes.to_string().as_bytes());
    data.push(b':');
    data.extend_from_slice(&vec![0u8; hash_bytes]);
    data.extend_from_slice(b"ee");

    data
}

fn bench_parse_torrent(c: &mut Criterion) {
    let data = create_large_torrent();

    c.bench_function("parse large torrent", |b| {
        b.iter(|| {
            let metainfo = Metainfo::from_bytes(black_box(&data)).unwrap();
            black_box(metainfo);
        })
    });
}

criterion_group!(benches, bench_parse_torrent);
criterion_main!(benches);
