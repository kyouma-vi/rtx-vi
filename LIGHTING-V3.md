# V3 lighting profiles

Original lighting: Play-OriginalLighting.cmd requests the title's own translated ray tracing, with the existing all-or-none builder safety check and ambient fallback. This is not native DXR. Known limitations and GPU cost are described in docs/fork-ray-tracing.md. The game is not modified.

Balanced: Play-VIctory.cmd keeps ray tracing off by default and raises the existing scene-adaptive indirect-light estimate from 0.8 to 1.0, with crease shade strength 0.65 instead of 1.0. These are uncalibrated tuning values, not a verified match to the original. Existing user environment overrides still apply. Play-DLSS.ps1 -RayTracing requests the original lighting path as before.

No shader changes relative to V2. Build and GPU visual tests were not performed in this environment. For comparison, use the same scene and reset inherited VICTORY_AMBIENT / VICTORY_BVH_BUILDER variables before the balanced launch.
