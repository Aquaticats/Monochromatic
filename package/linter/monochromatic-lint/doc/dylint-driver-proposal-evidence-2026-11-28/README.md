# Evidence for the Dylint driver proposal of 2026-11-28

What this folder is:
the logs and scripts behind
`doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`.
It is unapproved and non-final;
the owner declares completion.
The proposal cites these files by the paths written under each folder below.
Every extract names the lines it takes and the file they came from.
The full logs of the runs are larger than what is kept here:
the `strace` logs ran to about a megabyte each,
the container images and the driver binaries are not kept,
and the run's own scratch folder expires a day after its last change.
What is kept is every line a claim rests on,
except where the proposal says otherwise:
the first round kept the JSONL records of three skip probes and not their status lines,
kept two skip messages cut at 400 characters,
and kept no cause for its two closure failures.
Those gaps are named in the proposal's own account of M1 to M6,
and the second round re-measured every one of their figures that a claim still needs.

## The machine and the clock

The host facts are in `machine/machine-state.txt`,
read on 2026-11-28 between 03:07:05 and 03:11:10 -05:00,
with the command before each block.
The containers were Linux guests of this host;
their clock is the container clock and is printed at the top of each run log.

## How the containers were made

The image was built once from `docker.io/cachyos/cachyos-v4:latest` with:

```dockerfile
FROM docker.io/cachyos/cachyos-v4:latest
RUN pacman -Sy --noconfirm --needed rustup strace jq binutils pkgconf openssl ca-certificates && pacman -Scc --noconfirm
ENV RUSTUP_HOME=/root/.rustup CARGO_HOME=/root/.cargo
RUN rustup toolchain install nightly --profile minimal --no-self-update && rustup default nightly
RUN rustup toolchain install nightly-2026-11-16 --profile minimal --component rustc-dev --component llvm-tools-preview --component rust-src --no-self-update
RUN curl -fsSL https://github.com/trailofbits/dylint/releases/download/v6.4.0/cargo-dylint-x86_64-unknown-linux-gnu-v6.4.0.tar.gz | tar -xz --strip-components=1 -C /usr/local/bin
RUN curl -fsSL https://github.com/trailofbits/dylint/releases/download/v6.4.0/dylint-link-x86_64-unknown-linux-gnu-v6.4.0.tar.gz | tar -xz --strip-components=1 -C /usr/local/bin
```

That is the floating nightly the image's build day resolved (2026-11-27),
and `nightly-2026-11-16` with the components the lock records,
and `cargo-dylint` and `dylint-link` 6.4.0.
The image tag was `localhost/dylint-probe-base:2026-11-28`.

Each measurement ran one container,
started detached and kept alive for the copied inputs:

```sh
podman run --detach --name <name> --cpus=4 --memory=8g --pids-limit=1024 --blkio-weight=10 \
  localhost/dylint-probe-base:2026-11-28 sleep infinity
```

and the whole host script ran in its own scope:

```sh
systemd-run --user --scope --slice=agent-work.slice \
  --property=MemoryMax=12G --property=CPUQuota=650% --property=TasksMax=4096 \
  --property=IOWeight=1 --property=RuntimeMaxSec=5400 bash ./mN-host.sh
```

Nothing of this account's home was mounted:
the inputs were copied in with `podman cp` (the library,
`package/wg/cli-quicker-exempt`,
the released `monochromatic-lint` 0.5.2-alpha.24,
the host's registry cache and index for M3,
and for M3 the `git archive` of the worktree's `HEAD`).
Every container was removed at the end of its run with `podman rm --force <name>`;
none of this round is left.

## The second round of 2026-11-28

The second gate read of 2026-11-28 asked for two measurements under one instrument and one correction:
option K measured alone and option I measured on a compiler like this host's floating one.
They are here with two smaller ones.
Each folder holds the container's whole output in `run.txt` and its logs in `log/`,
and each script is in `scripts/`.
The image of this round is `localhost/dylint-probe-base:2026-11-28b`,
built from `docker.io/cachyos/cachyos-v4:latest` with `rustup`,
`strace`,
`jq`,
`binutils`,
`git`,
`cargo-dylint` and `dylint-link` 6.4.0,
and the three nightlies the measurements name;
its Containerfile is the one in the first round's section above plus
`nightly-2026-11-17` with `rustc-dev`,
`llvm-tools-preview` and `rust-src`,
and a `git` in the package list.
The linter is the released `monochromatic-lint` 0.5.2-alpha.25 (`sha256` begins `34ca8f26d6`,
printed in each `run.txt`).

- `m7-k-with-minimal-floating-channel/`:
  option K alone,
  with a minimal floating channel and the proxy first for `cargo` and `rustc`.
  `log/a-*.jsonl` and `log/e-*.jsonl` are the rustc-dev skips with the variable unset,
  `log/b-*.jsonl` the driver-gap skip with it set,
  `log/c-warm.*` the 123 s warm build (library 47.13 s,
  driver 1 m 13 s),
  `log/c-execve-counts.txt` the 3 `git` executions,
  `log/c-driver-*` the driver's place,
  version and RUNPATH,
  and `log/d-*.jsonl` the six findings.
- `m8-option-i-on-the-host-like-floating-build/`:
  option I on this host's floating compiler (`5df8c02af`,
  2026-11-16).
  `log/toolchains-*.txt`,
  `log/floating-components.txt` and `log/floating-rustc-version.txt`
  are the state,
  `log/a-*.jsonl` the driver-gap skip,
  `log/b-warm.*` the 91 s warm build (library 46.88 s,
  driver 38.85 s),
  `log/b-execve-counts.txt` the 0 `git` executions,
  `log/c-*.jsonl` the six findings and `log/d-*.jsonl` the `--max-warnings 0` run.
- `m10-skip-statuses/`:
  the three statuses of a skip for both causes,
  with the status lines kept.
- `m11-rustup-does-not-install-an-absent-toolchain/`:
  what `rustup` 1.29.3 does when a directory's `rust-toolchain.toml` names a toolchain it does not
  have.
- `gate-read-second-2026-11-28/`:
  the second same-vendor gate read,
  whole,
  with the probes it ran.
- `gate-read-third-2026-11-28/`:
  the third same-vendor gate read,
  whole,
  with the probes it ran,
  and `findings-answered.md`,
  which answers every finding of all four reads and carries the third and fourth revisions' reports.
- `gate-read-fourth-2026-11-28/`:
  the fourth same-vendor gate read,
  whole,
  with the twelve probes it ran.
- `third-round-2026-11-28/`:
  the third round's measurements and probes,
  with a `README.md` saying which state each arm stands for,
  the `Containerfile` and scripts that made them,
  and the image's digest.

## The measurements

- `m1-f1-nonproxy-cargo/`:
  the machine's state with a non-proxy `cargo` first (F1).
  `run.txt` is the container's whole output;
  `log/linter-skip.jsonl` is the skip the linter printed;
  `log/warm.err.dylint-link-failure.txt` is the library build's failure;
  `log/warm.err.last-6.txt` is the closing error;
  `log/dated-driver.err.last-12.txt` is the temporary-directory refusal;
  `log/execve-counts.txt` counts the programs the run executed.
- `m2a-proxy-alone-fails-at-rustc-dev/`:
  a proxy `cargo` first and the floating nightly still minimal.
  `run.txt` holds the whole run,
  and `linter-skip.jsonl` the skip.
- `m2-f2-proxy-cargo/`:
  a proxy `cargo` first and the components added to the floating nightly (F2,
  F5's dated arm).
  `log/lib-proxy.err.e0603.txt` is the library failure against the 2026-11-27 nightly;
  `log/lib-nonproxy.err.dylint-link.txt` is the same build without the proxy;
  `log/dated-driver.err.last-4.txt` and `log/dated-driver-execve.git-lines.txt` are the dated
  driver build and its clone;
  `log/nonproxy-driver.err.last-8.txt` is the driver build without the proxy.
- `m5-option-k-end-to-end/`:
  the library naming the dated toolchain (option K),
  the skip's statuses,
  and the `nightly` arm of F5.
  `log/skip-default.jsonl`,
  `log/skip-max-warnings.jsonl` and `log/skip-quiet.jsonl` are the three
  probes;
  `log/nightly-driver.err.tail.txt` and `log/execve-counts.txt` are the `nightly` arm;
  `log/k-warm.err.building-driver.txt` and `log/k-warm.err.finished.txt` name the toolchain Dylint
  chose and the build times;
  `log/k-driver-tree.txt`,
  `log/k-driver-version.txt` and `log/k-driver-readelf.txt` are the driver's
  place,
  version and dynamic section;
  `log/k-linter.jsonl` is the six findings;
  `log/no-file-linter.jsonl` is the skip with the original library;
  `library-rust-toolchain.toml` is the file option K adds.
- `m6-clean-driver-and-probe/`:
  a driver built without Dylint's rpath flag,
  and the probe once its offline resolution succeeds.
  `log/clean.err.tail.txt`,
  `log/clean-driver-readelf.txt` and `run.txt` are the first;
  `log/probe-fetch.err`,
  `log/probe-metadata.err` and `log/probe-vs-run.jsonl` are the second.
- `m4-f10-rpath-and-other-home/`:
  the driver's RUNPATH and its load where the builder's path is gone (F10).
  `log/driver-rpath-and-needed.txt`,
  `log/control-rpath-present.txt`,
  `log/other-path-run.txt` and `log/other-path-with-ld-library-path.txt` are the four steps;
  `log/install-opt-rustup.txt` is the second toolchain's install.
- `m3-f3-skips-with-a-driver-1/` and `m3-f3-skips-with-a-driver-2/`:
  the thirty selectable lock directories,
  fifteen per container,
  with a current driver,
  one linter invocation each (F3).
  `log/results.tsv` has one row per directory:
  path,
  status,
  seconds,
  offline-metadata result,
  skip records.
  `log/closure-skips.txt` collects every skip message,
  and `log/closure-failures.txt` every closure processing failure.

## The quotations

`scripts/quotations.tsv` lists every quotation the proposal uses with the file it comes from,
and `scripts/check-quotations.ts` checks each one after normalising whitespace.
It was last run on 2026-11-28 by the third revision and reported `92 of 92 quotations matched`.
Two rows that quoted `doc/planning/github-hosted-runners-for-package-checks-and-a-guest.md`
were removed by that revision,
because the sentences they quoted are no longer in that record and the
revision's text does not quote it.
The issue and the CI step are quoted from the captures in `issue/` and `ci/`.
The upstream quotations come from the tag's tarball,
whose excerpts are in `upstream/dylint-v6.4.0-excerpts.txt` with the tarball's sha256.

## The scripts

`scripts/` holds the in-container script of each measurement of the first two rounds.
Each was copied into its container and run there;
the host wrapper that started the container is described above and is not kept,
because its paths
name the run's scratch folder.

The third round keeps its host scripts as well,
in `third-round-2026-11-28/scripts/`,
because they write the linter configuration and copy the toolchain file the K1 measurement needs,
and a rerun needs those two files to be the same bytes.
