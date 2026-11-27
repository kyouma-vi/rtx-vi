# NVIDIA For for CGBJ: RTX-VI

> **Test system (user-reported):** NVIDIA GeForce RTX 5070 Ti · 96 GB DDR5 · AMD Ryzen 9 9900X3D.
>
> **Black sun: fixed and confirmed in-game.**
>
> **NVIDIA RT hardware acceleration:** native RT-core support is the target of the experimental V4 renderer, with the goal of significantly improving rendering performance. Native DXR traversal is not yet implemented; its performance gains have not been verified.
>
> **V5 — 60 FPS proof of concept:** the experimental patch raises the game's target frame rate from 30 to 60 FPS. Current user-reported results on the test system are **25–45 FPS in gameplay and 60 FPS in the main menu**. Stable 60 FPS in gameplay requires further optimization.
>
> **AMD boys go home, it's not for you.**

## How to run

1. Install [Rust](https://rustup.rs/) and [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with Desktop development with C++ and a Windows SDK.
2. Extract this repository and open PowerShell in its root directory.
3. Build the release executable:

   ```powershell
   cargo build --release --locked -p victory
   ```

4. Set the path to your own game dump (the folder must contain `game_scarlett_master.exe`):

   ```powershell
   $env:VICTORY_GAME_DIR = "C:\GTA6_Xbox"
   ```

5. Launch the experimental V4 original-lighting renderer:

   ```powershell
   .\Play-RT-V4.cmd
   ```

   This enables the original lighting path and experimental BVH address optimization. Native DXR traversal is not yet implemented, and this mode can be very slow.

   For the faster balanced renderer with the black-sun fix and shadow filtering, use:

   ```powershell
   Remove-Item Env:VICTORY_AMBIENT -ErrorAction SilentlyContinue
   .\Play-VIctory.cmd pareto-residency
   ```

   If blocky ambient lighting persists, try `$env:VICTORY_AMBIENT = "0"` before launching the balanced renderer. This may make shadows darker.

### Experimental 60 FPS mode (proof of concept)

The V5 patch raises the game's internal frame-rate target from 30 to 60 FPS.
**This is a proof of concept, not a guarantee of 60 FPS.** It is not frame
generation and does not make the renderer itself faster.

Current user-reported performance on the RTX 5070 Ti test system:

| Scenario | Observed FPS |
|---|---:|
| Gameplay | 25–45 FPS |
| Main menu | 60 FPS |

More CPU and GPU optimizations are required to achieve a stable 60 FPS in
gameplay. Actual performance depends on the scene and rendering settings;
the original-lighting path can be substantially slower.

#### Run with the 60 FPS patch

Build the updated runtime and set your game directory as described above.
For the faster balanced renderer:

```powershell
.\Play-VIctory.cmd pareto-residency -60fps
```

For the experimental original-lighting V4 renderer:

```powershell
.\Play-RT-V4.cmd -60fps
```

#### Run without the 60 FPS patch (default 30 FPS target)

For the faster balanced renderer:

```powershell
.\Play-VIctory.cmd pareto-residency
```

For the experimental original-lighting V4 renderer:

```powershell
.\Play-RT-V4.cmd
```

Simply omit `-60fps` to restore the normal frame-rate target. For DLSS, add
`-Fps60` to the existing `Play-DLSS.ps1` command to enable the patch, or omit
it to disable the patch. The runtime also accepts
`victory run <exe> --console series-s --execute --60fps`.

The patch changes both verified title-side target-rate loads to 60 FPS,
retaining the engine's own period calculation and refresh-divisor selection.
It checks the full executable BLAKE3 and both instruction signatures before
changing a separate cached copy; the original executable is never edited.
An unsupported build fails explicitly.

**Known limitations:** this is not a native DXR implementation. Simulation,
physics, animation, audio sync and cutscenes at 60 FPS still require in-game
validation. See [FPS60.md](FPS60.md) for validation and troubleshooting.

Keep the shader cache between launches to avoid unnecessary recompilation. The first run may stutter while shaders are compiled.

A source-only fork of [codec-d/cool_fox](https://github.com/codec-d/cool_fox) (`f4b48ad`) for
NVIDIA cards: DLSS in place of the title's FSR upscaler, and fixes for a black sun and a banded
sky. **This repository contains no game files, no third-party binaries and no `.exe`: there is
no prebuilt release, you build it from source (below).** You need your own copy of the game, and
you download OptiScaler and NVIDIA's DLSS DLL yourself.

> **Work in progress.** DLSS support has only just been implemented: it runs and looks clearly
> better than the plain upscale, but it still needs polishing (see *Known problems*).

Earlier upstream benchmark: RTX 5060 Ti 16 GB, Ryzen 7 7800X3D, 32 GB RAM, Windows 11, game build
1.0.2.1, Series S profile: about **15 fps median** in the open world (10.7-21.3), no errors in
8-minute sessions, with or without DLSS.

## What this fork changes

| Change | Switch | From |
|---|---|---|
| The title's FSR calls go to a PC FidelityFX DLL; with OptiScaler beside the runtime they become **DLSS** | `VICTORY_FSR_DLL=<amd_fidelityfx_dx12.dll>` | ported from [dudegio/Neptune-GDeflate](https://github.com/dudegio/Neptune-GDeflate) |
| NaN and infinite colours and image stores become 0: fixes the **black sun**, the **banded sky** and the dark outdoor light on NVIDIA | `VICTORY_NAN_SCRUB=1` | ported from [trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA) |
| Opt-in patches of the 1.0.2.1 executable (each checked by hash and bytes): internal render scale, the title's own upscaler, watchdog | `VICTORY_RENDER_SCALE`, `VICTORY_OWN_UPSCALER`, `VICTORY_NO_WATCHDOG` | from VIctory-NVIDIA (vitutu1/victory-perf, ZigZagBoi1/ScarlettPort) |
| The ambient stand-in's air pass no longer turns an infinite lamp value into NaN | always | this fork |
| 32-bit float stand-in for R9G9B9E5 render targets (did not fix the sun; kept as an experiment) | `VICTORY_E5_STAND_IN=32` | this fork |

## Run it

1. Build: install the pinned Rust toolchain, the MSVC C++ build tools and a Windows SDK, then
   `cargo build --release --locked -p victory`.
2. For DLSS, download:
   - **OptiScaler** from its official releases, <https://github.com/optiscaler/OptiScaler/releases>
     (tested: 0.9.5), and extract it to a folder of its own (not the game's folder);
   - **`nvngx_dlss.dll`** from NVIDIA's DLSS SDK, <https://github.com/NVIDIA/DLSS>
     (`lib/Windows_x86_64/rel/`, tested: 310.9.16).
3. Start:

```powershell
.\Play-DLSS.ps1 -Game D:\MyDump\game_scarlett_master.exe `
    -OptiScaler D:\Tools\Optiscaler_0.9.5 -Dlss D:\Tools\nvngx_dlss.dll
```

Without `-OptiScaler`/`-Dlss` it starts with the same settings and the title's own upscaler.
`-DataDir <folder>` uses another save folder, `-RayTracing` turns the title's ray tracing on,
`-Log` writes `target\release\last-run.log`. Nothing is written to the game's folder: OptiScaler
is set up in `target\release\opti`. The first run builds the shader cache and stutters.

## NVIDIA stability patch

- NaN/Infinity scrubbing now checks IEEE-754 integer bits, avoiding unordered float comparisons that may be optimized away by GPU compilers. Finite values are preserved exactly.
- The scrub switch is included in shader cache keys; changing it cannot reuse a translation made with the other setting. The CMD launcher now enables the scrub too (unless overridden).
- Concurrent requests for the same shader are serialized inside the process, avoiding duplicate translations and disk-cache publication races. Different shaders generally still compile in parallel.
- Optional `-WarmShaders -WarmJobs 2` on `Play-DLSS.ps1` precompiles previously used pipelines before launch. Run the game once to collect pipelines; warm-up is not a substitute for first-run discovery. The custom `VICTORY_CACHE_DIR` is respected.
- The patched translator uses new cache keys. Initial compilation may still stutter; do not delete the cache between runs.

Validation: source review only in a Linux environment without the pinned Rust toolchain, Windows D3D12, NVIDIA GPU or game. The added concurrency test is not executed. The black-sun fix has since been confirmed by the user; frametime improvement still needs measurement. No FPS increase is claimed. DLSS still requires non-thin mode; RT and frame-generation behavior are unchanged.

## Known problems

- **DLSS is new and unpolished**: it was just added and tested on one PC for a few sessions.
  Expect rough edges; it needs `VICTORY_THIN=0` (below), frame generation is not supported yet.
- **Lighting: magenta/purple on leaves, fabric and some interiors, dark rooms.** These are known
  to come from the title's ray tracing not running (its reflection image keeps its
  "uninitialised" colour). Turning ray tracing on (`-RayTracing`) fixes them, but it is still too
  costly to use: about 11 fps instead of 15 on an RTX 5060 Ti. Making it affordable is the next
  step (avatarmanz/VIctory-7900XTX issue #71).
- **Series S renders at 1280x720.** The title's FSR 4 refuses any other size, and its own
  upscaler stops at a breakpoint above 2/3 scale. Series X with `VICTORY_RENDER_SCALE=0.5`
  (a 1920x1080 world) runs at about 14.7 fps but is not what the launcher uses.
- One weather-particle shader (`weatherParticle:CS_BeginUpdate`) crashes NVIDIA's shader compiler
  and is left out.
- DLSS needs `VICTORY_THIN=0` (the launcher sets it): Neptune's catch-up for recorded lists is
  not ported.

## Credits

[VIctory](https://github.com/Marvisak/VIctory) and
[fxceless/victory-fork](https://github.com/fxceless/victory-fork);
[avatarmanz/VIctory-7900XTX](https://github.com/avatarmanz/VIctory-7900XTX);
[codec-d/cool_fox](https://github.com/codec-d/cool_fox);
[trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA);
[dudegio/Neptune-GDeflate](https://github.com/dudegio/Neptune-GDeflate) (Neptune by Traffic and
zm0kie). All GPL-3.0-or-later, as this fork. OptiScaler and DLSS are separate projects under their
own licences and are not included.

---

# cool_fox's README

*(Kept as published by cool_fox. Its "prebuilt Windows build" and `Play-VIctory.cmd` refer to
cool_fox's own releases, which this fork does not provide: build from source and use
`Play-DLSS.ps1` above.)*

# VIctory on an RX 7900 XTX

## Run the codec_d Windows build

Download and extract the Windows x64 ZIP from the [private releases](https://github.com/codec-d/cool_fox/releases/latest). Open PowerShell in the extracted folder and point the launcher at your own complete Xbox dump.

```powershell
$env:VICTORY_GAME_DIR = 'D:\Games\MyDump'
.\Play-VIctory.cmd pareto-residency
```

The launcher selects Series S and the RTX-tested residency settings. It builds its shader cache locally under `%LOCALAPPDATA%\VIctory\cache-pareto`. Set `VICTORY_CACHE_DIR` to reuse an existing cache outside the checkout. Windows x64, an AVX2 CPU, and a Direct3D 12 GPU with shader model 6.6 and mesh shader support are required.

To build from this checkout, install the pinned Rust toolchain and the MSVC C++ build tools with a Windows SDK, then run `cargo build --release --locked -p victory`. The same launcher finds `target\release\victory.exe`. The sections below describe the upstream AMD fork and its separate launcher.

A fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer
that runs an Xbox Series title natively on Windows from files the user supplies.
This one is about **speed on AMD RDNA 3**: the world at 30 frames a second on
one PC, without giving up the picture. It is not there yet.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## Where it stands (2026-11-25)

Tested on one machine only: Windows 11, Intel i9-12900K, 64 GB RAM, Radeon
RX 7900 XTX, 1440p, the Series X profile.

| | |
|---|---|
| Menus, videos, loading screens | 30 fps |
| Prologue, indoors | 50-75 ms a frame (13-20 fps) |
| First mission with the boat | 46-53 ms (19-22 fps) |
| Street chase with many cars and people | 100-165 ms (6-10 fps), single frames far longer |
| Outdoors on foot, the camp | 60-90 ms |
| Flying | 42-57 ms |
| Longest session so far | 41 minutes, no crash |
| Memory | 40-46 GB of private memory: **64 GB of RAM recommended** |

It started at 280 ms a frame in the same scenes two days earlier.

What is known to be wrong:

- Trees, plants and far buildings flicker outdoors in some places.
- Rooms with little direct light are too dark without `-RayTracing`.
- Textures of what just came into view appear late while the title streams.
- The first run with a new driver or a new build stutters while the driver makes
  pipelines (`victory warm` makes them ahead).
- Recording a frame in the world makes the title's own watchdog end it.

## How to run it

You need Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with shader
model 6.6 and mesh shaders, Rust (the toolchain is pinned), the MSVC build
tools, and your own dump of the title.

```powershell
git clone https://github.com/avatarmanz/VIctory-7900XTX
cd VIctory-7900XTX
copy victory.example.toml victory.toml      # then set your paths, outside this folder
cargo build --locked --profile diag -p victory --features diagnostics
scripts\play.ps1 -Exe <path of your copy's executable>
```

`scripts\play.ps1 -Safe` leaves out the newest, faster ways; try it first when
something is wrong. `-RayTracing` turns the title's ray-traced indirect light on.

Before the first run, ahead of time (both can be stopped and go on later):

```powershell
target\diag\victory.exe shaders <the shader archives>   # every shader translated, minutes
target\diag\victory.exe warm --used                     # pipelines earlier runs drew with, for a new driver or build
```

Commands, debugging switches and upstream's own description:
[docs/README-fxceless.md](docs/README-fxceless.md). Every `VICTORY_...` switch is
described in a comment where it is read.

## What is different here

In short; the reasons and the measurements are in [docs/findings.md](docs/findings.md).

- **List calls are written down, not carried out, on the title's threads**
  (`VICTORY_THIN`): eight threads of ours translate them.
- **Submissions are prepared by a thread of ours and joined**
  (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`), with what can be spread over
  threads spread: refreshes, compares (AVX2), descriptor records, landings.
- **What shaders wrote comes back beside the write guards**, by the dwords that
  changed, on a thread of its own (`VICTORY_RELAY`, `VICTORY_LAND`).
- **Buffer fills that need bytes still on the card are done on the card**
  (`VICTORY_CARD_STEPS`) instead of waiting for it.
- **The title's writes are caught more cheaply**: one view opened at a fault,
  blocks that rest, places in the title's code that look before they write.
- **Textures go up on a copy queue of their own** and a slot shows one only after
  its upload ran, which also ended a device loss that came every 15 minutes or so.
- **Boats float**: textures that shaders write and the title's CPU reads go back
  to title memory (from fxceless and Geckomacabre, on by default here).
- **Frame record and replay** with per-pass times and shader variants compared
  turn by turn ([docs/replay.md](docs/replay.md)).

## Credits

This is a tree of other people's work with more on top. The history is kept, and
commits taken from elsewhere name the commit and its author.

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  this one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.
- This fork's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not
covered is named in [docs/asset-policy.md](docs/asset-policy.md).
