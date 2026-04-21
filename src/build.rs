use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use std::{env, fs};

use cc::Build;

//-----------------------------------------------------------------------------

fn main() -> Result<(), String> {
    // Input directories.
    let project_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(
        |e| format!("Failed to get CARGO_MANIFEST_DIR environment variable: {}", e),
    )?);
    let htscodecs_dir = project_root.join("htscodecs");

    // Output directory.
    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(
        |e| format!("Failed to get OUT_DIR environment variable: {}", e),
    )?);
    fs::create_dir_all(&out_dir).map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Tell Cargo to include the output directory and the htscodecs directory in the include path.
    let paths = env::join_paths(&[&out_dir, &htscodecs_dir])
        .map_err(|e| format!("Failed to join paths: {}", e))?;
    let paths = paths.to_str().ok_or_else(
        || String::from("Paths are not valid UTF-8")
    )?;
    println!("cargo:include={}", paths);

    // Configure the C compiler.
    let mut compiler = Build::new();
    compiler.include(&out_dir);
    compiler.include(&htscodecs_dir);
    compiler.pic(true);
    if let Ok(rustflags) = env::var("RUSTFLAGS") {
        if rustflags.contains("target-cpu=native") {
            compiler.flag("-march=native");
        }
    }
    compiler.warnings(false); // Suppress warnings about signed-to-unsigned comparisons.

    // Select the source files to compile.
    let source_files = [
        "htscodecs/htscodecs/rANS_static4x16pr.c",

        "htscodecs/htscodecs/rANS_static32x16pr.c",
        "htscodecs/htscodecs/rANS_static32x16pr_avx2.c",
        "htscodecs/htscodecs/rANS_static32x16pr_avx512.c",
        "htscodecs/htscodecs/rANS_static32x16pr_neon.c",
        "htscodecs/htscodecs/rANS_static32x16pr_sse4.c",

        "htscodecs/htscodecs/pack.c",
        "htscodecs/htscodecs/htscodecs.c",
        "htscodecs/htscodecs/rle.c",
        "htscodecs/htscodecs/utils.c",
    ];
    for file in &source_files {
        compiler.file(file);
    }

    // Write a hacky config.h and a version.h.
    write_config_h(&out_dir)?;
    write_version_h(&out_dir)?;

    compiler.compile("htscodecs");

    Ok(())
}

//-----------------------------------------------------------------------------

// TODO: Update these when necessary.
const HTSCODECS_NAME: &str = "htscodecs";
const HTSCODECS_VERSION: &str = "1.6.6";

fn write_line_to(writer: &mut BufWriter<File>, line: &str) -> Result<(), String> {
    writer.write_all(line.as_bytes())
        .map_err(|e| format!("Failed to write to a header: {}", e))?;
    writer.write_all(b"\n")
        .map_err(|e| format!("Failed to write to a header: {}", e))?;
    Ok(())
}

fn write_config_h(out_dir: &PathBuf) -> Result<(), String> {
    let config_path = out_dir.join("config.h");
    let config_file = File::create(&config_path).map_err(|e| format!("Failed to create config.h: {}", e))?;
    let mut config_file = BufWriter::new(config_file);

    // We can ignore the CPU features, as we are not using the SIMD-enabled
    // rANS 32x16 code path.
    write_line_to(&mut config_file, "#define HAVE_AVX2 0")?;
    write_line_to(&mut config_file, "#define HAVE_AVX512 0")?;
    write_line_to(&mut config_file, "#define HAVE_BUILTIN_PREFETCH 0")?;
    write_line_to(&mut config_file, "#define HAVE_DECL___CPUID_COUNT 0")?;
    write_line_to(&mut config_file, "#define HAVE_DECL___CPUID_MAX 0")?;
    write_line_to(&mut config_file, "#define HAVE_POPCNT 0")?;
    write_line_to(&mut config_file, "#define HAVE_SSE4_1 0")?;
    write_line_to(&mut config_file, "#define HAVE_SSE3 0")?;

    // Assume that all standard headers are available.
    write_line_to(&mut config_file, "#define HAVE_DLFCN_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_FCNTL_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_INTTYPES_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_LIMITS_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_STDINT_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_STDIO_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_STDLIB_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_STRING_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_SYS_STAT_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_SYS_TYPES_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_SYS_WAIT_H 1")?;
    write_line_to(&mut config_file, "#define HAVE_UNISTD_H 1")?;
    write_line_to(&mut config_file, "#define STDC_HEADERS 1")?;

    // We do not use general-purpose compressors.
    write_line_to(&mut config_file, "#define HAVE_LIBBZ2 0")?;
    write_line_to(&mut config_file, "#undef HAVE_LIBDEFLATE")?;
    write_line_to(&mut config_file, "#define HAVE_ZLIB 0")?;

    // Package information.
    write_line_to(&mut config_file, "#define LT_OBJDIR \".libs/\"")?;
    let package_define = format!("#define PACKAGE \"{}\"", HTSCODECS_NAME);
    write_line_to(&mut config_file, &package_define)?;
    write_line_to(&mut config_file, "#define PACKAGE_BUGREPORT \"\"")?;
    let package_string_define = format!("#define PACKAGE_STRING \"{} {}\"", HTSCODECS_NAME, HTSCODECS_VERSION);
    write_line_to(&mut config_file, &package_string_define)?;
    let package_tarname_define = format!("#define PACKAGE_TARNAME \"{}\"", HTSCODECS_NAME);
    write_line_to(&mut config_file, &package_tarname_define)?;
    write_line_to(&mut config_file, "#define PACKAGE_URL \"\"")?;
    let package_version_define = format!("#define PACKAGE_VERSION \"{}\"", HTSCODECS_VERSION);
    write_line_to(&mut config_file, &package_version_define)?;
    let version_define = format!("#define VERSION \"{}\"", HTSCODECS_VERSION);
    write_line_to(&mut config_file, &version_define)?;

    Ok(())
}

fn write_version_h(out_dir: &PathBuf) -> Result<(), String> {
    let version_path = out_dir.join("version.h");
    let version_file = File::create(&version_path).map_err(|e| format!("Failed to create version.h: {}", e))?;
    let mut version_file = BufWriter::new(version_file);

    let version_define = format!("#define HTSCODECS_VERSION_TEXT \"{}\"", HTSCODECS_VERSION);
    write_line_to(&mut version_file, &version_define)?;

    Ok(())
}

//-----------------------------------------------------------------------------
