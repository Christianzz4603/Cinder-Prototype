# Native Stress Benchmark

A reproducible Android ARM64 benchmark for ELF inspection, disassembly, function discovery, and decompiler throughput.

- Build Native Stress Library generates and commits C++ source shards, builds an Android ARM64 shared library, and uploads an artifact. A 96 MiB generated read-only data section ensures the file is large enough for large-binary testing; this increases file size, not the complexity of each function.
- Decompile Native Stress Library downloads a selected build artifact, downloads the official Ghidra distribution, runs headless analysis, exports a bounded number of C-like pseudocode files, and commits reports to the decompiled-results branch.

## Run

1. Open Actions and run Build Native Stress Library. Defaults use 12,000 functions and 96 rounds per function.
2. Copy the successful build run ID from its Actions URL.
3. Run Decompile Native Stress Library with that run ID. Start with 100-500 functions.
4. Download the native-stress-pseudocode artifact or inspect the decompiled-results branch.

The workflow records the actual binary size and SHA-256. Large binaries are attached to a GitHub Release rather than committed as ordinary Git blobs, which have a 100 MB per-file limit. Decompilation is capped at 2,000 functions per run.
