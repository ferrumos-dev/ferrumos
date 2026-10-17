#!/usr/bin/env python3
"""Ferrum OS package-watch experiment.

This is a host-side teaching prototype, not pacman, Guix, pkgutils, or a
Ferrum-native package manager. Payloads are text fixtures, not executables.
Only exact dependency versions are supported. Run on Linux/macOS or WSL.
Requires Python >=3.10 and cryptography. No network or elevated privileges.
"""

import argparse
import gzip
import hashlib
import io
import json
import os
import platform
import re
import shutil
import sys
import tarfile
import tempfile
from datetime import datetime, timezone
from importlib.metadata import version as package_version
from pathlib import Path, PurePosixPath

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey


TARGET = "x86_64-ferrum-demo-abi1"  # Illustrative label, not an official Rust target.
TOKEN = re.compile(r"[a-z0-9][a-z0-9._+-]*\Z")
SHA = re.compile(r"[a-f0-9]{64}\Z")


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


class Rejected(Exception):
    pass


def identity(name, version):
    if not isinstance(name, str) or not TOKEN.fullmatch(name):
        raise Rejected("INVALID_NAME")
    if not isinstance(version, str) or not TOKEN.fullmatch(version):
        raise Rejected("INVALID_VERSION")
    return name + "@" + version


def make_archive(metadata, files):
    """Deterministic assembly for identical fixture inputs, not a compiler."""
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.USTAR_FORMAT) as tar:
        entries = {"manifest.json": canonical(metadata)}
        entries.update({"payload/" + p: data for p, data in files.items()})
        for path, data in sorted(entries.items()):
            member = tarfile.TarInfo(path)
            member.size = len(data)
            member.mtime = 0
            member.uid = member.gid = 0
            member.uname = member.gname = ""
            member.mode = 0o644
            tar.addfile(member, io.BytesIO(data))
    out = io.BytesIO()
    with gzip.GzipFile(fileobj=out, mode="wb", filename="", mtime=0) as zipped:
        zipped.write(raw.getvalue())
    return out.getvalue()


class Repository:
    def __init__(self, path):
        self.path = path
        self.path.mkdir()
        self.entries = {}
        self.key = Ed25519PrivateKey.generate()  # Ephemeral experiment key only.

    def add(self, name, version, files, dependencies=None, target=TARGET):
        metadata = {"name": name, "version": version, "target": target,
                    "dependencies": dependencies or {}}
        archive = make_archive(metadata, files)
        sha = digest(archive)
        (self.path / (sha + ".tar.gz")).write_bytes(archive)
        self.entries[identity(name, version)] = {**metadata, "sha256": sha}
        return archive

    def publish(self):
        raw = canonical({"format": 1, "packages": self.entries})
        (self.path / "index.json").write_bytes(raw)
        (self.path / "index.sig").write_bytes(self.key.sign(raw))


class Client:
    def __init__(self, repository, root):
        self.repo = repository.path
        self.trusted_key = repository.key.public_key()
        self.root = root
        self.last_order = []

    def load_index(self):
        raw = (self.repo / "index.json").read_bytes()
        try:
            self.trusted_key.verify((self.repo / "index.sig").read_bytes(), raw)
        except InvalidSignature as error:
            raise Rejected("INVALID_SIGNATURE") from error
        index = json.loads(raw)
        if index.get("format") != 1:
            raise Rejected("UNSUPPORTED_FORMAT")
        return index["packages"]

    def resolve(self, requested, entries):
        chosen, active, completed, order = {}, set(), set(), []

        def visit(name, version):
            key = identity(name, version)
            if name in chosen and chosen[name] != version:
                raise Rejected("DEPENDENCY_CONFLICT")
            if key in active:
                raise Rejected("DEPENDENCY_CYCLE")
            if key in completed:
                return
            if key not in entries:
                raise Rejected("MISSING_DEPENDENCY")
            spec = entries[key]
            if spec["name"] != name or spec["version"] != version:
                raise Rejected("INDEX_IDENTITY_MISMATCH")
            if spec["target"] != TARGET:
                raise Rejected("INCOMPATIBLE_TARGET")
            chosen[name] = version
            active.add(key)
            for dep, required in sorted(spec["dependencies"].items()):
                visit(dep, required)
            active.remove(key)
            completed.add(key)
            order.append(key)

        for name, version in sorted(requested.items()):
            visit(name, version)
        return order

    def verify_payloads(self, order, entries):
        all_files = {}
        for key in order:
            spec = entries[key]
            sha = spec["sha256"]
            if not isinstance(sha, str) or not SHA.fullmatch(sha):
                raise Rejected("INVALID_DIGEST")
            raw = (self.repo / (sha + ".tar.gz")).read_bytes()
            if digest(raw) != sha:
                raise Rejected("ARCHIVE_DIGEST_MISMATCH")
            expected = {field: spec[field] for field in
                        ("name", "version", "target", "dependencies")}
            with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as tar:
                manifest, seen = None, set()
                for member in tar:
                    if member.name in seen or not member.isfile():
                        raise Rejected("UNSUPPORTED_ARCHIVE_ENTRY")
                    seen.add(member.name)
                    stream = tar.extractfile(member)
                    data = stream.read()
                    if member.name == "manifest.json":
                        manifest = json.loads(data)
                        continue
                    if not member.name.startswith("payload/"):
                        raise Rejected("UNSAFE_PATH")
                    relative = member.name[len("payload/"):]
                    path = PurePosixPath(relative)
                    if (not relative or path.is_absolute() or "\\" in relative
                            or any(p in ("", ".", "..") for p in relative.split("/"))):
                        raise Rejected("UNSAFE_PATH")
                    if relative == ".packages.json":
                        raise Rejected("RESERVED_PATH")
                    if any(relative == p or relative.startswith(p + "/")
                           or p.startswith(relative + "/") for p in all_files):
                        raise Rejected("FILE_CONFLICT")
                    all_files[relative] = data
                if manifest != expected:
                    raise Rejected("MANIFEST_MISMATCH")
        return all_files

    def current(self):
        link = self.root / "current"
        return os.readlink(link) if link.is_symlink() else None

    def activate(self, relative_generation):
        generations = self.root / "generations"
        candidate = self.root / relative_generation
        if (candidate.parent != generations or not candidate.is_dir()
                or not (candidate / ".packages.json").is_file()):
            raise Rejected("UNKNOWN_GENERATION")
        temp_link = self.root / ".current-next"
        if temp_link.is_symlink():
            temp_link.unlink()
        os.symlink(relative_generation, temp_link)
        os.replace(temp_link, self.root / "current")

    def install(self, requested, fail_before_activation=False, fail_during_staging=False):
        # requested is the COMPLETE desired profile, not an additive install list.
        # Validate that set before creating or changing an installation.
        entries = self.load_index()
        order = self.resolve(requested, entries)
        files = self.verify_payloads(order, entries)
        generations = self.root / "generations"
        generations.mkdir(parents=True, exist_ok=True)
        existing = [int(p.name[1:]) for p in generations.iterdir()
                    if re.fullmatch(r"g[0-9]+", p.name)]
        name = "g%04d" % (max(existing, default=0) + 1)
        staged = Path(tempfile.mkdtemp(prefix=".stage-", dir=generations))
        try:
            for relative, data in sorted(files.items()):
                path = staged / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
                if fail_during_staging:
                    raise Rejected("INJECTED_WRITE_FAILURE")
            (staged / ".packages.json").write_bytes(canonical({"packages": order}))
            os.replace(staged, generations / name)
        finally:
            if staged.exists():
                shutil.rmtree(staged)
        if fail_before_activation:
            raise Rejected("INJECTED_STOP_BEFORE_ACTIVATION")
        self.activate("generations/" + name)
        self.last_order = order
        return "generations/" + name

    def read(self, relative):
        return (self.root / "current" / relative).read_bytes()


def demand(condition, message):
    if not condition:
        raise AssertionError(message)


def expect_rejection(client, requested, expected, **kwargs):
    before = client.current()
    try:
        client.install(requested, **kwargs)
    except Rejected as error:
        demand(str(error) == expected, "unexpected rejection: " + str(error))
    else:
        raise AssertionError("operation unexpectedly accepted")
    demand(client.current() == before, "active generation changed after rejection")
    return {"observed_error": expected, "active_generation_unchanged": True}


def run():
    results = []

    def check(identifier, title, operation):
        try:
            evidence = operation()
            results.append({"id": identifier, "scenario": title,
                            "status": "PASS", "evidence": evidence})
        except Exception as error:
            results.append({"id": identifier, "scenario": title,
                            "status": "FAIL", "error": repr(error)})

    with tempfile.TemporaryDirectory(prefix="ferrum-package-watch-") as directory:
        work = Path(directory)
        repo = Repository(work / "repository")
        for version in ("1.0.0", "2.0.0", "3.0.0"):
            repo.add("base", version, {"lib/base.txt": ("base " + version).encode()})
            repo.add("hello", version, {"bin/hello.txt": ("hello " + version).encode()},
                     {"base": version})
        repo.add("missing", "1.0.0", {"bin/missing.txt": b"fixture"}, {"ghost": "1.0.0"})
        repo.add("cycle-a", "1.0.0", {}, {"cycle-b": "1.0.0"})
        repo.add("cycle-b", "1.0.0", {}, {"cycle-a": "1.0.0"})
        repo.add("wrong-target", "1.0.0", {"bin/other.txt": b"fixture"},
                 target="aarch64-other-demo-abi9")
        repo.add("bad-path", "1.0.0", {"../outside.txt": b"must never be written"})
        repo.add("collision", "1.0.0", {"bin/hello.txt": b"conflicting file"})
        repo.publish()
        client = Client(repo, work / "installation")
        state = {}

        def deterministic():
            metadata = {"name": "repro", "version": "1.0.0", "target": TARGET,
                        "dependencies": {}}
            a = make_archive(metadata, {"b.txt": b"B", "a.txt": b"A"})
            b = make_archive(metadata, {"a.txt": b"A", "b.txt": b"B"})
            demand(a == b, "archives differ")
            return {"sha256_run_1": digest(a), "sha256_run_2": digest(b),
                    "scope": "fixed payload archive assembly, not compilation"}

        check("E01", "Assemblage deterministe de deux archives identiques", deterministic)

        def initial_install():
            state["g1"] = client.install({"hello": "1.0.0"})
            demand(client.last_order == ["base@1.0.0", "hello@1.0.0"], "wrong order")
            demand(client.read("bin/hello.txt") == b"hello 1.0.0", "wrong payload")
            demand(client.read("lib/base.txt") == b"base 1.0.0", "missing dependency")
            return {"generation": state["g1"], "dependency_order": client.last_order}

        check("E02", "Installation avec dependance exacte et index signe", initial_install)
        check("E03", "Refus d'une contrainte de versions incompatible", lambda:
              expect_rejection(client, {"hello": "1.0.0", "base": "2.0.0"}, "DEPENDENCY_CONFLICT"))
        check("E04", "Refus d'une dependance absente", lambda:
              expect_rejection(client, {"missing": "1.0.0"}, "MISSING_DEPENDENCY"))
        check("E05", "Refus d'un cycle de dependances", lambda:
              expect_rejection(client, {"cycle-a": "1.0.0"}, "DEPENDENCY_CYCLE"))
        check("E06", "Refus d'une cible ou ABI differente", lambda:
              expect_rejection(client, {"wrong-target": "1.0.0"}, "INCOMPATIBLE_TARGET"))

        def corrupted_archive():
            archive = repo.path / (repo.entries["hello@2.0.0"]["sha256"] + ".tar.gz")
            original = archive.read_bytes()
            try:
                archive.write_bytes(original + b"tampered")
                return expect_rejection(client, {"hello": "2.0.0"}, "ARCHIVE_DIGEST_MISMATCH")
            finally:
                archive.write_bytes(original)

        check("E07", "Refus d'une archive modifiee apres signature de l'index", corrupted_archive)

        def forged_index():
            index = repo.path / "index.json"
            original = index.read_bytes()
            try:
                index.write_bytes(original + b" ")
                return expect_rejection(client, {"hello": "2.0.0"}, "INVALID_SIGNATURE")
            finally:
                index.write_bytes(original)

        check("E08", "Refus d'un index dont la signature ne correspond plus", forged_index)
        check("E09", "Refus d'une archive contenant un chemin ../", lambda:
              expect_rejection(client, {"bad-path": "1.0.0"}, "UNSAFE_PATH"))
        check("E10", "Refus d'une collision de fichiers entre deux paquets", lambda:
              expect_rejection(client, {"hello": "1.0.0", "collision": "1.0.0"}, "FILE_CONFLICT"))

        def update():
            state["g2"] = client.install({"hello": "2.0.0"})
            demand(client.read("bin/hello.txt") == b"hello 2.0.0", "update failed")
            demand(client.read("lib/base.txt") == b"base 2.0.0", "dependency not updated")
            demand((client.root / state["g1"] / "bin/hello.txt").read_bytes() == b"hello 1.0.0",
                   "old generation overwritten")
            return {"new_generation": state["g2"], "old_generation_retained": True}

        check("E11", "Mise a jour coherente du paquet et de sa dependance", update)
        check("E12", "Exception injectee avant activation de la generation suivante", lambda:
              expect_rejection(client, {"hello": "3.0.0"}, "INJECTED_STOP_BEFORE_ACTIVATION",
                               fail_before_activation=True))

        def staging_recovery():
            evidence = expect_rejection(client, {"hello": "3.0.0"}, "INJECTED_WRITE_FAILURE",
                                        fail_during_staging=True)
            demand(not list((client.root / "generations").glob(".stage-*")), "staging left behind")
            client.install({"hello": "2.0.0"})
            demand(client.read("bin/hello.txt") == b"hello 2.0.0", "retry failed")
            return {**evidence, "temporary_directory_removed": True, "retry_succeeded": True}

        check("E13", "Reprise apres une exception injectee pendant la preparation", staging_recovery)

        def rollback():
            mutable = client.root / "state" / "user-data.txt"
            mutable.parent.mkdir()
            mutable.write_text("modified after update", encoding="utf-8")
            client.activate(state["g1"])
            demand(client.read("bin/hello.txt") == b"hello 1.0.0", "rollback failed")
            demand(client.read("lib/base.txt") == b"base 1.0.0", "dependency rollback failed")
            demand(mutable.read_text(encoding="utf-8") == "modified after update", "user data rolled back")
            return {"restored_generation": state["g1"], "package_versions_restored": True,
                    "mutable_user_data_restored": False}

        check("E14", "Retour arriere des logiciels sans restauration des donnees utilisateur", rollback)

    return {
        "study": "Ferrum OS - host-side package mechanism experiments",
        "performed_at_utc": datetime.now(timezone.utc).isoformat(),
        "environment": {"python": platform.python_version(), "system": platform.platform(),
                        "cryptography": package_version("cryptography"), "target_label": TARGET},
        "source_sha256": digest(Path(__file__).read_bytes()),
        "results": results,
        "passed": sum(r["status"] == "PASS" for r in results),
        "total": len(results),
        "limitations": [
            "Upstream pacman, Guix and Redox tools were not executed.",
            "No Ferrum kernel, native executable, compiler or QEMU was used.",
            "Target and ABI checks compare illustrative metadata only.",
            "Exact version pins only; no ranges, SAT solving or virtual packages.",
            "Each requested set replaces the complete profile; additive installation is not implemented.",
            "Single writer, small trusted fixtures; no general archive-size/resource hardening.",
            "Ed25519 experiment keys are ephemeral; no key rotation, revocation, expiry or replay protection.",
            "Generation switching uses an OS symlink rename; no filesystem power-loss durability proof.",
            "Interruptions are injected exceptions during staging and before activation, not physical power cuts.",
            "A killed process can leave unused staging/generation directories; persistent cleanup is not implemented.",
            "Retained generations are logical directories, not an enforced read-only store.",
            "Archive byte reproducibility is not compiler/build reproducibility.",
            "No performance comparison, restart/migration policy or disk-space optimization is provided."
        ]
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", default="results.json")
    args = parser.parse_args()
    report = run()
    Path(args.output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    for row in report["results"]:
        print(row["id"], row["status"], row["scenario"])
        if row["status"] != "PASS":
            print("  ", row["error"])
    print(str(report["passed"]) + "/" + str(report["total"]) + " passed; report: " + args.output)
    sys.exit(0 if report["passed"] == report["total"] else 1)
