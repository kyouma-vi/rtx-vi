@echo off
setlocal EnableExtensions
set "VICTORY_BVH_BUILDER=1"
set "VICTORY_BVH_QUARANTINE=0"
set "VICTORY_AMBIENT=1.0"
set "VICTORY_AMBIENT_SHADE=0.65,0.6"
if not defined VICTORY_LOG set "VICTORY_LOG=info"
call "%~dp0Play-VIctory.cmd" pareto-residency %*
exit /b %errorlevel%
