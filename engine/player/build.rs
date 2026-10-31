//! Builds the player's one piece of C++: the bridge to FFmpeg for PinUP packs' media
//! (`native/pup_av.cpp`), compiled against FFmpeg's headers in `vendor/ffmpeg`. FFmpeg
//! itself isn't linked: the bridge loads its libraries when a pack plays.

fn main() {
    // Windows only so far: the bridge loads FFmpeg's libraries the Windows way (`src/pup/av.rs`
    // stands in for it elsewhere)
    if std::env::var("CARGO_CFG_WINDOWS").is_err() {
        return;
    }
    println!("cargo:rerun-if-changed=native/pup_av.cpp");
    println!("cargo:rerun-if-changed=../../vendor/ffmpeg/include");
    cc::Build::new()
        .cpp(true)
        .file("native/pup_av.cpp")
        .include("../../vendor/ffmpeg/include")
        .flag_if_supported("/std:c++17")
        .warnings(true)
        .compile("vpi_pup_av");
}
