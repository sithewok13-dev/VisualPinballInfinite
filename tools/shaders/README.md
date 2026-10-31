# The player's shaders, made ahead for Metal and Vulkan

The player's shaders are HLSL (`engine/player/src/render/*.hlsl`). On Windows the player compiles them itself when it starts, with FXC, which comes with Windows, for Direct3D 12. Elsewhere there's no FXC and Direct3D: SDL3 GPU draws on **Metal** on Apple's systems (macOS, and iOS to come) and on **Vulkan** on others, and these take MSL and SPIR-V. So every shader is made ahead, from the same HLSL, by [SDL_shadercross](https://github.com/libsdl-org/SDL_shadercross) (zlib licence): HLSL to SPIR-V through Microsoft's DXC, and SPIR-V to MSL through SPIRV-Cross. The copies are kept in the repository, in `engine/player/shaders`, so building the player anywhere needs none of these tools.

- `engine/player/src/render/shaders.rs`: every source the player compiles, each one fixed text (the defines a variant takes written into it), and how the player finds a shader made ahead (by its source's fingerprint and its entry point).
- `engine/player/examples/precompile_shaders.rs`: makes them. Every entry point of every source (`vs_…`, `…_vs`, `fs_…`, `…_fs`), as SPIR-V and MSL, with what each binds as shadercross reads it, and the list the player finds them by (`engine/player/src/render/shaders/made.rs`).
- `setup.sh`: builds shadercross on Linux (on Windows, from WSL) into `vpx-build/shadercross` at fixed versions: DXC's own release build, SPIRV-Cross and our copy of SDL from source.
- `precompile.sh`: makes them again with that build, and checks them.

## When a shader changes

```
tools/shaders/setup.sh        # once
tools/shaders/precompile.sh
```

and the changed files in `engine/player/shaders` and `made.rs` go in the same commit as the HLSL. A shader changed without this is caught by the test `render::shaders::tests::every_source_has_its_shaders_made` (on every system, Windows too), and by the player elsewhere, which then won't start a table rather than draw with an older shader.

The same tools give the same bytes: made twice, from nothing, the files are identical.

## What the HLSL keeps to

DXC reads HLSL 2021, which FXC's HLSL doesn't always pass:

- No `?:` on vectors (`c <= 0.04045 ? lo : hi` with `float3`s): it's written a component at a time (`display_srgb` in `display.hlsl`), which FXC compiles as before.
- A texture read only with `Load`, no sampler used beside it, is a storage texture to shadercross. The player binds it with the others, by its register, as Direct3D 12 does, and on Vulkan and Metal its slot is its register's too, so it's counted with the samplers (`still.hlsl`, `display_dmd_fs`, SMAA's passes).
- A texture or sampler a shader never reads is left out of it, as FXC leaves it out: the player checks a shader reads no more than it binds, not exactly as many.
- The two HDR10 readouts (`MENU_PQ_WHITE`, `OVERLAY_PQ_WHITE`) carry the screen's white in their text, so they're made only on Windows, where the HDR10 output is; elsewhere the player doesn't offer it yet.

## Checked so far

- **Vulkan**, on Linux with Mesa's software renderer (lavapipe): VPX's test table (`tests/assets/test000-default-table.vpx` in VPX's repository) in VPX's look at 1920x1080, against VPX's own reference picture of it: 0.30% of pixels more than 32 levels apart, all along edges (antialiasing), and the score display, which the table's script fills (scripts don't run outside Windows yet). Ambient occlusion and still parts drawn once crash inside lavapipe for now (in its `vkAllocateDescriptorSets`, with no validation error), so that picture has `--ao off --still-parts every-frame`.
- **Metal**: `.github/workflows/apple-metal.yml` compiles every MSL shader with Apple's own compiler for macOS and for iOS, builds the player on macOS and draws the same table there.
