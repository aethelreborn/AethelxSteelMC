# ============================================================================
# AethelxStoneMC
# A reusable SteelMC Minecraft server template
# https://steelmc.dev
# ============================================================================

## Quick Start
#
#   ./start 4G      — Start the server with 4G RAM hint
#   ./stop          — Gracefully stop the server
#   ./log           — Follow the latest log file
#
# SteelMC is a native Rust binary — no Java/JVM required!

## Playit
#
# Set up the public tunnel for Minecraft Java Edition:
#
#   cp playit.conf.example playit.conf
#   ./setup-playit
#
# The setup script downloads the official Playit Linux agent, stores its
# runtime files under .playit/, and prints a one-time browser claim link.
# Approve that link, then create or select a TCP tunnel to local port 25565
# in the Playit dashboard. The generated public address is the address
# players use to connect.
#
# Keep playit.conf and .playit/ private. They are ignored by Git and the
# agent secret must never be committed. To remove local Playit credentials:
#
#   rm -f .playit/state/playit.toml
#
# Run ./setup-playit again to claim a new agent.

## What is SteelMC?
#
# SteelMC is a modern, high-performance Minecraft Java Edition server written
# in Rust. It targets block-for-block terrain parity with vanilla Minecraft
# 26.2 and features:
#   • Multithreaded chunk generation (chunk pyramid scheduler)
#   • Parallel lighting engine
#   • Persistent multiplayer worlds
#   • Inventory, commands, and redstone support
#   • Targets latest Minecraft Java Edition releases
#
# GitHub: https://github.com/Steel-Foundation/SteelMC
# Docs:   https://steelmc.dev
# Discord: https://discord.gg/suSXXNdVSf

## Files
#
#   steel-linux      — The server binary
#   start            — Launch script (./start [RAM])
#   stop             — Graceful stop script
#   log              — Tail the latest log
#   setup-playit     — Install and claim the Playit tunnel agent
#   playit.conf.example — Non-secret Playit defaults
#   config/          — Server configuration (TOML files)
#     ├ config.toml   — Main server config (port, motd, threads, etc.)
#     ├ groups.toml   — Permission groups
#     ├ worlds.toml   — World definitions (overworld, nether, end)
#     └ favicon.png   — Server icon
#   saves/           — Persistent world data
#   .logs/           — Server log files (auto-rotated daily)

## Configuration
#
# Edit config/config.toml to change:
#   server_port         — Default: 25565
#   max_players         — Default: 20
#   motd                — Server message of the day
#   online_mode         — Enable Mojang auth (default: true)
#   view_distance       — Default: 10 chunks
#   simulation_distance — Default: 10 chunks
#
# See: https://steelmc.dev/configuration/server-configuration/

## Notes
#
# • This is a pre-alpha server — expect bugs and incomplete features.
# • Plugins do not exist yet; Paper/Bukkit/Fabric/Forge plugins will NOT work.
# • No Java runtime is required (native Rust binary).
# • The RAM argument to ./start is informational — SteelMC manages its own memory.
