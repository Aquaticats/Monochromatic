# What:  base image of the native wrapper gate (`mise run //package/git-policy/cli:native:test:container`) and of the
#        mutation campaigns that start from the gate image: CachyOS with the Rust toolchain, the build tools, and every
#        tool the tests start (Git, Node, pnpm, strace, cargo-nextest), all installed from pacman, plus the pinned npm
#        data tarballs that `file-enforcer check` resolves a `.browserslistrc` against.
# Why:   the gate image itself builds without network, so every download happens here, once per CachyOS image: the
#        build script (bin/native-test-base.ts) pulls the `latest` tag, and this image is rebuilt only when that pull
#        brings a different image, when this file changes, or when the browserslist data lock changes
#        (doc/decision/container-base-distro.md).
# Pins:  none. Owner decisions of 2026-11-04: "We don't need to pin git because my current system is already using git
#        latest.", "We don't need to pin Node too.", and, asked where the tools toolchain.lock.json locks should come
#        from in test images, "Pacman for everything it has" (an exception to AGENTS.md rule CM7 for these images).
#        So Git is whatever pacman's current `git` is, and Node, pnpm, strace, cargo-nextest and Rust are pacman's
#        current packages too; `mise run //package/git-policy/cli:test:git-image` prints the release of each the image
#        got. The wrapper's MINIMUM_GIT_VERSION (src/native/invocation/git_version.rs) follows the Git release here.
# Signatures: pacman checks package signatures (`SigLevel = Required DatabaseOptional` in the CachyOS image's
#        /etc/pacman.conf); nothing below turns that off.
# Build context: any directory; nothing is copied from it.
# Build arguments: CACHYOS_IMAGE (the base, resolved by package/test-harness/core-matrix); BROWSERSLIST_DATA_LOCK (the
#        text of the repository's browserslist-data.lock.json, read and passed by bin/native-test-base.ts).
ARG CACHYOS_IMAGE=docker.io/cachyos/cachyos:latest
FROM ${CACHYOS_IMAGE}

# --disable-sandbox: pacman's hook sandbox needs a network namespace a rootless podman build cannot give it
# (doc/troubleshooting/pacman-hook-sandbox-rootless-podman-build.md). The cache mount keeps downloaded packages between
# builds outside the image. `rust` carries cargo and Clippy. `base-devel` carries the compiler and linker that build the
# crates. `strace` injects the faults of the fault-injection tests (they check nothing without it). `nodejs` and `pnpm`
# serve the controls that run a real pnpm. `man-db` renders the packaged githooks(5) page, which the hook-event guard
# test reads when no AsciiDoc source is installed (the Git package ships the page, not the source). The step that
# reinstalls Git when that page is absent repairs the base image's slimmed copy of the package. The remaining packages
# are the tools the tests start (`ps`, `ssh-keygen`, perl, python); the image already carries zlib-ng-compat, which
# provides zlib.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["pacman", "--disable-sandbox", "--sync", "--refresh", "--sysupgrade", "--noconfirm", "--needed", \
     "base-devel", "rust", "git", "cargo-nextest", "nodejs", "pnpm", "strace", "man-db", "perl", "python", "openssh", \
     "procps-ng"]

# The x86-64 CachyOS image is a slimmed copy: its packages lost their manual pages, translations, info pages and docs
# while the image's pacman database still lists every file (`pacman --query --check git` there names 240 missing of the
# package's 757 files, `/usr/share/man/man5/githooks.5.gz` among them). `--needed` reads such a package as up to date
# and never re-extracts the files, so `man` has no page to render and the hook-event guard test has nothing to read. The
# x86-64-v4 image keeps all of them, which is why a build on the resolver's image finds the page without this step.
# Reinstall the package from the synced database when its installed copy lacks the page; the `man --where githooks`
# check still fails without it.
RUN --mount=type=cache,target=/var/cache/pacman/pkg,sharing=locked \
    ["sh", "-c", "test -e /usr/share/man/man5/githooks.5.gz || pacman --disable-sandbox --sync --noconfirm git"]

# What:  the pinned browserslist data, cached where the gate's browserslist rule reads it.
# Why:   `file-enforcer check`'s `browserslist` rule resolves a `.browserslistrc` against two npm tarballs that it fetches
#        into a per-user cache on first use, and a missing tarball is its own offline failure
#        (package/file-enforcer/cli/src/kind_browserslist.rs, `browserslist-fetch-failed`).
#        The tarballs therefore arrive here, where the build may use the network, and never at check time.
#        The versions and the sha512 digests are not typed below: BROWSERSLIST_DATA_LOCK carries the text of the
#        repository's browserslist-data.lock.json, which stays the one source of them, and that text is part of the image
#        tag, so a changed lock builds a new base instead of leaving a cache that no longer matches it.
#        The rule reads `$HOME/.cache/browserslist` and names each tarball after the hex of the digest the lock pins
#        (package/browserslist/core-rust/src/source.rs and src/integrity.rs).
#        Two users run images built on this base: the plan's own gate image as root (`HOME=/root`), and the wrapper's gate
#        image as user 1000 with `HOME=/home/tester`. One verified copy under the unprivileged home serves both, and the
#        root cache is a symlink to it. Every download is refused unless its sha512 equals the lock's, so a tampered lock
#        fails the build here.
ARG BROWSERSLIST_DATA_LOCK
RUN <<'PY'
#!/usr/bin/env python
import base64
import hashlib
import json
import os
import pathlib
import urllib.request

# The npm packages the resolve reads its data out of; the lock names the version and the digest of each.
PACKAGES = ("caniuse-lite", "node-releases")
REGISTRY = "https://registry.npmjs.org"
# Owner of the unprivileged home the wrapper's gate image runs from.
TESTER = 1000
lock_text = os.environ.get("BROWSERSLIST_DATA_LOCK", "")
if not lock_text.strip():
    raise SystemExit("BROWSERSLIST_DATA_LOCK is empty: build with the text of browserslist-data.lock.json as that argument")
lock = json.loads(lock_text)
tester_home = pathlib.Path("/home/tester")
cache = tester_home / ".cache" / "browserslist"
cache.mkdir(parents=True, exist_ok=True)
for directory in (tester_home, cache.parent, cache):
    os.chown(directory, TESTER, TESTER)
for name in PACKAGES:
    version = lock[name]["version"]
    integrity = lock[name]["integrity"]
    expected = base64.b64decode(integrity.removeprefix("sha512-"), validate=True)
    with urllib.request.urlopen(f"{REGISTRY}/{name}/-/{name}-{version}.tgz") as answer:
        tarball = answer.read()
    actual = hashlib.sha512(tarball).digest()
    if actual != expected:
        raise SystemExit(
            f"{name} {version}: the download is sha512-{base64.b64encode(actual).decode()}, the lock pins {integrity}"
        )
    cached = cache / f"{actual.hex()}.tgz"
    cached.write_bytes(tarball)
    os.chown(cached, TESTER, TESTER)
    print(f"{name} {version}: {cached} matches the lock")
root_cache = pathlib.Path("/root/.cache/browserslist")
root_cache.parent.mkdir(parents=True, exist_ok=True)
if not root_cache.exists():
    root_cache.symlink_to(cache, target_is_directory=True)
PY

RUN ["sh", "-c", "git --version && node --version && pnpm --version && strace -V | head --lines=1 && cargo nextest --version | head --lines=1 && man --where githooks"]
