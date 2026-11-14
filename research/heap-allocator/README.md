# Heap allocator research (objective 1)

Decision prototype and exploration prototype for the FerrumOS kernel heap.
Documents and results: [`docs/greenlights/objecitf_obligatoire_1/tas_noyau/`](../../docs/greenlights/objecitf_obligatoire_1/tas_noyau/README.md).

| Path | Content |
|---|---|
| `bench/` | host benchmark: same scenarios (ports of kernel experiments 1 to 3, plus `phase_change` and `long_run`) against every allocator, on a fresh 100 KiB heap each time |
| `../../libs/ferrum_alloc/` | the allocators themselves (bump, linked list, fixed-size blocks), the same code the kernel uses |

```sh
cd bench
cargo run --release -- --output bench.json   # stable Rust, about 0.5 s
```

The deterministic columns (iterations, errors, capacities, refusals) must be
identical from run to run and must match the QEMU runs; the nanosecond
columns depend on the machine. The TLSF row uses the `rlsf` crate and is the
exploration prototype only: the kernel does not depend on it.
