#!/usr/bin/env python3
"""Measures what a concurrent reader of a version's name meets while one released mpm-shadow publishes over an
unfinished copy at that name.

usage: measure_a_publish_and_read_its_absence.py <arm> <publishes> [--delay-enter-ms <N>] [--refuse-exchange]

Arms, each naming the binary, the file system and the publish the code takes there:

  exchange-on-home-zfs     0.2.7-alpha.14 over an unfinished copy, toolchain on this host's ZFS home
  exchange-on-tmpfs        0.2.7-alpha.14 over an unfinished copy, toolchain on /dev/shm (tmpfs)
  fallback-on-home-zfs     the same binary with the exchange answered EINVAL, so the code publishes in two
  fallback-on-tmpfs        renames; the same on /dev/shm
  two-renames-on-home-zfs  0.2.7-alpha.13, whose publish always moves the copy aside and renames the new one in
  two-renames-on-tmpfs     the same as 0.2.7-alpha.13 on /dev/shm

Each publish runs the released binary's own `install --frozen-lockfile` against a fixture toolchain root, home and
cache of its own, under strace with absolute microsecond timestamps and each call's own duration.  A poller stats
the version's name throughout, so the trace and the poller are two instruments on the same publish.

`--refuse-exchange` makes strace answer the exchange with EINVAL, which is the error a file system that cannot
exchange two names gives, so the code takes its own two-rename fallback.

`--delay-enter-ms` makes strace stop the traced process just before entering each rename, which plants an absence
of that length between the two renames: the positive control that the probe can see one.

The archive comes from a local HTTPS server the harness starts itself, on a loopback port, with a fixture CA that
the binary trusts through SSL_CERT_FILE; nothing reaches the network.

The two roots below are the round's own scratch folder and the tmpfs it made its tmpfs fixtures in; a re-run
points MPM_SHADOW_PUBLISH_SCRATCH and MPM_SHADOW_PUBLISH_TMPFS at its own.  The certificate and key the server
needs are made with openssl into <scratch>/tls: a CA, and a server certificate for 127.0.0.1 signed by it.
"""
import hashlib
import io
import json
import os
import shutil
import socket
import ssl
import subprocess
import sys
import tarfile
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

SCRATCH = Path(os.environ.get(
    "MPM_SHADOW_PUBLISH_SCRATCH",
    "/home/user/temp/agent/mpm-shadow-alpha-14-release-pass-the-busy-start-record-says-what-the-code-and-the-trace-show-scratch",
))
TMPFS = Path(os.environ.get(
    "MPM_SHADOW_PUBLISH_TMPFS",
    "/dev/shm/mpm-shadow-alpha-14-release-pass-the-busy-start-record-says-what-the-code-and-the-trace-show-scratch",
))
BIN14 = SCRATCH / "bin" / "a14" / "mpm-shadow"
BIN13 = SCRATCH / "bin" / "a13" / "mpm-shadow"
POLLER = SCRATCH / "bin-tools" / "poll_the_published_name"
TLS = SCRATCH / "tls"
SERVED = SCRATCH / "servedir"
ARCHIVE = "fixture-tool-0.0.1.tar.gz"
TOOL = "fixture-tool"
VERSION = "0.0.1"
PLATFORM = "linux-x64"
MARKER = ".mpm-shadow-installed"
PORT = 29543


def sha256_of(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def free_port(start):
    for candidate in range(start, start + 200):
        with socket.socket() as probe:
            try:
                probe.bind(("127.0.0.1", candidate))
            except OSError:
                continue
        return candidate
    raise SystemExit("no free port")


# region the fixture


def make_archive(tag):
    data = io.BytesIO()
    with tarfile.open(fileobj=data, mode="w:gz") as archive:
        program = ("#!/bin/sh\necho %s %s %s\n" % (TOOL, VERSION, tag)).encode()
        info = tarfile.TarInfo("%s-%s/%s" % (TOOL, VERSION, TOOL))
        info.mode = 0o755
        info.size = len(program)
        info.mtime = 1700000000
        archive.addfile(info, io.BytesIO(program))
    raw = data.getvalue()
    SERVED.mkdir(parents=True, exist_ok=True)
    (SERVED / ARCHIVE).write_bytes(raw)
    return hashlib.sha256(raw).hexdigest()


def write_plan_and_lock(work, port, digest):
    plan = {
        TOOL: {
            "request": "github:Aquaticats/Monochromatic@" + VERSION,
            "route": "archive",
            "bins": [TOOL],
            "options": {"tagPrefix": "monochromatic-fixture-tool-v"},
        },
    }
    lock = {
        "schemaVersion": 1,
        "generatedFrom": "toolchain.plan.jsonc",
        "tools": {
            TOOL: {
                "request": "github:Aquaticats/Monochromatic@" + VERSION,
                "version": VERSION,
                "route": "archive",
                "bins": [TOOL],
                "env": {},
                "options": {"tagPrefix": "monochromatic-fixture-tool-v"},
                "artifacts": {
                    PLATFORM: {
                        "url": "https://127.0.0.1:%d/%s" % (port, ARCHIVE),
                        "sha256": digest,
                    },
                },
            },
        },
    }
    (work / "toolchain.plan.jsonc").write_text(
        json.dumps({"schemaVersion": 1, "tools": plan}, indent=2) + "\n")
    body = json.dumps(lock, separators=(",", ":"), ensure_ascii=False)
    document = dict(lock)
    document["digest"] = "sha256:" + hashlib.sha256(body.encode()).hexdigest()
    (work / "toolchain.lock.json").write_text(json.dumps(document, indent=2) + "\n")


class Fixture:
    """One disposable world: a home, the XDG roots, a cargo home, a work tree with the plan and the lock."""

    def __init__(self, root, port):
        self.root = Path(root)
        self.home = self.root / "home"
        self.data = self.root / "xdg-data"
        self.cache = self.root / "xdg-cache"
        self.config = self.root / "xdg-config"
        self.cargo = self.root / "cargo-home"
        self.work = self.root / "work"
        self.sub = self.work / "sub"
        self.standins = self.root / "standins"
        self.toolroot = self.data / "monochromatic-dev" / "toolchain"
        self.tooldir = self.toolroot / TOOL
        self.port = port

    def build(self):
        if self.root.exists():
            shutil.rmtree(self.root)
        for directory in (self.home, self.data, self.cache, self.config, self.cargo, self.sub, self.standins):
            directory.mkdir(parents=True)
        os.chmod(self.root, 0o700)
        digest = make_archive("v1")
        write_plan_and_lock(self.work, self.port, digest)
        return self

    def unfinished_version(self):
        """The copy alpha.12 and earlier unpacked in place: a program of the right name, and no completion marker."""
        version_dir = self.toolroot / TOOL / VERSION
        release = version_dir / ("%s-%s" % (TOOL, VERSION))
        release.mkdir(parents=True)
        program = release / TOOL
        program.write_text("#!/bin/sh\necho %s %s old\n" % (TOOL, VERSION))
        os.chmod(program, 0o755)
        os.symlink(os.path.join(TOOL + "-" + VERSION, TOOL), version_dir / TOOL)
        return version_dir

    def snapshot(self):
        directory = self.toolroot / TOOL
        if not directory.exists():
            return ["(no tool directory)"]
        lines = []
        for entry in sorted(directory.iterdir(), key=lambda item: item.name):
            if entry.is_symlink():
                lines.append("%-44s symlink -> %s" % (entry.name, os.readlink(entry)))
                continue
            if not entry.is_dir():
                lines.append("%-44s file" % entry.name)
                continue
            marker = entry / MARKER
            program = [p for p in entry.rglob(TOOL) if p.is_file() and not p.is_symlink()]
            label = "marker=" + (marker.read_text().strip() if marker.is_file() else "none")
            if program:
                label += " program=" + program[0].read_text().strip().splitlines()[-1]
                label += "(exec)" if os.access(program[0], os.X_OK) else "(not exec)"
            lines.append("%-44s dir  %s" % (entry.name, label))
        return lines

    def env(self):
        return {
            "HOME": str(self.home),
            "XDG_DATA_HOME": str(self.data),
            "XDG_CACHE_HOME": str(self.cache),
            "XDG_CONFIG_HOME": str(self.config),
            "CARGO_HOME": str(self.cargo),
            "PATH": str(self.standins) + ":/usr/bin:/bin",
            "SSL_CERT_FILE": str(TLS / "ca.pem"),
            "LC_ALL": "C",
        }


# endregion

# region running things in their own cgroups


def scoped(argv, cwd, env, mem="1G", cpu="200%", tasks=300, secs=120):
    command = [
        "systemd-run", "--user", "--scope", "--slice=agent-work.slice", "--quiet",
        "--property=MemoryMax=" + mem, "--property=CPUQuota=" + cpu,
        "--property=TasksMax=" + str(tasks), "--property=IOWeight=1",
        "--property=RuntimeMaxSec=" + str(secs), "--",
    ]
    command += ["env", "--ignore-environment"]
    command += ["%s=%s" % (key, value) for key, value in env.items()]
    command += [str(part) for part in argv]
    return subprocess.run(command, cwd=str(cwd), capture_output=True, stdin=subprocess.DEVNULL, timeout=secs + 30)


# endregion

# region the trace


def parse_calls(trace_path, fixture_root):
    """Every rename and renameat2 the trace holds: (entry moment, duration, text).

    With --follow-forks every line begins with the process id, then the entry's wall clock, then the call.
    """
    calls = []
    for line in Path(trace_path).read_text().splitlines():
        stripped = line.strip()
        fields = stripped.split(" ", 2)
        if len(fields) < 3:
            continue
        _, stamp, call = fields
        if not call.startswith(("rename(", "renameat(", "renameat2(")):
            continue
        if "= 0 <" not in call and "= 0 (DELAYED) <" not in call:
            continue
        moment = float(stamp)
        duration = float(call.rsplit("<", 1)[1].rstrip(">"))
        text = call.replace(str(fixture_root), "<fixture>")
        calls.append((moment, duration, text))
    return calls


def quoted_paths(text):
    """Every double-quoted path the line names, in order."""
    return [part.split('"', 1)[0] for part in text.split('"')[1::2]]


def publish_calls(calls, version_name):
    """The calls that publish over the version's name: the exchange, or the pair of renames."""
    suffix = "/" + version_name
    for index, (moment, duration, text) in enumerate(calls):
        paths = quoted_paths(text)
        if "RENAME_EXCHANGE" in text and any(path.endswith(suffix) for path in paths):
            return ("exchange", [(moment, duration, text)])
        if text.startswith("rename(") and len(paths) == 2 and paths[0].endswith(suffix) and ".replaced." in paths[1]:
            pair = [(moment, duration, text)]
            for later in calls[index + 1:]:
                later_paths = quoted_paths(later[2])
                if later[2].startswith("rename(") and len(later_paths) == 2 and ".staging." in later_paths[0] and later_paths[1].endswith(suffix):
                    pair.append(later)
                    return ("two-renames", pair)
    return ("none", [])


def report_publish(kind, pair, injected_seconds):
    if kind == "exchange":
        moment, duration, text = pair[0]
        own = duration - injected_seconds
        print("  the publish is one call: %s" % text)
        print("  that call took %.0f us of its own work%s, and both names existed throughout it"
              % (own * 1_000_000, (", with %d ms planted before it" % round(injected_seconds * 1000)) if injected_seconds else ""))
        return
    if kind == "two-renames":
        (first_moment, first_duration, first_text), (second_moment, second_duration, second_text) = pair
        gap = (second_moment - first_moment) * 1_000_000
        own = (first_duration - injected_seconds) * 1_000_000
        absence = gap - own
        print("  the publish is two calls:")
        print("    %s" % first_text)
        print("    %s" % second_text)
        print("  entries %.0f us apart, the first call's own work took %.0f us%s, so the version's name was absent for %.0f us"
              % (gap, own, (", with %d ms planted before it" % round(injected_seconds * 1000)) if injected_seconds else "", absence))
        return
    print("  no publish over the version's name appears in the trace")


# endregion


class QuietHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(SERVED), **kwargs)

    def log_message(self, format, *args):
        print("  request: %s" % (format % args))


def serve():
    port = free_port(PORT)
    server = ThreadingHTTPServer(("127.0.0.1", port), QuietHandler)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.load_cert_chain(str(TLS / "server.pem"), str(TLS / "server.key"))
    server.socket = context.wrap_socket(server.socket, server_side=True)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, port


def main():
    arm = sys.argv[1]
    publishes = int(sys.argv[2])
    delay_enter_ms = 0
    if "--delay-enter-ms" in sys.argv:
        delay_enter_ms = int(sys.argv[sys.argv.index("--delay-enter-ms") + 1])
    refuse_exchange = "--refuse-exchange" in sys.argv
    binary, root = {
        "exchange-on-home-zfs": (BIN14, SCRATCH / "fixtures" / "home-zfs"),
        "exchange-on-tmpfs": (BIN14, TMPFS / "fixtures"),
        "fallback-on-home-zfs": (BIN14, SCRATCH / "fixtures" / "home-zfs"),
        "fallback-on-tmpfs": (BIN14, TMPFS / "fixtures"),
        "two-renames-on-home-zfs": (BIN13, SCRATCH / "fixtures" / "home-zfs"),
        "two-renames-on-tmpfs": (BIN13, TMPFS / "fixtures"),
    }[arm]
    server, port = serve()
    print("arm %s: binary %s  sha256 %s  file system %s  server https://127.0.0.1:%d"
          % (arm, binary, sha256_of(binary), "zfs" if "home-zfs" in arm else "tmpfs", port))
    print("delay before entering each rename: %d ms" % delay_enter_ms)
    print("exchange answered EINVAL: %s" % refuse_exchange)
    try:
        for publish in range(publishes):
            fixture = Fixture(root, port).build()
            version_dir = fixture.unfinished_version()
            old_inode = version_dir.stat().st_ino
            prefix = arm
            if refuse_exchange:
                prefix += "-exchange-answered-EINVAL"
            if delay_enter_ms:
                prefix += "-slowed-%dms" % delay_enter_ms
            trace_path = SCRATCH / "transcripts" / ("%s-publish-%d.strace.txt" % (prefix, publish + 1))
            poller = subprocess.Popen(
                ["systemd-run", "--user", "--scope", "--slice=agent-work.slice", "--quiet",
                 "--property=MemoryMax=64M", "--property=CPUQuota=50%", "--property=TasksMax=8",
                 "--property=IOWeight=1", "--property=RuntimeMaxSec=60", "--",
                 "env", "--ignore-environment", "HOME=" + str(fixture.home),
                 str(POLLER), VERSION, "4000", "20", str(old_inode)],
                cwd=str(fixture.tooldir),
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, stdin=subprocess.DEVNULL)
            time.sleep(0.2)
            started = time.monotonic()
            strace = [
                "strace", "--follow-forks",
                "--absolute-timestamps=format:unix,precision:us", "--syscall-times",
                "--trace=rename,renameat,renameat2",
                "--output", str(trace_path),
            ]
            if delay_enter_ms:
                strace.append("--inject=rename,renameat2:delay_enter=%dms" % delay_enter_ms)
            if refuse_exchange:
                strace.append("--inject=renameat2:error=EINVAL")
            strace += ["--", str(binary), "install", "--frozen-lockfile"]
            done = scoped(strace, fixture.sub, fixture.env(), secs=120)
            install_seconds = time.monotonic() - started
            poll_stdout, poll_stderr = poller.communicate(timeout=60)
            poll_output = poll_stdout.decode(errors="replace").strip()
            if poll_stderr.strip():
                print("  poller stderr: %s" % poll_stderr.decode(errors="replace").strip())
            print("publish %d: install status %d in %.2f s" % (publish + 1, done.returncode, install_seconds))
            if done.returncode != 0:
                print("  stdout: %s" % done.stdout.decode(errors="replace").strip()[:800])
                print("  stderr: %s" % done.stderr.decode(errors="replace").strip()[:800])
            calls = parse_calls(trace_path, fixture.root)
            kind, pair = publish_calls(calls, VERSION)
            report_publish(kind, pair, delay_enter_ms / 1000.0)
            print("  %s" % poll_output.replace(str(fixture.root), "<fixture>"))
            print("  after: %s" % [line.replace(str(fixture.root), "<fixture>") for line in fixture.snapshot()])
            print("  trace: %s" % trace_path.name)
    finally:
        server.shutdown()
        server.server_close()


if __name__ == "__main__":
    main()
