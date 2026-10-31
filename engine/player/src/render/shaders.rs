//! Every shader source the player compiles, each as one fixed text, and the same shaders compiled
//! ahead of time for the graphics APIs that take no HLSL: SPIR-V for Vulkan and MSL for Metal,
//! made from these texts by SDL_shadercross (`tools/shaders`, through
//! `examples/precompile_shaders.rs`) and kept in `engine/player/shaders`. On Windows the player
//! compiles these texts itself with FXC, as it always has; elsewhere it takes the copies made
//! ahead, found by the text they were made from, so a changed shader with no copies made again is
//! told at once (and by `every_source_has_its_shaders_made`).
//!
//! The two HDR10 readouts (`MENU_PQ_WHITE`, `OVERLAY_PQ_WHITE`) carry the screen's white in their
//! text and are made only on Windows, where the HDR10 output is.

/// The table: its parts, lights, flashers and bulbs.
pub const SCENE: &str = include_str!("player.hlsl");
pub const OVERLAY: &str = include_str!("overlay.hlsl");
/// The score display's dots, compiled after the overlay's or the scene's shaders.
pub const DMD: &str = include_str!("dmd.hlsl");
/// The score display's own window: VPX's newer display renderer.
pub const DISPLAY: &str = include_str!("display.hlsl");
/// VPX's look: the final pass, tonemapping the table's picture into the window's colours.
pub const TONEMAP: &str = include_str!("tonemap.hlsl");
/// VPX's look: VPX's basic material, and its environment's irradiance.
pub const MATERIAL: &str = include_str!("material.hlsl");
/// VPX's look: bloom, the glow around what's brighter than VPX's threshold.
pub const BLOOM: &str = include_str!("bloom.hlsl");
/// VPX 5436's embedded windows: a window's picture taken into the table's (`embed.rs`).
pub const EMBED: &str = include_str!("embed.hlsl");
pub const EFFECTS: &str = include_str!("effects.hlsl");
pub const STILL: &str = include_str!("still.hlsl");
pub const MENU: &str = include_str!("menu.hlsl");
pub const POST: &str = concat!(include_str!("post_common.hlsl"), include_str!("post.hlsl"));
pub const SMAA: &str = concat!(include_str!("post_common.hlsl"), include_str!("smaa.hlsl"), include_str!("smaa_passes.hlsl"));

/// A score display shown as a flasher in the table.
pub const DMD_SCENE: &str = concat!("#define DMD_SCENE\n", include_str!("player.hlsl"), "\n", include_str!("dmd.hlsl"));
/// A score display on the backdrop.
pub const DMD_OVERLAY: &str = concat!("#define DMD_OVERLAY\n", include_str!("overlay.hlsl"), "\n", include_str!("dmd.hlsl"));
/// VPX's material seeing through to what's behind it.
pub const MATERIAL_REFRACTION: &str = concat!("#define REFRACTION\n", include_str!("material.hlsl"));
/// VPX's ball.
pub const MATERIAL_BALL: &str = concat!("#define BALL\n", include_str!("material.hlsl"));
/// VPX's classic lights, with pictures of their own.
pub const MATERIAL_CLASSIC_LIGHT: &str = concat!("#define CLASSIC_LIGHT\n", include_str!("material.hlsl"));
/// The environment's irradiance, made once a table loads.
pub const MATERIAL_IRRADIANCE: &str = concat!("#define IRRADIANCE\n", include_str!("material.hlsl"));

/// Every source made ahead, by the name its shaders' files take.
pub const SOURCES: &[(&str, &str)] = &[
    ("scene", SCENE),
    ("overlay", OVERLAY),
    ("display", DISPLAY),
    ("tonemap", TONEMAP),
    ("material", MATERIAL),
    ("bloom", BLOOM),
    ("embed", EMBED),
    ("effects", EFFECTS),
    ("still", STILL),
    ("menu", MENU),
    ("post", POST),
    ("smaa", SMAA),
    ("dmd_scene", DMD_SCENE),
    ("dmd_overlay", DMD_OVERLAY),
    ("material_refraction", MATERIAL_REFRACTION),
    ("material_ball", MATERIAL_BALL),
    ("material_classic_light", MATERIAL_CLASSIC_LIGHT),
    ("material_irradiance", MATERIAL_IRRADIANCE),
];

/// The entry points a source may hold, each with whether it's a vertex shader: the functions
/// named as ours are, `vs_…` or `…_vs` for vertex shaders, `fs_…` or `…_fs` for fragment ones.
/// Some are left out of a source by its defines; making them says so, and they're skipped.
pub fn entry_points(source: &str) -> Vec<(&str, bool)> {
    let mut out: Vec<(&str, bool)> = Vec::new();
    for line in source.lines() {
        let line = line.trim_start();
        // `<type> <name>(`
        let Some((_, rest)) = line.split_once(char::is_whitespace) else { continue };
        let Some((name, _)) = rest.trim_start().split_once('(') else { continue };
        let name = name.trim_end();
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_') {
            continue;
        }
        let vertex = name.starts_with("vs_") || name.ends_with("_vs");
        let fragment = name.starts_with("fs_") || name.ends_with("_fs");
        if (vertex || fragment) && !out.iter().any(|(n, _)| *n == name) {
            out.push((name, vertex));
        }
    }
    out
}

/// A text's fingerprint (64-bit FNV-1a), naming the source a shader was made from.
pub const fn fingerprint(text: &str) -> u64 {
    let bytes = text.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0;
    while i < bytes.len() {
        hash = (hash ^ bytes[i] as u64).wrapping_mul(0x0100_0000_01b3);
        i += 1;
    }
    hash
}

/// A shader made ahead: its source's fingerprint and name, its entry point, what it binds as
/// SDL_shadercross read it, and its code for each API.
pub struct Made {
    pub source: u64,
    pub source_name: &'static str,
    pub entry: &'static str,
    pub vertex: bool,
    pub samplers: u32,
    pub uniform_buffers: u32,
    pub spirv: &'static [u8],
    pub msl: &'static str,
}

#[cfg(any(not(windows), test))]
#[rustfmt::skip]
mod made;

/// The shader made ahead from `source`'s entry point `entry`, if there is one.
#[cfg(any(not(windows), test))]
pub fn made(source: &str, entry: &str) -> Option<&'static Made> {
    let print = fingerprint(source);
    made::MADE.iter().find(|m| m.source == print && m.entry == entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_points_are_found_by_their_names() {
        let source = "float4 menu_fs(MenuVaryings v) : SV_Target\n{\n}\nfloat3 menu_to_linear(float3 c)\nMenuVaryings menu_vs(uint id : SV_VertexID)\n";
        assert_eq!(entry_points(source), vec![("menu_fs", false), ("menu_vs", true)]);
    }

    /// A shader changed with its copies not made again would draw its old self, or nothing,
    /// on Vulkan and Metal: `cargo run -p player --example precompile_shaders` (tools/shaders).
    #[test]
    fn every_source_has_its_shaders_made() {
        for (name, text) in SOURCES {
            let print = fingerprint(text);
            assert!(
                made::MADE.iter().any(|m| m.source == print),
                "the shaders of `{name}` weren't made again since it changed: see tools/shaders/README.md"
            );
            for m in made::MADE.iter().filter(|m| m.source_name == *name) {
                assert_eq!(m.source, print, "`{name}`'s {} was made from an older text", m.entry);
            }
        }
    }
}
