use super::*;

use rand::Rng;

//-----------------------------------------------------------------------------

fn rans_test(data: &[u8], flags: RANSFlags, test_case: &str) {
    let compressed = rans_compress(data, flags);
    assert!(compressed.is_ok(), "Compression failed for {}: {}", test_case, compressed.unwrap_err());
    let compressed = compressed.unwrap();

    let with_output_size = rans_decompress(&compressed, Some(data.len()));
    assert!(with_output_size.is_ok(), "Decompression with output size failed for {}: {}", test_case, with_output_size.unwrap_err());
    let with_output_size = with_output_size.unwrap();
    assert_eq!(with_output_size.len(), data.len(), "Wrong decompressed size with output size for {}", test_case);
    assert_eq!(with_output_size, data, "Wrong decompressed data with output size for {}", test_case);

    let without_output_size = rans_decompress(&compressed, None);
    assert!(without_output_size.is_ok(), "Decompression without output size failed for {}: {}", test_case, without_output_size.unwrap_err());
    let without_output_size = without_output_size.unwrap();
    assert_eq!(without_output_size.len(), data.len(), "Wrong decompressed size without output size for {}", test_case);
    assert_eq!(without_output_size, data, "Wrong decompressed data without output size for {}", test_case);
}

const ALPHANUMERIC: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

const DNA: &[u8] = b"ACGT";

fn rans_test_cases(flags: RANSFlags) {
    let mut test_cases = Vec::new();

    test_cases.push(("empty", vec![]));
    test_cases.push(("hello", b"Hello, world!".to_vec()));
    test_cases.push(("repeated", vec![7; 1000]));

    let mut rng = rand::rng();

    let random_data: Vec<u8> = (0..10000).map(|_| rng.random()).collect();
    test_cases.push(("random bytes", random_data));

    let random_alphanumeric: Vec<u8> = (0..10000).map(|_|
        ALPHANUMERIC[rng.random_range(0..ALPHANUMERIC.len())]
    ).collect();
    test_cases.push(("random alphanumeric", random_alphanumeric));

    let base: Vec<u8> = (0..1000).map(|_|
        DNA[rng.random_range(0..DNA.len())]
    ).collect();
    let mut repetitive_dna = Vec::new();
    for _ in 0..10 {
        for &byte in base.iter() {
            if rng.random::<f32>() < 0.03 {
                repetitive_dna.push(DNA[rng.random_range(0..DNA.len())]);
            } else {
                repetitive_dna.push(byte);
            }
        }
    }
    test_cases.push(("repetitive DNA", repetitive_dna));

    for (test_case, data) in test_cases {
        rans_test(&data, flags, test_case);
    }
}

//-----------------------------------------------------------------------------

#[test]
fn rans_zero() {
    let flags = RANSFlags::zero_order();
    rans_test_cases(flags);
}

#[test]
fn rans_zero_rle() {
    let flags = RANSFlags::zero_order().with_rle();
    rans_test_cases(flags);
} 

#[test]
fn rans_zero_pack() {
    let flags = RANSFlags::zero_order().with_pack();
    rans_test_cases(flags);
}

#[test]
fn rans_zero_rle_pack() {
    let flags = RANSFlags::zero_order().with_rle().with_pack();
    rans_test_cases(flags);
}

#[test]
fn rans_first() {
    let flags = RANSFlags::first_order();
    rans_test_cases(flags);
}

#[test]
fn rans_first_rle() {
    let flags = RANSFlags::first_order().with_rle();
    rans_test_cases(flags);
}

#[test]
fn rans_first_pack() {
    let flags = RANSFlags::first_order().with_pack();
    rans_test_cases(flags);
}

#[test]
fn rans_first_rle_pack() {
    let flags = RANSFlags::first_order().with_rle().with_pack();
    rans_test_cases(flags);
}

//-----------------------------------------------------------------------------
