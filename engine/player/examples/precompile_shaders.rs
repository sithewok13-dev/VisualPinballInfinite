//! Makes the player's shaders ahead for the APIs that take no HLSL (`src/render/shaders.rs`):
//! every entry point of every source, through SDL_shadercross's command line, as SPIR-V
//! (Vulkan) and MSL (Metal), into `engine/player/shaders`, with what each binds as
//! shadercross reads it; and the list the player finds them by, `src/render/shaders/made.rs`.
//! Run again whenever a shader changes (`tools/shaders/README.md`):
//!
//! ```text
//! cargo run -p player --example precompile_shaders [-- <shadercross>]
//! ```
//!
//! shadercross is `$SHADERCROSS`, else the argument, else `shadercross` on the path.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use player::render::shaders::{SOURCES, entry_points, fingerprint};

fn main() -> ExitCode {
    let tool = std::env::var_os("SHADERCROSS").map(PathBuf::from).or_else(|| std::env::args_os().nth(1).map(PathBuf::from)).unwrap_or_else(|| "shadercross".into());
    match run(&tool) {
        Ok(n) => {
            println!("{n} shaders made");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(tool: &Path) -> Result<usize, String> {
    let player = Path::new(env!("CARGO_MANIFEST_DIR"));
    let work = std::env::temp_dir().join("vpi-precompile-shaders");
    // all made here first, and only then put in place: the player builds on the list as it was
    // until every shader is made
    let fresh = work.join("shaders");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&fresh).map_err(|e| format!("{}: {e}", fresh.display()))?;
    let mut list = String::from(
        "// Made by `cargo run -p player --example precompile_shaders` (tools/shaders/README.md): not to\n\
         // be edited by hand. Each shader's source fingerprint, entry point, what it binds, and its code.\n\n\
         use super::Made;\n\npub const MADE: &[Made] = &[\n",
    );
    let mut made = 0;
    for (name, text) in SOURCES {
        let hlsl = work.join(format!("{name}.hlsl"));
        std::fs::write(&hlsl, text).map_err(|e| format!("{}: {e}", hlsl.display()))?;
        for (entry, vertex) in entry_points(text) {
            let stage = if vertex { "vertex" } else { "fragment" };
            let file = format!("{name}.{entry}");
            let json = work.join(format!("{file}.json"));
            // the reflection first: an entry point this source's defines leave out is skipped
            match shadercross(tool, &hlsl, "JSON", stage, entry, &json)? {
                Compiled::Missing => continue,
                Compiled::Made => {}
            }
            for (dest, ext) in [("SPIRV", "spv"), ("MSL", "msl")] {
                if let Compiled::Missing = shadercross(tool, &hlsl, dest, stage, entry, &fresh.join(format!("{file}.{ext}")))? {
                    return Err(format!("{file}: shadercross made its reflection but not its {dest}"));
                }
            }
            let reflection: serde_json::Value = serde_json::from_slice(&std::fs::read(&json).map_err(|e| e.to_string())?).map_err(|e| format!("{file}.json: {e}"))?;
            let count = |key: &str| reflection.get(key).and_then(serde_json::Value::as_u64).unwrap_or(0);
            // a texture read only with `Load`, no sampler beside it, is a storage texture to
            // shadercross; the player binds it with the others (D3D12's way), and its slot is
            // its register's on Vulkan and Metal alike, so it counts among them
            if count("storage_buffers") != 0 {
                return Err(format!("{file}: storage buffers aren't bound by the player"));
            }
            list.push_str(&format!(
                "    Made {{ source: {:#018x}, source_name: \"{name}\", entry: \"{entry}\", vertex: {vertex}, samplers: {}, uniform_buffers: {}, spirv: include_bytes!(\"../../../shaders/{file}.spv\"), msl: include_str!(\"../../../shaders/{file}.msl\") }},\n",
                fingerprint(text),
                count("samplers") + count("storage_textures"),
                count("uniform_buffers"),
            ));
            made += 1;
        }
    }
    list.push_str("];\n");
    // the old ones go, so a shader gone from its source leaves no file behind
    let out = player.join("shaders");
    std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    for entry in std::fs::read_dir(&out).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|x| x == "spv" || x == "msl") {
            std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    for entry in std::fs::read_dir(&fresh).map_err(|e| e.to_string())?.flatten() {
        let to = out.join(entry.file_name());
        std::fs::copy(entry.path(), &to).map_err(|e| format!("{}: {e}", to.display()))?;
    }
    let rs = player.join("src").join("render").join("shaders").join("made.rs");
    std::fs::create_dir_all(rs.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&rs, list).map_err(|e| format!("{}: {e}", rs.display()))?;
    Ok(made)
}

enum Compiled {
    Made,
    /// The source holds no such entry point (its defines leave it out).
    Missing,
}

fn shadercross(tool: &Path, hlsl: &Path, dest: &str, stage: &str, entry: &str, output: &Path) -> Result<Compiled, String> {
    let run = Command::new(tool)
        .arg(hlsl)
        .args(["-s", "HLSL", "-d", dest, "-t", stage, "-e", entry, "-o"])
        .arg(output)
        .output()
        .map_err(|e| format!("couldn't run {}: {e} (tools/shaders/README.md)", tool.display()))?;
    if run.status.success() {
        return Ok(Compiled::Made);
    }
    let said = format!("{}{}", String::from_utf8_lossy(&run.stderr), String::from_utf8_lossy(&run.stdout));
    if said.contains("missing entry point definition") {
        return Ok(Compiled::Missing);
    }
    Err(format!("{} {entry} ({dest}):\n{}", hlsl.display(), said.trim()))
}
