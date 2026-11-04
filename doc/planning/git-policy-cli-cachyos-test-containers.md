# Git policy wrapper test containers on CachyOS latest

Status:
 implemented on branch `build/git-policy-cli-cachyos-test-containers`; the tool picks were then replaced by the owner's
 decisions of 2026-11-04 (section "Owner decisions of 2026-11-04", on branch
 `feat/git-policy-cli-test-images-from-pacman`).
Where a bullet of "Picks" below names a source build, a checksum or a version pin, the owner decisions supersede it.

## Why

The gate image (`native:test:container`) started from a content-addressed local image of Debian trixie
 (the official Rust image plus a Git 2.56.0 built on top),
 and the end-to-end image (`test:e2e:concurrent:image`) started from `docker.io/library/node:24.11.0-slim` (Debian)
 with the compile stage in `docker.io/library/rust:1.97-bookworm`.
`doc/decision/container-base-distro.md` makes every image that builds a tool, test or development environment start from
 the newest CachyOS image the host CPU runs, on its `latest` tag, with no version tag and no digest.
None of its exceptions apply here:
 these are not release images (nothing shipped is built in them),
 no tool is missing from the CachyOS repositories (`strace` and `cargo-nextest` were already built or downloaded by
 checksum, and Rust, the compiler and `ssh-keygen` are packaged),
 and no binary built in the images runs on the host.

## The conflict and the options

The decision forbids pinning, and the gate image builds without network from a content-addressed base.
A pin by digest satisfies the second and violates the first;
 a `FROM <latest>` with network satisfies the first and loses the second.

Option A: resolve `latest` once per run, derive a base keyed on what the pull returned.
Each run pulls the tag with `podman pull --policy=newer`,
 names a derived base `<first digits of the pulled image ID>-<first digits of the Containerfile hash>`,
 builds it with network only when that tag is missing,
 and builds the gate image from the derived base's ID with no network, as before.
Pros:
 follows the rule literally (nothing is pinned, a CachyOS republish is picked up on the next run),
 keeps every gate layer offline and cached on the base ID,
 and a run that finds its base spends one registry request.
Cons:
 a CachyOS republish rebuilds the derived base and every gate layer above it once,
 and the first run on a host needs the network.

Option B: `FROM` the CachyOS image inside the gate Containerfile and install packages there with network.
Pros:
 one Containerfile, no derived image.
Cons:
 every layer that follows is keyed on a base that the pull may have changed,
 the compile layers lose the no-network guarantee that proves the vendored sources are complete,
 and the package step runs wherever the cache key moves.

Option C: pin the base by digest or version.
Pros:
 fully reproducible and offline.
Cons:
 forbidden by the owner ("Must be latest, since if an upstream OS update breaks us, we are doing something wrong and we'd
 have to fix it immediately.").

Ranking: A > B > C.
A beats B because it keeps the gate image's offline, cached build and costs the same network use as B only when CachyOS
 changed.
B beats C because C is excluded by the owner's directive while B only costs speed.

## Picks (agent's pick, veto open)

- Option A, with the derived-base tag as above (`localhost/git-policy-native-test-base` for the gate,
  `localhost/cli-git-concurrent-e2e` for the end-to-end suite).
- A registry that cannot be reached is reported and the run continues on the CachyOS copy the host holds,
  because a gate that cannot start offline teaches people to stop running it;
  with no local copy the run fails.
  This is the one pick that brushes against "must be latest":
  the other choice is to fail the run whenever the pull fails.
- The gate keeps Git 2.56.0 built from the kernel.org tarball (checksum pinned) at `/usr/bin/git`,
  after removing the CachyOS Git package from the base,
  because the gate's results are evidence about that release and the tests name `/usr/bin/git`.
  The CachyOS Git package is also 2.56.0 today but moves with each pacman sync.
  Ranking: pinned release > package Git, because a gate whose Git moves under it cannot say which Git it verified.
- The end-to-end executable is compiled in the gate's base image (same CachyOS image, Rust included) instead of a second
  Rust image, so one pull gives both stages the same C library.
- Node 24.11.0 for the end-to-end suite comes from the official tarball (checksum pinned), not the packaged Node,
  because the suite has always run on that version.
- Rust, Cargo and Clippy in the images are the CachyOS packages (the Debian images carried 1.97.x),
  so the gate compiles with whatever stable release CachyOS ships.
- `strace` stays at the pinned source release but builds with its bundled kernel headers and without `-Werror`,
  because the newest `linux-api-headers` and compiler stop the unmodified build.
  This is the cost of `latest` the decision names: the build broke on an upstream update and was fixed here.

## Owner decisions of 2026-11-04

Asked to veto the picks above (Git 2.56.0 built from source in the gate image, Rust from pacman, Node 24.11.0 from the
upstream tarball, `strace` 6.13 downloaded and checked by checksum), the owner answered "Make as many pkgs as possible
come from podman." and, asked which was meant, "Sorry, typo. Yes I meant from pacman." Then, in order:

- "We don't need to pin git because my current system is already using git latest."
- "We don't need to pin Node too."
- Asked where the test images should get the tools `toolchain.lock.json` locks (Node, cargo-nextest, Rust, pnpm),
  against `AGENTS.md` rule CM7, the owner chose "Pacman for everything it has": Node, cargo-nextest and Rust from pacman's
  current packages, and pnpm from the lock only if pacman lacks it.
  This is an exception to CM7 for these test images.
- Asked about Git 2.55.0 in the end-to-end image (the oldest Git the wrapper accepted; it refused older Git):
  "Raise minimum, drop the test, merely print a warning on older git and not refuse."

What changed, each with its evidence (probed 2026-11-04 in `docker.io/cachyos/cachyos-v4:latest` after `pacman -Sy`):

- Git: the gate and end-to-end images install pacman's current `git`; no source build, no checksum.
  `pacman -Si git` listed `git 2.56.0-1.1` (repository `cachyos-extra-v4`) and `git 2.56.0-1` (`extra`); the image
  installed `git 2.56.0-1.1`.
  The host's `/usr/bin/git --version` printed `git version 2.56.0` on the same day.
  No exact-version route (an Arch Linux Archive package, a dated snapshot) is used.
- The Git source tree at `/work/git-source` is gone.
  The package ships the rendered page `/usr/share/man/man5/githooks.5.gz` and no `githooks.adoc`, so the hook-event guard
  test (`invocation::hooks_off::tests::every_documented_hook_event_is_listed`) reads the page through `man`, which the
  image gets from the `man-db` package.
- Node: `nodejs` from pacman (26.12.1-2 from `cachyos-v4` when probed; `extra` had 26.13.0-1, and the host runs
  v26.13.0), instead of the 24.11.0 tarball.
  The suite is TypeScript run by Node type stripping, and nothing in the package reads the Node version.
  `nodejs-lts-krypton` (24.22.2-1 in `extra`) exists if the owner wants the 24 line.
- pnpm: pacman carries it (`pnpm 12.17.0-1.1`, `cachyos-extra-v4`; the lock row records 12.16.1), so the gate image no
  longer extracts the lock row's archive and `bin/native-pnpm-snapshot.ts` is deleted.
- cargo-nextest: pacman carries it (`cargo-nextest 0.9.150-1.1`, `cachyos-extra-v4`; the former pin was 0.9.148 by
  checksum).
  `bin/native-nextest-snapshot.ts` stays only because `package/git-policy/cli.fuzz` and `package/linter/monochromatic-lint`
  (and its fuzz sidecar) still import it for their own images.
- strace: pacman's `strace` (7.3-1 in `extra`) replaces the statically linked 6.13 build; the gate image no longer needs
  a build stage, a source archive or `--enable-bundled`/`--disable-gcc-Werror`, and `bin/native-strace-snapshot.ts` is
  deleted.
  The Arch Linux Archive lists `strace-6.13-1-x86_64.pkg.tar.zst` should the owner ever want 6.13 back through pacman.
- Rust: unchanged, pacman's `rust` (1:1.99.1-1 in `extra` when probed).
- End-to-end Git: the 2.55.0 source build and its scenario list are gone; the suite runs the image's `/usr/bin/git` and
  reads its release at run time.
  The wrapper no longer refuses an older Git; see `package/git-policy/cli/SPEC.md`, "Compatibility and degradation".
  Its `MINIMUM_GIT_VERSION` is 2.56.0, the release the gate runs with, and is raised by hand with the image's Git.
- Signatures: the CachyOS image's `/etc/pacman.conf` sets `SigLevel = Required DatabaseOptional`, and the Containerfiles
  never lower it; `pacman --sync` downloaded `git-2.56.0-1.1-x86_64_v4.pkg.tar.zst` with its `.sig` beside it.
  Reproducibility is the cost: the tools move with each CachyOS republish or pacman sync, which the owner accepted by
  rejecting pins ("if an upstream OS update breaks us, we are doing something wrong and we'd have to fix it immediately").
  `mise run //package/git-policy/cli:test:git-image` prints the release of each tool the base image got.

## Left open

- `package/git-policy/cli.fuzz/bin/fuzz-images.ts` (base `62ba2f7ce22b...`, Debian Rust image),
  `package/forbidden-strings/cli/bin/snapshot-constants.ts` (base `6ec87f6d290a...`, the old gate base) and
  `package/linter/monochromatic-lint/test.Containerfile` (base `84f24e75017a...`) still start from Debian images.
  They are other packages, so this change does not touch them.
- The CachyOS Git package and the derived gate base could serve those packages too once their owners move them.
- Flagged for the owner, veto open: package state drifts between CachyOS republishes.
  Each derived base runs a full `pacman` system upgrade when it is built,
  and it is rebuilt only when the CachyOS image or the Containerfile changes.
  The packages inside a cached base are therefore the ones that were newest when it was built,
  not the ones newest today, until the next republish or Containerfile edit.
  The end-to-end image check looks for `not found` lines of `ldd`,
  so a compile base rebuilt later than the end-to-end base cannot silently link against a newer C library.
  Forcing a fresh system upgrade on every run would restore "newest" literally and cost the cached gate layers each time.
- `strace` stays at 6.13 by checksum while upstream has released 7.3;
  it is a fixed test tool, not a base image, so the base rule does not move it.

## Measured on 2026-11-04 (this host under its ordinary load, two-CPU containers)

Gate image build with the cache switched off, alternating old and new:
 old 293.4 s and 260.6 s, new 276.3 s and 267.4 s (the spread of the old pair is larger than the difference).
Gate image build with the cache warm: old 27.0 s, new 11.2 s plus 2.0 s for the pull check and base lookup.
New base image, built once per CachyOS republish or Containerfile edit: 107 s at two CPUs, with the package cache warm.
Gate test run: old control 451.8 s (earlier gates of this size: 442 s to 460 s); new 458.4 s when the host was otherwise
 quiet and 511.0 s while lint and type checks ran beside it, so the new image adds no time that the load did not explain.
Both gates ran 4707 tests with 7 skipped, and the lists of passing test names are identical.
End-to-end suite: old and new both run the same 38 scenario runs, with the same one failure
 (`sigkill-service-during-work`, the service settles no push records within 30 s) on the old and the new containers,
 so that failure is not caused by the base image.
Its image build: compile stage 284 s old, 277 s new; suite run 47 s old, 48 s new; base cold build 131 s old, 94 s new.
