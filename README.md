# Rust wrapper for HTSlib compression codecs

This is a minimal Rust wrapper for some [HTSlib](https://github.com/samtools/htslib) compression codecs.
It currently depends on [htscodecs](https://github.com/samtools/htscodecs) version 1.6.6.
Only codecs used in [GAF-base](https://github.com/jltsiren/gbz-base) are compiled.

## Notes

* The included `.cargo/config.toml` sets the target CPU to `native`.
