//! Decision prototype for the FerrumOS heap (objective 1).
//!
//! Runs every scenario of the protocol against every allocator, each time on
//! a fresh 100 KiB heap, and prints the results side by side.
//!
//!   cargo run --release -- [--output results.json]
//!
//! Protocol: docs/objectif1/06_prototype_decision.md

// The scenarios are line-by-line ports of the kernel experiments: keep their
// arithmetic as written there rather than following newer clippy idioms.
#![allow(clippy::manual_is_multiple_of)]

mod heaps;
mod scenarios;
mod timing;

use heaps::{make, Kind};
use scenarios::Outcome;
use std::fmt::Write as _;

const EXP1_SEEDS: [u64; 6] = [0x2026_1017, 1, 2, 3, 0xDEAD_BEEF, 0x1234_5678];
const LONG_RUN_SEED: u64 = 0x2026_1112;

struct Row {
    kind: Kind,
    exp1: Outcome,
    exp1_seeds_errors: u64,
    exp1_seeds_exhaustions: Vec<u64>,
    exp2: Outcome,
    exp3: Outcome,
    phase: Outcome,
    long_run: Outcome,
    exp1_latency: timing::Latency,
    long_run_latency: timing::Latency,
    phase_change_first_large_ns: u64,
}

fn run(kind: Kind) -> Row {
    let fresh = || make(kind);

    let (_r, mut h) = fresh();
    let exp1 = scenarios::exp1(h.as_mut(), EXP1_SEEDS[0]);

    let mut exp1_seeds_errors = 0;
    let mut exp1_seeds_exhaustions = Vec::new();
    for seed in EXP1_SEEDS {
        let (_r, mut h) = fresh();
        let o = scenarios::exp1(h.as_mut(), seed);
        exp1_seeds_errors += o.errors;
        exp1_seeds_exhaustions.push(o.exhaustions);
    }

    let (_r, mut h) = fresh();
    let exp2 = scenarios::exp2(h.as_mut());
    let (_r, mut h) = fresh();
    let exp3 = scenarios::exp3(h.as_mut());
    let (_r, mut h) = fresh();
    let phase = scenarios::phase_change(h.as_mut());
    let (_r, mut h) = fresh();
    let long_run = scenarios::long_run(h.as_mut(), LONG_RUN_SEED);

    Row {
        kind,
        exp1,
        exp1_seeds_errors,
        exp1_seeds_exhaustions,
        exp2,
        exp3,
        phase,
        long_run,
        exp1_latency: timing::exp1_latency(kind),
        long_run_latency: timing::long_run_latency(kind),
        phase_change_first_large_ns: timing::phase_change_first_large_ns(kind),
    }
}

fn opt<T: std::fmt::Display>(v: Option<T>) -> String {
    v.map_or("n/a".to_string(), |v| v.to_string())
}

fn markdown(rows: &[Row]) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "### Correction et réutilisation (grandeurs déterministes)\n"
    );
    let _ = writeln!(
        s,
        "| Allocateur | exp1 erreurs | exp1 épuisements | exp1 × 6 graines : erreurs / épuisements | exp2 itérations | exp2 `used` | exp3 capacité (3 tours) | changement de profil : 64 o puis 1 Kio | long_run : refus / 100 000 | long_run : fragmentation max | erreurs (toutes) |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|---|---|---|---|");
    for r in rows {
        let all_errors = r.exp1.errors
            + r.exp1_seeds_errors
            + r.exp2.errors
            + r.exp3.errors
            + r.phase.errors
            + r.long_run.errors;
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} / {:?} | {} | {} | {:?} | {} puis {} | {} | {} | {} |",
            r.kind.name(),
            r.exp1.errors,
            r.exp1.exhaustions,
            r.exp1_seeds_errors,
            r.exp1_seeds_exhaustions,
            r.exp2.iterations,
            opt(r.exp2.used_after),
            r.exp3.capacities,
            r.phase.capacities[0],
            r.phase.capacities[1],
            r.long_run.exhaustions,
            opt(r
                .long_run
                .fragmentation_permille
                .map(|p| format!("{} ‰", p))),
            all_errors,
        );
    }

    let _ = writeln!(
        s,
        "\n### Coût par opération sur l'hôte (ns, médiane de {} exécutions)\n",
        timing::RUNS
    );
    let _ = writeln!(
        s,
        "| Allocateur | exp1 p50 | exp1 p99 | exp1 max | exp1 moyenne | long_run p50 | long_run p99 | long_run max | long_run moyenne | changement de profil : 1re alloc 1 Kio |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|---|---|---|");
    for r in rows {
        let a = r.exp1_latency;
        let b = r.long_run_latency;
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            r.kind.name(),
            a.p50_ns,
            a.p99_ns,
            a.max_ns,
            a.mean_ns,
            b.p50_ns,
            b.p99_ns,
            b.max_ns,
            b.mean_ns,
            if r.phase_change_first_large_ns == 0 {
                "refusée".to_string()
            } else {
                r.phase_change_first_large_ns.to_string()
            }
        );
    }
    s
}

fn json(rows: &[Row]) -> String {
    let mut s = String::from("{\n  \"heap_size\": 102400,\n  \"timing_runs\": ");
    let _ = write!(s, "{},\n  \"allocators\": [\n", timing::RUNS);
    for (i, r) in rows.iter().enumerate() {
        let lat = |l: timing::Latency| {
            format!(
                "{{\"ops\": {}, \"p50_ns\": {}, \"p99_ns\": {}, \"max_ns\": {}, \"mean_ns\": {}}}",
                l.ops, l.p50_ns, l.p99_ns, l.max_ns, l.mean_ns
            )
        };
        let num = |v: Option<usize>| v.map_or("null".to_string(), |v| v.to_string());
        let _ = write!(
            s,
            "    {{\"id\": \"{}\", \"name\": \"{}\",\n     \"exp1\": {{\"errors\": {}, \"exhaustions\": {}, \"used_after\": {}}},\n     \"exp1_seeds\": {{\"seeds\": {}, \"errors\": {}, \"exhaustions\": {:?}}},\n     \"exp2\": {{\"iterations\": {}, \"errors\": {}, \"used_after\": {}, \"live_bytes\": {}}},\n     \"exp3\": {{\"capacities\": {:?}, \"errors\": {}, \"used_after\": {}}},\n     \"phase_change\": {{\"small_capacity\": {}, \"large_capacity\": {}, \"errors\": {}}},\n     \"long_run\": {{\"refused\": {}, \"errors\": {}, \"worst_fragmentation_permille\": {}}},\n     \"latency_exp1\": {},\n     \"latency_long_run\": {},\n     \"phase_change_first_large_ns\": {}}}{}\n",
            r.kind.id(),
            r.kind.name(),
            r.exp1.errors,
            r.exp1.exhaustions,
            num(r.exp1.used_after),
            EXP1_SEEDS.len(),
            r.exp1_seeds_errors,
            r.exp1_seeds_exhaustions,
            r.exp2.iterations,
            r.exp2.errors,
            num(r.exp2.used_after),
            num(r.exp2.live_bytes_after),
            r.exp3.capacities,
            r.exp3.errors,
            num(r.exp3.used_after),
            r.phase.capacities[0],
            r.phase.capacities[1],
            r.phase.errors,
            r.long_run.exhaustions,
            r.long_run.errors,
            r.long_run
                .fragmentation_permille
                .map_or("null".to_string(), |v| v.to_string()),
            lat(r.exp1_latency),
            lat(r.long_run_latency),
            r.phase_change_first_large_ns,
            if i + 1 < rows.len() { "," } else { "" }
        );
    }
    s.push_str("  ]\n}\n");
    s
}

fn main() {
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = args.next(),
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    let rows: Vec<Row> = Kind::ALL.iter().map(|&k| run(k)).collect();
    print!("{}", markdown(&rows));
    if let Some(path) = output {
        std::fs::write(&path, json(&rows)).expect("cannot write the results");
        eprintln!("results written to {path}");
    }
}
