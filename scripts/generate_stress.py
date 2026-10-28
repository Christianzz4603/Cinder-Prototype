#!/usr/bin/env python3
import argparse
from pathlib import Path

MASK = (1 << 64) - 1
SHARD = 250

def u64(v): return v & MASK

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--functions", type=int, default=12000)
    p.add_argument("--rounds", type=int, default=96)
    p.add_argument("--output-dir", default="generated-src")
    a = p.parse_args()
    if not 100 <= a.functions <= 12000: p.error("--functions must be 100..12000")
    if not 8 <= a.rounds <= 96: p.error("--rounds must be 8..96")
    out = Path(a.output_dir)
    out.mkdir(parents=True, exist_ok=True)
    for old in out.glob("stress_*.cpp"): old.unlink()
    shards = (a.functions + SHARD - 1) // SHARD
    for shard in range(shards):
        lo, hi = shard * SHARD, min(a.functions, (shard + 1) * SHARD)
        with (out / f"stress_{shard:04d}.cpp").open("w") as f:
            f.write("""#include <cstdint>
static inline std::uint64_t mix(std::uint64_t x) {
 x ^= x >> 30; x *= 0xBF58476D1CE4E5B9ULL;
 x ^= x >> 27; x *= 0x94D049BB133111EBULL;
 return x ^ (x >> 31);
}
""")
            for n in range(lo, hi):
                seed = u64((n + 1) * 0x9E3779B97F4A7C15)
                f.write(f'\nextern "C" __attribute__((visibility("hidden"),noinline,used)) std::uint64_t stress_{n}(std::uint64_t x,std::uint64_t k) {{\n x ^= 0x{seed:016X}ULL;\n')
                for r in range(a.rounds):
                    c = u64((n + 1) * (r + 3) * 0xD6E8FEB86659FD93)
                    shift = (n * 7 + r * 11) % 63 + 1
                    f.write(f' x += k ^ 0x{c:016X}ULL;\n')
                    f.write(f' x = (x << {shift}) | (x >> {64-shift});\n')
                    f.write(f' x ^= mix(x + 0x{u64(c ^ 0xA5A5A5A5A5A5A5A5):016X}ULL);\n')
                    f.write(f' x *= 0x{u64(c | 1):016X}ULL;\n')
                    f.write(f' if (((x >> {(r % 31)+1}) & 3ULL) == {(r % 4)}ULL) x ^= k + 0x{u64(c+r):016X}ULL;\n')
                f.write(' return mix(x ^ k);\n}\n')
    with (out / "dispatch.cpp").open("w") as f:
        f.write("""#include <cstdint>
#include <cstddef>
using u64=std::uint64_t;
static u64 mix_dispatch(u64 x){x^=x>>30;x*=0xBF58476D1CE4E5B9ULL;x^=x>>27;x*=0x94D049BB133111EBULL;return x^(x>>31);}
""")
        for n in range(a.functions): f.write(f'extern "C" u64 stress_{n}(u64,u64);\n')
        f.write('extern "C" __attribute__((visibility("default"),noinline)) u64 native_stress_big(const u64* d,std::size_t n,u64 k){if(!d||!n||n>4096)return 0;u64 s=mix_dispatch(k^n);for(std::size_t i=0;i<n;i++){switch((d[i]^s^k)%'+str(a.functions)+'ULL){\n')
        for n in range(a.functions): f.write(f'case {n}: s^=stress_{n}(d[i]+s,k^i);break;\n')
        f.write('}}return mix_dispatch(s);}\n')
    print("functions", a.functions, "rounds", a.rounds, "source_bytes", sum(x.stat().st_size for x in out.glob("*.cpp")))

if __name__ == "__main__": main()
