//! Wrapper for HTSlib codecs used in GAF-base.
//!
//! This is a minimal Rust wrapper for some [HTSlib](https://github.com/samtools/htslib) compression codecs.
//! It depends on the [htscodecs](https://github.com/samtools/htscodecs) C library.
//! Only codecs used in [GAF-base](https://github.com/jltsiren/gbz-base) are compiled.
//!
//! # Codecs
//!
//! ### rANS
//!
//! rANS (range Asymmetric Numeral Systems) is a family of fast entropy encoders.
//! The 4x16 variant with 4 parallel states and 16-bit renormalization is appropriate for compressing quality scores.
//! Zero-order and first-order models with optional run-length encoding and symbol packing can be selected using [`RANSFlags`].
//! Compression and decompression are done using [`rans_compress`] and [`rans_decompress`].
//!
//! # References
//!
//! ### Asymmetric numeral systems
//!
//! Jarek Duda:
//! **Asymmetric numeral systems: entropy coding combining speed of Huffman coding with compression rate of arithmetic coding**.\
//! arXiv, 2013.
//! DOI: [10.48550/arXiv.1311.2540](https://doi.org/10.48550/arXiv.1311.2540).
//!
//! ### HTSlib codecs
//!
//! James K. Bonfield:
//! **CRAM 3.1: advances in the CRAM file format**.\
//! Bioinformatics 38(6):1497-1503, 2022.
//! DOI: [10.1093/bioinformatics/btac010](https://doi.org/10.1093/bioinformatics/btac010).

use std::fmt::{self, Display};
use std::os::raw::{c_uint, c_int, c_uchar};

#[cfg(test)]
mod tests;

//-----------------------------------------------------------------------------

unsafe extern "C" {
    // Returns a worst-case bound on the compressed size for the given input size and flags.
    #[must_use]
    unsafe fn rans_compress_bound_4x16(size: c_uint, flags: c_int) -> c_uint;

    // Compresses the input data using rANS 4x16 with the specified flags.
    #[must_use]
    unsafe fn rans_compress_to_4x16(
        input: *const c_uchar,
        input_size: c_uint,
        output: *mut c_uchar, // Set to NULL to use a temporary buffer allocated in the C code.
        output_size: *mut c_uint, // Allocated size; replaced with actual size.
        flags: c_int,
    ) -> *mut c_uchar; // Compressed data or NULL on failure.

    // Decompresses the input data using rANS 4x16.
    #[must_use]
    unsafe fn rans_uncompress_to_4x16(
        input: *const c_uchar,
        input_size: c_uint,
        output: *mut c_uchar, // Set to NULL to use a temporary buffer allocated in the C code.
        output_size: *mut c_uint, // Allocated size; replaced with actual size.
    ) -> *mut c_uchar; // Decompressed data or NULL on failure.
}

//-----------------------------------------------------------------------------

/// Flags for rANS compression.
///
/// This assumes the 4x16 variant from htscodecs.
/// Zero-order and first-order compression with optional run-length encoding and symbol packing are supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RANSFlags {
    flags: c_int,
}

impl RANSFlags {
    const FLAG_FIRST_ORDER: c_int = 1; // Use first-order model instead of zero-order.
    const FLAG_RLE: c_int = 0x40; // Enable run-length encoding.
    const FLAG_PACK: c_int = 0x80; // Enable packing multiple symbols into a byte.

    /// Creates flags for zero-order rANS.
    pub fn zero_order() -> Self {
        Self { flags: 0 }
    }

    /// Creates flags for first-order rANS.
    pub fn first_order() -> Self {
        Self { flags: Self::FLAG_FIRST_ORDER }
    }

    /// Enables run-length encoding.
    pub fn with_rle(self) -> Self {
        Self { flags: self.flags | Self::FLAG_RLE }
    }

    /// Enables packing multiple symbols into a byte.
    pub fn with_pack(self) -> Self {
        Self { flags: self.flags | Self::FLAG_PACK }
    }
}

impl Display for RANSFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rANS 4x16 (")?;
        if self.flags & Self::FLAG_FIRST_ORDER != 0 {
            write!(f, "o1")?;
        } else {
            write!(f, "o0")?;
        }
        if self.flags & Self::FLAG_RLE != 0 {
            write!(f, ", rle")?;
        }
        if self.flags & Self::FLAG_PACK != 0 {
            write!(f, ", pack")?;
        }
        write!(f, ")")
    }
}

//-----------------------------------------------------------------------------

/// Compresses the input data using rANS 4x16 with the specified flags.
///
/// Returns the compressed data on success, or an error message on failure.
/// This allocates space for the worst-case compressed size.
/// If multiple blocks are compressed and kept in memory, it can make sense to copy the data to an appropriately sized vector.
///
/// # Errors
///
/// Returns an error if the worst-case compressed size would exceed approximately [`i32::MAX`].
/// Returns an error if the compression fails for any reason.
///
/// # Examples
///
/// ```
/// use htscodecs_wrapper::RANSFlags;
///
/// let input = b"alabar a la alabarda, alabar a la alabarda, alabar a la alabarda";
/// let flags = RANSFlags::zero_order().with_pack();
/// let compressed = htscodecs_wrapper::rans_compress(input, flags);
/// assert!(compressed.is_ok());
/// let compressed = compressed.unwrap();
/// assert!(compressed.len() < input.len());
/// assert!(compressed.capacity() > input.len());
///
/// // We provide the output size to avoid an extra copy.
/// let decompressed = htscodecs_wrapper::rans_decompress(&compressed, Some(input.len()));
/// assert!(decompressed.is_ok(), "Decompression failed: {}", decompressed.err().unwrap());
/// let decompressed = decompressed.unwrap();
/// assert_eq!(decompressed, input);
/// ```
pub fn rans_compress(input: &[u8], flags: RANSFlags) -> Result<Vec<u8>, String> {
    // Worst-case compressed size can exceed input size.
    if input.len() > 2_000_000_000 {
        return Err(String::from("Input size exceeds maximum supported size of 2 GB"));
    }

    let mut bound = unsafe { rans_compress_bound_4x16(input.len() as c_uint, flags.flags) };
    let mut compressed = Vec::with_capacity(bound as usize);
    let result = unsafe {
        rans_compress_to_4x16(
            input.as_ptr(),
            input.len() as c_uint,
            compressed.as_mut_ptr(),
            &mut bound,
            flags.flags,
        )
    };
    if result.is_null() {
        return Err(format!("Compression failed with flags {}", flags.flags));
    }
    unsafe { compressed.set_len(bound as usize) };

    Ok(compressed)
}

/// Decompresses the input data using rANS 4x16.
///
/// Returns the decompressed data on success, or an error message on failure.
/// If `output_size` is provided, the data is decompressed directly into the output buffer.
/// Otherwise it is decompressed into a temporary buffer allocated in the C code and copied to a Rust vector.
///
/// See [`rans_compress`] for an example.
///
/// # Errors
///
/// Returns an error if input or output size reaches or exceeds [`i32::MAX`].
/// Returns an error if the decompression fails for any reason.
pub fn rans_decompress(input: &[u8], output_size: Option<usize>) -> Result<Vec<u8>, String> {
    if input.len() >= (i32::MAX as usize) {
        return Err(String::from("Input size does not fit in i32"));
    }

    // Allocate an output buffer or tell the C code to use a temporary buffer.
    let (mut output, mut output_size, temporary_buffer) = if let Some(size) = output_size {
        if size >= (i32::MAX as usize) {
            return Err(String::from("Output size does not fit in i32"));
        }
        (Vec::with_capacity(size), size as c_uint, false)
    } else {
        (Vec::new(), 0, true)
    };
    let output_ptr = if temporary_buffer {
        std::ptr::null_mut()
    } else {
        output.as_mut_ptr()
    };


    // Actual decompression.
    let result = unsafe {
        rans_uncompress_to_4x16(
            input.as_ptr(),
            input.len() as c_uint,
            output_ptr,
            &mut output_size,
        )
    };
    if result.is_null() {
        eprintln!("Decompression failed with input size {} and output size {}", input.len(), output_size);
        return Err(String::from("Decompression failed"));
    }

    // Now handle the output.
    if temporary_buffer {
        output.reserve(output_size as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(result, output.as_mut_ptr(), output_size as usize);
            libc::free(result as *mut libc::c_void);
        }
    }
    unsafe { output.set_len(output_size as usize) };

    Ok(output)
}

//-----------------------------------------------------------------------------
