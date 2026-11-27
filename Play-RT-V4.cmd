@echo off
setlocal EnableExtensions DisableDelayedExpansion
if not defined VICTORY_BVH_ADDR32 set "VICTORY_BVH_ADDR32=1"
if not defined VICTORY_LOG set "VICTORY_LOG=info"
call "%~dp0Play-OriginalLighting.cmd"
exit /b %errorlevel%
