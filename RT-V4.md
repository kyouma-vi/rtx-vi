# V4 — DXR groundwork / experimental translated RT

This is a source patch, not native DXR lighting. The existing original-lighting
path still uses translated GFX10 compute traversal; no RT-core dispatch is added.
V3 lighting defaults, sun fix, builder bounds, barriers and shadow filters stay.
The game binary is not included or changed.

## Build and compare

```powershell
cargo build --release --locked -p victory
$env:VICTORY_GAME_DIR = 'C:\GTA6_Xbox'
.\Play-RT-V4.cmd
```

The V4 wrapper enables `VICTORY_BVH_ADDR32=1` and logs DXR capability.
To compare with the old address math, set `$env:VICTORY_BVH_ADDR32 = '0'`
and use the same wrapper. Both variants use the title's own lighting; no rays,
resolution or shading samples are removed. Let shaders compile and use the same
scene and settings for both tests. The switch is in the shader cache key.

Log capture (Windows PowerShell):

```powershell
.\Play-RT-V4.cmd 2>&1 | Tee-Object -FilePath rt-v4.log
```

Look for `host DXR capability (not active)` (tier 11 = DXR 1.1), host wave lanes,
frame statistics and `title memory by address`. For detailed GPU timings, the
existing diagnostics build (`cargo build --locked --profile diag -p victory
--features diagnostics`) supports `VICTORY_GPU_TIMING=1` and `VICTORY_D3D12_DRED=1`.
Run it with the same environment/profile; timing serializes work and is not an
FPS benchmark. Do not replay the known hanging captures described in the fork docs.

## Changes

- Optional 32-bit split-word address calculation for BVH node loads, preserving
  wrapping arithmetic and all 64 address bits. The original path is the default
  outside the V4 launcher. Actual NVIDIA instruction count and speedup unmeasured.
- A D3D12 OPTIONS5 capability query on the active host device. No change to its
  advertised title features and no assumption that DXR is in use.
- `victory_shader::dxr_geometry::extract_blas`: bounded offline extraction of
  reachable triangles from a stable, rehydrated BLAS snapshot, with original
  leaf IDs and barycentric vertex order. Rejects malformed IDs, nonfinite vertices,
  repeated pointers, unavailable nodes, unsupported kinds and budget overruns.
  It is not called by live rendering, and assumes the caller supplies bounded,
  correctly identified BLAS memory. Packed structures and TLAS are not supported.

## Remaining work for native DXR

1. Capture correct BLAS boundaries, root pointers, instance transforms, original
   primitive/material identity, flags and update lifetimes. Resolve overlapping
   dynamic allocations before trusting snapshots. See `docs/fork-ray-tracing.md`.
2. Decode GPU-resident data without per-frame CPU readbacks; build native BLAS/TLAS
   with resource lifetime, synchronization, refit and compaction tracking.
3. Replace complete traversal loops, not the per-node intersection instruction,
   with inline DXR RayQuery where semantics can be matched. Preserve alpha tests,
   instance masks, hit distance, barycentrics, primitive IDs and shading.
4. Compare hit results and lighting against translated tracing, then profile
   on real hardware before enabling any native path by default.

Validation here: 100,160 edge/random address arithmetic comparisons in a Python
model, archive integrity. Rust compiler and Windows D3D12 GPU unavailable;
Rust unit tests and DXIL GPU tests have not run. On Windows run the GPU tests
with `$env:VICTORY_BVH_ADDR32='1'` and `cargo test --locked -p victory-shader
intersect_tests`, and `cargo test --locked -p victory-shader dxr_geometry`.
No FPS target is verified; this first step is not expected by itself to yield 15x.
