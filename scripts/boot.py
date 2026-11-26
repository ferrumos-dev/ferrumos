#!/usr/bin/env python3
"""Build, run or smoke-test the real FerrumOS BIOS image (Python 3.11+)."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import socket
import subprocess
import sys
import time
import tomllib

ROOT = Path(__file__).resolve().parents[1]
BOOT = ROOT / "target" / "boot"
IMAGE = BOOT / "ferrumos-bios.img"
RESULTS = BOOT / "results"
MARKERS = ("FERRUMOS_BOOT_OK", "MEMORY_MAP_READY", "HEAP_READY", "HEAP_ALLOC_FREE_OK",
           "DISPLAY_READY", "KEYBOARD_READY", "TIMER_READY", "HEAP_IRQ_SAFE_OK",
           "INPUT_TASK_READY", "KERNEL_PRIORITY_OK")
ACK = "INPUT_LINE: test"


class BootError(RuntimeError):
    """A build or VM check failed; never continue with an old image."""


class QmpClosed(BootError):
    """QEMU closed the QMP connection before sending the awaited reply."""


def toolchain(directory: Path) -> str:
    with (directory / "rust-toolchain.toml").open("rb") as source:
        return tomllib.load(source)["toolchain"]["channel"]


def build_environment() -> dict[str, str]:
    """Select targets and toolchains here, independently of a caller's overrides."""
    environment = os.environ.copy()
    for name in (
        "RUSTUP_TOOLCHAIN", "CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
    ):
        environment.pop(name, None)
    return environment


def build_image() -> Path:
    """Rebuild the kernel and publish an image only if both build steps succeed."""
    BOOT.mkdir(parents=True, exist_ok=True)
    temporary = IMAGE.with_suffix(".img.partial")
    IMAGE.unlink(missing_ok=True)
    temporary.unlink(missing_ok=True)
    environment = build_environment()
    helper = ROOT / "tools" / "image-builder"
    stable = toolchain(ROOT)
    nightly = toolchain(helper)
    kernel = ROOT / "target" / "x86_64-unknown-none" / "debug" / "ferrumos"
    log_path = BOOT / "build.log"
    commands = [
        (ROOT, ["cargo", f"+{stable}", "build", "--locked", "--bin", "ferrumos",
                "--features", "baremetal", "--target", "x86_64-unknown-none",
                "--target-dir", str(ROOT / "target")]),
        (helper, ["cargo", f"+{nightly}", "run", "--locked",
                  "--target-dir", str(ROOT / "target" / "image-builder"), "--",
                  str(kernel), str(temporary)]),
    ]
    try:
        with log_path.open("w", encoding="utf-8") as log:
            for directory, command in commands:
                print(f"Building {'kernel' if directory == ROOT else 'BIOS image'}...", flush=True)
                log.write(f"{command!r}\n")
                log.flush()
                subprocess.run(command, cwd=directory, env=environment, stdin=subprocess.DEVNULL,
                               stdout=log, stderr=subprocess.STDOUT, check=True)
        if not temporary.is_file() or temporary.stat().st_size == 0:
            raise BootError("image builder returned without a nonempty image")
        temporary.replace(IMAGE)
    except (OSError, subprocess.CalledProcessError, BootError) as error:
        temporary.unlink(missing_ok=True)
        raise BootError(f"build failed; see {log_path}: {error}") from error
    print(f"Image: {IMAGE}", flush=True)
    return IMAGE


def find_qemu() -> str:
    override = os.environ.get("QEMU")
    if override:
        candidate = shutil.which(override)
        if candidate:
            return candidate
        raise BootError(f"QEMU executable not found: {override}")
    candidate = shutil.which("qemu-system-x86_64")
    if candidate:
        return candidate
    if os.name == "nt":
        candidate_path = Path(os.environ.get("ProgramFiles", "C:/Program Files")) / "qemu/qemu-system-x86_64.exe"
        if candidate_path.is_file():
            return str(candidate_path)
    raise BootError("QEMU is missing; install qemu-system-x86_64 or set QEMU to its executable path")


def qemu_arguments(executable: str, image: Path, serial: Path) -> list[str]:
    # QEMU key/value options escape literal commas by doubling them.
    drive = image.as_posix().replace(",", ",,")
    return [executable, "-machine", "pc", "-accel", "tcg", "-m", "128M", "-smp", "1",
            "-drive", f"format=raw,file={drive}", "-vga", "std",
            "-serial", f"file:{serial.as_posix()}", "-monitor", "none",
            "-no-reboot", "-no-shutdown"]


def ensure_running(process: subprocess.Popen) -> None:
    code = process.poll()
    if code is not None:
        raise BootError(f"QEMU exited before the check completed (code {code})")


def wait_for_serial(process: subprocess.Popen, path: Path, expected: tuple[str, ...],
                    deadline: float) -> str:
    while time.monotonic() < deadline:
        ensure_running(process)
        text = path.read_text(encoding="utf-8", errors="replace") if path.exists() else ""
        lines = text.splitlines()
        if any("PANIC" in line or "BOOT_FAILED" in line or "CPU_EXCEPTION" in line for line in lines):
            raise BootError("kernel reported a panic or boot failure")
        if all(marker in lines for marker in expected):
            return text
        time.sleep(0.05)
    raise BootError(f"timed out waiting for serial lines: {', '.join(expected)}")


def verify_memory_summary(text: str) -> dict[str, int]:
    """Check the boot-time usable map summary for the 128 MiB test machine."""
    match = re.search(r"^MEMORY_USABLE bytes=(\d+) frames=(\d+)$", text, re.MULTILINE)
    if match is None:
        raise BootError("missing physical memory summary")
    usable, frames = map(int, match.groups())
    if not 0 < frames * 4096 <= usable <= 128 * 1024 * 1024:
        raise BootError("invalid physical memory summary for the QEMU test machine")
    return {"usable_bytes": usable, "usable_frames": frames}


def verify_heap_summary(text: str) -> dict[str, int]:
    """Require a real heap sample after allocate, write, free, reuse and OOM checks."""
    match = re.search(r"^HEAP_SAMPLE capacity=(\d+) used=(\d+) free=(\d+)$", text, re.MULTILINE)
    if match is None:
        raise BootError("missing heap sample")
    capacity, used, free = map(int, match.groups())
    if capacity != 1024 * 1024 or used != 0 or free != capacity:
        raise BootError("heap sample did not recover its full capacity")
    return {"capacity_bytes": capacity, "used_bytes": used, "free_bytes": free}


def timer_samples(text: str) -> list[dict[str, int]]:
    """Read immutable snapshots; the kernel emits at most two during boot."""
    # The log is read while QEMU is still writing it one byte at a time. The
    # trailing newline is required so a half-written line, whose number would be
    # cut short, is left for the next read instead of becoming a sample.
    return [{"ticks": int(ticks), "uptime_ms": int(milliseconds)}
            for ticks, milliseconds in re.findall(r"^TIMER_SAMPLE ticks=(\d+) uptime_ms=(\d+)\r?\n",
                                                   text, re.MULTILINE)]


def verify_timer_samples(samples: list[dict[str, int]]) -> None:
    """Check repeated IRQ delivery and the documented PIT rational conversion."""
    if len(samples) != 2:
        raise BootError("expected two periodic timer samples")
    previous = {"ticks": 0, "uptime_ms": 0}
    for sample in samples:
        # The PIT base and programmed divisor define uptime. Host wall time
        # is deliberately excluded because TCG may run at a different speed.
        expected_ms = min(sample["ticks"] * 11932 * 1000 // 1193182, (1 << 64) - 1)
        if (sample["ticks"] <= previous["ticks"] or sample["uptime_ms"] <= previous["uptime_ms"]
                or sample["uptime_ms"] != expected_ms):
            raise BootError("timer samples are not increasing with the PIT period")
        previous = sample


def wait_for_timer(process: subprocess.Popen, serial: Path, deadline: float) -> list[dict[str, int]]:
    """Require two distinct interrupts; one IRQ cannot establish a working EOI path."""
    while time.monotonic() < deadline:
        ensure_running(process)
        text = serial.read_text(encoding="utf-8", errors="replace") if serial.exists() else ""
        if "PANIC" in text or "BOOT_FAILED" in text or "CPU_EXCEPTION" in text:
            raise BootError("kernel failed while waiting for periodic timer interrupts")
        samples = timer_samples(text)
        if len(samples) >= 2:
            verify_timer_samples(samples)
            return samples
        time.sleep(0.02)
    raise BootError("timed out waiting for periodic timer interrupts")


def verify_stats_report(text: str) -> dict[str, str]:
    """Require the stats report to read the mapped heap and the delivered ticks."""
    match = re.search(r"^STATS_BEGIN$(.*?)^STATS_END$", text, re.MULTILINE | re.DOTALL)
    if match is None:
        raise BootError("missing kernel statistics report")
    body = match.group(1)
    lines = [line for line in body.splitlines() if line]
    if len(lines) != 7:
        raise BootError("the statistics report does not hold its seven measurements")
    if "unavailable" in body:
        raise BootError("the statistics report still has an unavailable measurement")
    # The view pads every label to a fixed column, so the split is positional.
    report = {line[:18].strip(): line[18:].strip() for line in lines}
    heap = re.fullmatch(r"(\d+) B used of (\d+) B \(\d+\.\d %\)", report.get("heap", ""))
    if heap is None or int(heap.group(2)) != 1024 * 1024:
        raise BootError("the statistics report does not read the mapped kernel heap")
    uptime = re.fullmatch(r"(\d+):(\d{2}):(\d{2})\.(\d{3})", report.get("uptime", ""))
    if uptime is None:
        raise BootError("the statistics report does not express its uptime")
    hours, minutes, seconds, milliseconds = map(int, uptime.groups())
    ticks = int(report.get("timer ticks", "0"))
    # Same rational PIT conversion as the timer samples, from the report's own
    # tick count. Host wall time stays excluded because TCG speed varies.
    expected_ms = min(ticks * 11932 * 1000 // 1193182, (1 << 64) - 1)
    if ticks <= 0 or ((hours * 60 + minutes) * 60 + seconds) * 1000 + milliseconds != expected_ms:
        raise BootError("the statistics report's uptime does not match its tick count")
    return report


class Qmp:
    """Small synchronous QMP client; unsolicited events never satisfy a command."""

    def __init__(self, connection: socket.socket, deadline: float):
        self.connection = connection
        self.deadline = deadline
        self.buffer = b""
        self.sequence = 0
        if "QMP" not in self.receive():
            raise BootError("invalid QMP greeting")
        self.execute("qmp_capabilities")

    def receive(self) -> dict:
        while b"\n" not in self.buffer:
            remaining = self.deadline - time.monotonic()
            if remaining <= 0:
                raise BootError("QMP response timed out")
            self.connection.settimeout(remaining)
            chunk = self.connection.recv(65536)
            if not chunk:
                raise QmpClosed("QMP connection closed")
            self.buffer += chunk
        line, self.buffer = self.buffer.split(b"\n", 1)
        return json.loads(line)

    def execute(self, command: str, arguments: dict | None = None) -> dict:
        self.sequence += 1
        message = {"execute": command, "id": self.sequence}
        if arguments is not None:
            message["arguments"] = arguments
        self.connection.sendall(json.dumps(message).encode("utf-8") + b"\n")
        while True:
            reply = self.receive()
            if reply.get("id") != self.sequence:
                continue
            if "error" in reply:
                raise BootError(f"QMP {command} failed: {reply['error']}")
            if "return" not in reply:
                raise BootError(f"QMP {command} returned an invalid response")
            return reply["return"]

    def quit(self) -> None:
        """Ask QEMU to exit.

        QEMU may close the connection before its reply arrives, so a lost reply
        is not a failure: the caller judges the outcome from the exit code.
        """
        try:
            self.execute("quit")
        except (ConnectionError, QmpClosed):
            pass


def connect_qmp(process: subprocess.Popen, port: int, deadline: float) -> socket.socket:
    while time.monotonic() < deadline:
        ensure_running(process)
        try:
            return socket.create_connection(("127.0.0.1", port), timeout=0.25)
        except (ConnectionRefusedError, TimeoutError):
            time.sleep(0.05)
    raise BootError("QMP connection timed out")


def verify_screenshot(path: Path) -> tuple[int, int]:
    data = path.read_bytes()
    header = re.match(rb"P6\s+(\d+)\s+(\d+)\s+255\s", data)
    if header is None:
        raise BootError("QEMU screenshot is not an RGB PPM image")
    width, height = map(int, header.groups())
    pixels = data[header.end():]
    if width <= 0 or height <= 0 or len(pixels) != width * height * 3:
        raise BootError("QEMU screenshot is incomplete")
    first = pixels[:3]
    if all(pixels[index:index + 3] == first for index in range(0, len(pixels), 3)):
        raise BootError("QEMU framebuffer is a uniform blank image")
    return width, height


def stop_process(process: subprocess.Popen) -> None:
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def prepare_test_results() -> None:
    """Never attach a previous passing VM verdict to a failed new build."""
    RESULTS.mkdir(parents=True, exist_ok=True)
    for name in ("serial.log", "qemu.stderr.log", "screen.ppm", "summary.json"):
        (RESULTS / name).unlink(missing_ok=True)
    (RESULTS / "summary.json").write_text(
        json.dumps({"status": "not-run", "stage": "preparation"}, indent=2) + "\n",
        encoding="utf-8",
    )


def smoke_test(image: Path, executable: str, timeout: float) -> None:
    RESULTS.mkdir(parents=True, exist_ok=True)
    serial = RESULTS / "serial.log"
    screenshot = RESULTS / "screen.ppm"
    summary_path = RESULTS / "summary.json"
    for path in (serial, screenshot, summary_path):
        path.unlink(missing_ok=True)
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    command = qemu_arguments(executable, image, serial) + [
        "-display", "none", "-qmp", f"tcp:127.0.0.1:{port},server=on,wait=off",
    ]
    summary: dict = {"status": "failed", "image": str(image), "image_sha256": hashlib.sha256(image.read_bytes()).hexdigest(),
                     "command": command, "timeout_seconds": timeout}
    started = time.monotonic()
    deadline = started + timeout
    process = None
    connection = None
    try:
        with (RESULTS / "qemu.stderr.log").open("wb") as error_log:
            process = subprocess.Popen(command, cwd=ROOT, stdin=subprocess.DEVNULL,
                                       stdout=error_log, stderr=subprocess.STDOUT,
                                       creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
            connection = connect_qmp(process, port, deadline)
            qmp = Qmp(connection, deadline)
            boot_text = wait_for_serial(process, serial, MARKERS, deadline)
            summary["physical_memory"] = verify_memory_summary(boot_text)
            summary["heap"] = verify_heap_summary(boot_text)
            summary["timer"] = wait_for_timer(process, serial, deadline)
            summary["stats"] = verify_stats_report(
                wait_for_serial(process, serial, ("STATS_END",), deadline))
            for key in ("t", "e", "s", "x", "backspace", "t", "ret"):
                qmp.execute("send-key", {"keys": [{"type": "qcode", "data": key}], "hold-time": 50})
                time.sleep(0.08)
            wait_for_serial(process, serial, (ACK,), deadline)
            qmp.execute("screendump", {"filename": screenshot.as_posix()})
            summary["framebuffer"] = verify_screenshot(screenshot)
            ensure_running(process)
            qmp.quit()
            process.wait(timeout=max(0.1, deadline - time.monotonic()))
            if process.returncode != 0:
                raise BootError(f"QEMU quit with code {process.returncode}")
            summary["status"] = "passed"
            summary["serial_markers"] = [*MARKERS, ACK]
    except (OSError, ValueError, subprocess.SubprocessError, BootError) as error:
        summary["error"] = str(error)
        raise BootError(f"boot smoke test failed: {error}; diagnostics: {RESULTS}") from error
    finally:
        if connection is not None:
            connection.close()
        if process is not None:
            stop_process(process)
            summary["qemu_exit_code"] = process.returncode
        summary["duration_seconds"] = round(time.monotonic() - started, 3)
        summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"PASS: heap allocation/free, periodic timer, framebuffer and PS/2 keyboard; diagnostics: {RESULTS}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("build", "run", "test"))
    parser.add_argument("--timeout", type=float, default=30, help="VM smoke-test timeout in seconds (default: 30)")
    arguments = parser.parse_args()
    if not 0 < arguments.timeout <= 300:
        parser.error("--timeout must be greater than 0 and at most 300 seconds")
    try:
        if arguments.command == "test":
            prepare_test_results()
        executable = find_qemu() if arguments.command != "build" else None
        image = build_image()
        if arguments.command == "test":
            smoke_test(image, executable, arguments.timeout)
        elif arguments.command == "run":
            serial = BOOT / "serial.log"
            print("Starting QEMU. Click in the window to type; Ctrl+Alt releases input.")
            result = subprocess.run(qemu_arguments(executable, image, serial), cwd=ROOT, check=False)
            if result.returncode != 0:
                raise BootError(f"QEMU exited with code {result.returncode}")
        return 0
    except (OSError, BootError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("Interrupted.", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
