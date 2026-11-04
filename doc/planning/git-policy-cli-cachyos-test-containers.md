# Git policy wrapper test containers on CachyOS latest

Status:
 implemented on branch `build/git-policy-cli-cachyos-test-containers`,
 every pick below is the agent's pick with the veto open.

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
