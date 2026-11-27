# Experimental 60 FPS target patch

## Implementation

Static analysis of the supplied executable identified two paths that load a
configured FPS value, clamp it, convert it into a microsecond interval and
store that interval in the same runtime variable. That variable is used by the
engine's refresh-divisor selection and time-based pacing. The patch changes
only the two FPS loads to return 60, overriding the configured target in
those paths (including values other than 30). It leaves the engine's calculations and
host frame clock intact. The console display mode reports 60 Hz. No system
clock, physics constants, shared float constants, graphics settings, or
previous NVIDIA fixes are changed.

The full original build hash and both instruction signatures are checked
before either site is changed. The prepared image has a separate `-60fps.dll`
cache name; disabling the flag restores the original prepared image. No game
binary or extracted game data is included in this repository.

## Run and verify

```powershell
cargo build --release --locked -p victory
$env:VICTORY_GAME_DIR = "C:\GTA6_Xbox"
$env:VICTORY_LOG = "info"
.\Play-VIctory.cmd pareto-residency -60fps
```

Look for `60 FPS patch   applied (2 sites; experimental; target 60 FPS)` and
`frame interval` near 16.667 ms in the log. Compare the window's measured FPS
and frametime against a run without the flag, using the same scene and settings.
Test menus, character movement, physics, vehicle motion, animation, cutscenes
and audio/video sync. Check whether 60 seconds of game time still behave like
60 seconds in the normal mode (where a real-time timer is available). Stop and
remove the flag if gameplay speed or stability changes unexpectedly.

The original-lighting path is still expensive; the flag does not speed up GPU
work. A frame must finish in about 16.7 ms to sustain 60 real FPS. A scene that
currently renders at 2 FPS will not reach 60 just by removing a limiter.

## Tests

```powershell
cargo test --locked -p victory-loader fps60
cargo test --locked -p victory-loader sixty_fps_refuses_an_unrelated_build
cargo test --locked -p victory fps_cli_tests
$env:VICTORY_TEST_60FPS_EXE = "$env:VICTORY_GAME_DIR\game_scarlett_master.exe"
cargo test --locked -p victory-loader supplied_build_optional
Remove-Item Env:VICTORY_TEST_60FPS_EXE
```

The optional test reads your own executable, checks the build and both patch
sites, compares the prepared bytes and checks the distinct cache name. The
archive was statically checked against the supplied executable, but the Rust
toolchain and Windows GPU runtime were not available in the editing environment.
No Rust build, Windows launch, or gameplay timing validation has been performed.
