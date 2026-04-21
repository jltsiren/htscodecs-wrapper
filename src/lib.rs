// FIXME: document, implement, tests

use std::os::raw::{c_uint, c_int, c_uchar};

// FIXME: These are the raw bindings. Create more usable wrappers.
unsafe extern "C" {
    #[must_use]
    pub fn rans_compress_bound_4x16(size: c_uint, order: c_int) -> c_uint;

    #[must_use]
    pub fn rans_compress_to_4x16(
        input: *const c_uchar,
        input_size: c_uint,
        output: *mut c_uchar,
        output_size: *mut c_uint, // Allocated size; replaced with actual size.
        order: c_int,
    ) -> *mut c_uchar; // NULL on failure.

    #[must_use]
    pub fn rans_uncompress_to_4x16(
        input: *const c_uchar,
        input_size: c_uint,
        output: *mut c_uchar,
        output_size: *mut c_uint, // Allocated size; replaced with actual size.
    ) -> *mut c_uchar; // NULL on failure.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_and_uncompress() {
        let input = b"Hello, world! Hello, world! Hello, world!";
        let input_size = input.len() as c_uint;

        let mut bound = unsafe { rans_compress_bound_4x16(input_size, 0) };
        let mut compressed = vec![0u8; bound as usize];
        let compress_result = unsafe {
            rans_compress_to_4x16(
                input.as_ptr(),
                input_size,
                compressed.as_mut_ptr(),
                &mut bound,
                0,
            )
        };
        assert!(!compress_result.is_null(), "Compression failed");
        compressed.resize(bound as usize, 0);

        let mut decompressed = vec![0u8; input.len()];
        let mut decompressed_size = decompressed.len() as c_uint;
        let uncompress_result = unsafe {
            rans_uncompress_to_4x16(
                compressed.as_ptr(),
                compressed.len() as c_uint,
                decompressed.as_mut_ptr(),
                &mut decompressed_size,
            )
        };
        assert!(!uncompress_result.is_null(), "Decompression failed");
        decompressed.resize(decompressed_size as usize, 0);

        assert_eq!(decompressed.len(), input.len(), "Decompressed size does not match original");
        assert_eq!(decompressed, input, "Decompressed data does not match original");
    }
}
