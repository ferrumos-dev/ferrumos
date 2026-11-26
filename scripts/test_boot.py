"""Failure-path checks for the boot/image orchestration (no QEMU required)."""

from pathlib import Path
import socket
import subprocess
import tempfile
import time
import unittest
from unittest.mock import Mock, patch

import boot


class BootChecks(unittest.TestCase):
    def test_new_test_attempt_removes_previous_success_and_screenshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            (directory / "summary.json").write_text('{"status": "passed"}', encoding="utf-8")
            (directory / "screen.ppm").write_bytes(b"old screenshot")
            with patch.object(boot, "RESULTS", directory):
                boot.prepare_test_results()
            self.assertFalse((directory / "screen.ppm").exists())
            self.assertEqual(boot.json.loads((directory / "summary.json").read_text())["status"], "not-run")

    def test_failed_kernel_build_invalidates_previous_image(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            image = directory / "ferrumos-bios.img"
            image.write_bytes(b"old image")
            partial = image.with_suffix(".img.partial")
            partial.write_bytes(b"old partial")
            with patch.object(boot, "BOOT", directory), patch.object(boot, "IMAGE", image), \
                    patch.object(boot.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "cargo")):
                with self.assertRaises(boot.BootError):
                    boot.build_image()
            self.assertFalse(image.exists())
            self.assertFalse(partial.exists())

    def test_successful_builder_without_image_is_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            image = directory / "ferrumos-bios.img"
            with patch.object(boot, "BOOT", directory), patch.object(boot, "IMAGE", image), \
                    patch.object(boot.subprocess, "run", return_value=subprocess.CompletedProcess([], 0)):
                with self.assertRaisesRegex(boot.BootError, "without a nonempty image"):
                    boot.build_image()
            self.assertFalse(image.exists())

    def test_failing_image_builder_never_publishes_partial_image(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            image = directory / "ferrumos-bios.img"

            def execute(command, **_kwargs):
                if "run" in command:
                    Path(command[-1]).write_bytes(b"incomplete image")
                    raise subprocess.CalledProcessError(1, command)
                return subprocess.CompletedProcess(command, 0)

            with patch.object(boot, "BOOT", directory), patch.object(boot, "IMAGE", image), \
                    patch.object(boot.subprocess, "run", side_effect=execute):
                with self.assertRaises(boot.BootError):
                    boot.build_image()
            self.assertFalse(image.exists())
            self.assertFalse(image.with_suffix(".img.partial").exists())

    def test_early_qemu_exit_is_failure_even_with_markers(self):
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / "serial.log"
            log.write_text("\n".join(boot.MARKERS) + "\n", encoding="utf-8")
            process = Mock()
            process.poll.return_value = 0
            with self.assertRaisesRegex(boot.BootError, "exited before"):
                boot.wait_for_serial(process, log, boot.MARKERS, time.monotonic() + 1)

    def test_markers_in_panic_messages_do_not_count_as_success(self):
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / "serial.log"
            log.write_text("PANIC FERRUMOS_BOOT_OK\nDISPLAY_READY\nKEYBOARD_READY\n", encoding="utf-8")
            process = Mock()
            process.poll.return_value = None
            with self.assertRaisesRegex(boot.BootError, "panic"):
                boot.wait_for_serial(process, log, boot.MARKERS, time.monotonic() + 1)

    def test_missing_ack_times_out(self):
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / "serial.log"
            log.write_text("INPUT_LINE: tesxt\n", encoding="utf-8")
            process = Mock()
            process.poll.return_value = None
            with self.assertRaisesRegex(boot.BootError, "timed out"):
                boot.wait_for_serial(process, log, (boot.ACK,), time.monotonic() + 0.01)

    def test_driver_boot_markers_do_not_validate_kernel_tasks(self):
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / "serial.log"
            log.write_text("FERRUMOS_BOOT_OK\nDISPLAY_READY\nKEYBOARD_READY\n", encoding="utf-8")
            process = Mock()
            process.poll.return_value = None
            with self.assertRaisesRegex(boot.BootError, "timed out"):
                boot.wait_for_serial(process, log, boot.MARKERS, time.monotonic() + 0.01)

    def test_memory_summary_rejects_absent_inconsistent_or_impossible_totals(self):
        for text in ("", "MEMORY_USABLE bytes=0 frames=0", "MEMORY_USABLE bytes=1 frames=1",
                     "MEMORY_USABLE bytes=268435456 frames=65536"):
            with self.subTest(text=text), self.assertRaises(boot.BootError):
                boot.verify_memory_summary(text)
        self.assertEqual(boot.verify_memory_summary("MEMORY_USABLE bytes=8200 frames=2\n"),
                         {"usable_bytes": 8200, "usable_frames": 2})

    def test_heap_sample_requires_full_recovery_after_free(self):
        for text in ("", "HEAP_SAMPLE capacity=1048576 used=16 free=1048560",
                     "HEAP_SAMPLE capacity=0 used=0 free=0"):
            with self.subTest(text=text), self.assertRaises(boot.BootError):
                boot.verify_heap_summary(text)
        self.assertEqual(boot.verify_heap_summary("HEAP_SAMPLE capacity=1048576 used=0 free=1048576\n"),
                         {"capacity_bytes": 1048576, "used_bytes": 0, "free_bytes": 1048576})

    def test_one_irq_stalled_ticks_and_inconsistent_uptime_fail_timer_check(self):
        first = {"ticks": 5, "uptime_ms": 50}
        for samples in ([first], [first, first], [first, {"ticks": 10, "uptime_ms": 99}]):
            with self.subTest(samples=samples), self.assertRaises(boot.BootError):
                boot.verify_timer_samples(samples)
        samples = boot.timer_samples("TIMER_SAMPLE ticks=5 uptime_ms=50\nTIMER_SAMPLE ticks=10 uptime_ms=100\n")
        boot.verify_timer_samples(samples)

    def test_missing_second_irq_times_out(self):
        with tempfile.TemporaryDirectory() as temporary:
            log = Path(temporary) / "serial.log"
            log.write_text("TIMER_SAMPLE ticks=5 uptime_ms=50\n", encoding="utf-8")
            process = Mock()
            process.poll.return_value = None
            with self.assertRaisesRegex(boot.BootError, "timed out"):
                boot.wait_for_timer(process, log, time.monotonic() + 0.01)

    def test_stats_report_requires_the_real_heap_and_a_consistent_uptime(self):
        measured = ("STATS_BEGIN\n"
                    "uptime            00:00:01.000\n"
                    "heap              0 B used of 1048576 B (0.0 %)\n"
                    "heap free         1048576 B\n"
                    "runnable tasks    1\n"
                    "timer ticks       100\n"
                    "context switches  0\n"
                    "dispatches        12\n"
                    "STATS_END\n")
        for text in ("",
                     measured.replace("STATS_END", ""),
                     measured.replace("dispatches        12\n", ""),
                     measured.replace("0 B used of 1048576 B (0.0 %)", "unavailable"),
                     measured.replace("1048576 B (0.0 %)", "65536 B (0.0 %)"),
                     measured.replace("00:00:01.000", "00:00:02.000"),
                     measured.replace("timer ticks       100", "timer ticks       0")):
            with self.subTest(text=text), self.assertRaises(boot.BootError):
                boot.verify_stats_report(text)
        report = boot.verify_stats_report(measured)
        self.assertEqual(report["timer ticks"], "100")
        self.assertEqual(report["heap free"], "1048576 B")

    def test_uniform_or_truncated_screenshot_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            image = Path(temporary) / "screen.ppm"
            image.write_bytes(b"P6\n2 1\n255\n" + b"\x00" * 6)
            with self.assertRaisesRegex(boot.BootError, "uniform blank"):
                boot.verify_screenshot(image)
            image.write_bytes(b"P6\n2 1\n255\n" + b"\x00" * 3)
            with self.assertRaisesRegex(boot.BootError, "incomplete"):
                boot.verify_screenshot(image)
            image.write_bytes(b"P6\n2 1\n255\n" + b"\x00" * 3 + b"\xff" * 3)
            self.assertEqual(boot.verify_screenshot(image), (2, 1))

    def test_qmp_event_cannot_hide_command_error(self):
        connection = Mock(spec=socket.socket)
        connection.recv.side_effect = [
            b'{"QMP": {}}\n',
            b'{"event": "RESET"}\n{"return": {}, "id": 1}\n',
            b'{"event": "STOP"}\n{"error": {"desc": "bad key"}, "id": 2}\n',
        ]
        client = boot.Qmp(connection, time.monotonic() + 1)
        with self.assertRaisesRegex(boot.BootError, "bad key"):
            client.execute("send-key")

    def test_quit_tolerates_qemu_closing_before_reply(self):
        for closing in (ConnectionResetError(104, "Connection reset by peer"), b""):
            connection = Mock(spec=socket.socket)
            connection.recv.side_effect = [b'{"QMP": {}}\n', b'{"return": {}, "id": 1}\n', closing]
            client = boot.Qmp(connection, time.monotonic() + 1)
            client.quit()

    def test_quit_error_reply_is_still_a_failure(self):
        connection = Mock(spec=socket.socket)
        connection.recv.side_effect = [
            b'{"QMP": {}}\n',
            b'{"return": {}, "id": 1}\n',
            b'{"error": {"desc": "quit refused"}, "id": 2}\n',
        ]
        client = boot.Qmp(connection, time.monotonic() + 1)
        with self.assertRaisesRegex(boot.BootError, "quit refused"):
            client.quit()

    def test_closed_connection_still_fails_other_commands(self):
        connection = Mock(spec=socket.socket)
        connection.recv.side_effect = [b'{"QMP": {}}\n', b'{"return": {}, "id": 1}\n', b""]
        client = boot.Qmp(connection, time.monotonic() + 1)
        with self.assertRaisesRegex(boot.BootError, "closed"):
            client.execute("screendump")

    def test_invalid_explicit_qemu_does_not_fall_back(self):
        with patch.dict(boot.os.environ, {"QEMU": "missing-qemu"}), \
                patch.object(boot.shutil, "which", return_value=None):
            with self.assertRaisesRegex(boot.BootError, "missing-qemu"):
                boot.find_qemu()


if __name__ == "__main__":
    unittest.main()
