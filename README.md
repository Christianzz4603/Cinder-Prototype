# Native Stress Benchmark

A reproducible Android ARM64 benchmark for ELF inspection, disassembly, function discovery, and decompiler throughput.

## Workflows

- Build Native Stress Library manually generates C++ source shards, commits them under generated-src/, builds an ARM64 Android shared library, and uploads an artifact. If the library is under 90 MiB it is also committed; larger libraries are attached to a GitHub Release because ordinary Git files have a 100 MB limit.
- Decompile Native Stress Library downloads a selected build artifact, downloads the official Ghidra distribution, runs headless analysis, exports a bounded number of C-like pseudocode files, and commits reports to the decompiled-results branch.

## Running

1. Open Actions and run Build Native Stress Library. Defaults use 12,000 functions and 96 rounds per function, so the build may take a while.
2. Copy the successful build run ID from its Actions URL.
3. Run Decompile Native Stress Library with that run ID. Start with 100-500 functions.
4. Download the native-stress-pseudocode artifact or inspect the decompiled-results branch.

Compiled size depends on compiler output and is not guaranteed to exceed a particular threshold. The workflow records actual size and SHA-256. Decompilation is capped at 2,000 functions per run to control time and storage.
