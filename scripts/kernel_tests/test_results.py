#!/usr/bin/env python3
"""Self-tests for results.py. Run: python3 -m unittest discover -s scripts/kernel_tests"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import results  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
BASELINE = ROOT / "POC/ferrum_Poc_adaptative_sheduller/tests/baseline.json"

# Shape of a real `cargo test --no-fail-fast` run with the bootimage runner.
PASSING_LOG = """\
   Compiling ferrumos v0.1.0 (/repo/POC/ferrum_Poc_adaptative_sheduller)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.12s
     Running unittests src/lib.rs (target/x86_64-ferrumos/debug/deps/ferrumos-1a2b3c)
Building bootloader
    Finished `release` profile [optimized + debuginfo] target(s) in 0.71s
Running: `qemu-system-x86_64 -drive format=raw,file=target/x86_64-ferrumos/debug/deps/bootimage-ferrumos-1a2b3c.bin -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04 -serial stdio -display none`
Running 3 tests
ferrumos::tests::trivial_assertion...\t[ok]
ferrumos::tests::cycle_counter_advances_and_measures...\t8596320 cycles for 10 000 additions\t[ok]
ferrumos::tests::record_format_is_printable...\t
\tRECORD name=box_alloc_64B iterations=1 errors=0 used_before=32 used_after=96 cycles=409928
[ok]
     Running tests/experiment_1.rs (target/x86_64-ferrumos/debug/deps/experiment_1-4d5e6f)
Running: `qemu-system-x86_64 -drive format=raw,file=bootimage-experiment_1.bin`
Running experiment 1 on a freshly booted kernel
\theap exhaustions forcing a full release: 13
\tRECORD name=exp1_alloc_free_sequence iterations=10000 errors=0 used_before=0 used_after=0 cycles=1348639884
     Running tests/experiment_2.rs (target/x86_64-ferrumos/debug/deps/experiment_2-7a8b9c)
Running: `qemu-system-x86_64 -drive format=raw,file=bootimage-experiment_2.bin`
Running experiment 2 on a freshly booted kernel
\tchurn of 1024 B survived 99 iterations out of 10000 (102400 B heap, 4 long lived blocks)
\tRECORD name=exp2_long_lived_vs_churn iterations=99 errors=0 used_before=0 used_after=101632 cycles=1971768
"""

FAILING_LOG = PASSING_LOG.replace(
    "\tRECORD name=exp2_long_lived_vs_churn iterations=99 errors=0 used_before=0 used_after=101632 cycles=1971768\n",
    "\tRECORD name=exp2_long_lived_vs_churn iterations=50 errors=2 used_before=0 used_after=101632 cycles=1971768\n"
    "[failed]\n"
    "panicked at src/experiments.rs:276:5:\n"
    "assertion `left == right` failed\n"
    "error: test failed, to rerun pass `--test experiment_2`\n",
) + "error: 1 target failed:\n    `--test experiment_2`\n"


class ParseLogTest(unittest.TestCase):
    def test_passing_run(self):
        suites, records = results.parse_log(PASSING_LOG)
        self.assertEqual([s["id"] for s in suites], ["lib", "experiment_1", "experiment_2"])
        self.assertTrue(all(s["status"] == "passed" for s in suites))
        lib = suites[0]
        self.assertEqual(lib["declared_tests"], 3)
        self.assertEqual([t["status"] for t in lib["tests"]], ["passed"] * 3)
        self.assertEqual([r["name"] for r in records],
                         ["box_alloc_64B", "exp1_alloc_free_sequence", "exp2_long_lived_vs_churn"])
        self.assertEqual(records[2]["iterations"], 99)
        self.assertEqual(records[2]["suite"], "experiment_2")

    def test_failing_experiment(self):
        suites, records = results.parse_log(FAILING_LOG)
        by_id = {s["id"]: s for s in suites}
        self.assertEqual(by_id["experiment_2"]["status"], "failed")
        self.assertEqual(by_id["experiment_1"]["status"], "passed")
        self.assertTrue(by_id["experiment_2"]["panic"].startswith("panicked at src/experiments.rs"))

    def test_failing_unit_test(self):
        log = PASSING_LOG.replace(
            "ferrumos::tests::trivial_assertion...\t[ok]",
            "ferrumos::tests::trivial_assertion...\t[failed]\npanicked at src/lib.rs:90:9:",
        ) + "error: test failed, to rerun pass `--lib`\n"
        suites, _ = results.parse_log(log)
        self.assertEqual(suites[0]["status"], "failed")
        self.assertEqual(suites[0]["tests"][0]["status"], "failed")

    def test_hung_test_is_unfinished(self):
        log = PASSING_LOG.replace("ferrumos::tests::trivial_assertion...\t[ok]",
                                  "ferrumos::tests::trivial_assertion...\t")
        suites, _ = results.parse_log(log)
        self.assertEqual(suites[0]["tests"][0]["status"], "unfinished")
        self.assertEqual(suites[0]["status"], "failed")

    def test_crlf(self):
        suites, records = results.parse_log(PASSING_LOG.replace("\n", "\r\n"))
        self.assertEqual(len(suites), 3)
        self.assertEqual(len(records), 3)

    def test_committed_measurement_logs(self):
        # Rayan's logs (docs/memoire/logs) only kept the kernel lines; every
        # RECORD must still be read with all its fields.
        for log in sorted((ROOT / "docs/memoire/logs").glob("*.log")):
            text = log.read_text(encoding="utf-8")
            wrapped = "     Running tests/all.rs (x)\n" + text
            _, records = results.parse_log(wrapped)
            self.assertEqual(len(records), text.count("RECORD name="), log.name)
            for r in records:
                self.assertEqual(
                    set(r) - {"name", "suite"},
                    {"iterations", "errors", "used_before", "used_after", "cycles"},
                    log.name,
                )


class CompareTest(unittest.TestCase):
    def setUp(self):
        self.baseline = json.loads(BASELINE.read_text(encoding="utf-8"))

    def run_collect(self, log, exit_code):
        with tempfile.TemporaryDirectory() as tmp:
            log_path = Path(tmp, "log")
            log_path.write_text(log, encoding="utf-8")
            out = Path(tmp, "results.json")
            results.main(["collect", "--log", str(log_path), "--exit-code", str(exit_code),
                          "--output", str(out), "--commit", "abc1234", "--branch", "test"])
            return json.loads(out.read_text(encoding="utf-8"))

    def test_baseline_matches_reference_measurements(self):
        # Every committed reference run must pass the baseline.
        for log in sorted((ROOT / "docs/memoire/logs").glob("*.log")):
            _, records = results.parse_log("     Running tests/all.rs (x)\n" + log.read_text(encoding="utf-8"))
            run = {"status": "passed", "suites": [], "records": records}
            baseline = dict(self.baseline, expected_suites=[])
            for rec in records:
                rules = baseline["records"].get(rec["name"], {})
                for field, rule in rules.items():
                    self.assertIsNone(results.check_rule(rec[field], rule), f"{log.name} {rec['name']}.{field}")
            problems, _ = results.compare(run, baseline)
            # the filtered logs do not contain every record, only check rules on those present
            self.assertFalse([p for p in problems if "no RECORD line" not in p], log.name)

    def test_regression_is_detected(self):
        run = self.run_collect(FAILING_LOG, 101)
        self.assertEqual(run["status"], "failed")
        problems, report = results.compare(run, self.baseline)
        joined = "\n".join(problems)
        self.assertIn("exp2_long_lived_vs_churn.iterations = 50", joined)
        self.assertIn("exp2_long_lived_vs_churn.errors = 2", joined)
        self.assertIn("test image `experiment_2` failed", joined)
        self.assertIn("test image `experiment_3` did not run", joined)

    def test_improvement_passes_and_shows_delta(self):
        previous = self.run_collect(PASSING_LOG, 0)
        better = PASSING_LOG.replace("iterations=99 errors=0", "iterations=10000 errors=0")
        current = self.run_collect(better, 0)
        baseline = dict(self.baseline, expected_suites=["lib", "experiment_1", "experiment_2"])
        problems, report = results.compare(current, baseline, previous)
        self.assertFalse([p for p in problems if "exp2" in p])
        self.assertIn("+9901", "\n".join(report))

    def test_build_error(self):
        run = self.run_collect("error[E0425]: cannot find value `x` in this scope\n", 101)
        self.assertEqual(run["status"], "error")
        problems, _ = results.compare(run, self.baseline)
        self.assertTrue(any("before running the tests" in p for p in problems))

    def test_cycles_never_fail(self):
        slower = PASSING_LOG.replace("cycles=1348639884", "cycles=9999999999")
        run = self.run_collect(slower, 0)
        baseline = dict(self.baseline, expected_suites=[])
        problems, _ = results.compare(run, baseline)
        self.assertFalse([p for p in problems if "cycles" in p])


if __name__ == "__main__":
    unittest.main()
