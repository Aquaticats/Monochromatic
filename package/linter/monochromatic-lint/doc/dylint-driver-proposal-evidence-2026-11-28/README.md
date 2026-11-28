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
What is kept is every line a claim rests on.

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
It was run on 2026-11-28 and reported `33 of 33 quotations matched`.
The issue and the CI step are quoted from the captures in `issue/` and `ci/`.
The upstream quotations come from the tag's tarball,
whose excerpts are in `upstream/dylint-v6.4.0-excerpts.txt` with the tarball's sha256.

## The scripts

`scripts/` holds the in-container script of each measurement.
Each was copied into its container and run there;
the host wrapper that started the container is described above and is not kept,
because its paths
name the run's scratch folder.
