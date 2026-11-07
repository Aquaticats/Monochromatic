# Zig 0.17.1 `zig fetch` and `zig build` could not download packages on this host, which failed `libghostty-vt-sys` and the closure check of `package/terminal/app`

Recorded 2026-11-07T18:40:00-05:00.

## Symptom

`monochromatic-lint package/terminal/app` (the closure check,
which runs `cargo dylint`)
ended with `core/processing-failure`.
The end of cargo's messages was:

```text
pkg/translate-c/build.zig.zon:15:20: error: server connection failed: Unexpected
            .url = "https://deps.files.ghostty.org/translate_c-02ff0c523fb92a38939b04d0145c4a9602a74d56.tar.gz",
thread 'main' panicked at .../libghostty-vt-sys/build.rs:486:5:
zig build failed with status exit status: 1
```

## What was measured

- `curl` fetched the same URL with status 200 from the same shell (155199 bytes).
- `ZIG_GLOBAL_CACHE_DIR=<scratch> zig fetch <that URL>` failed with the same `server connection failed: Unexpected`,
  so the failure is in Zig's own HTTP client,
  not in the build script or the lint.
- `strace` showed Zig reading `/etc/ssl/certs/ca-certificates.crt` and `/etc/resolv.conf`,
  then connecting to the addresses the system resolver returns for `deps.files.ghostty.org`
  (`172.64.80.1` and `2606:4700:130:436c:6f75:6466:6c61:7265`,
  the same ones `curl` uses),
  and both `connect` calls returned `ECONNABORTED`.
- The cause inside Zig (version `0.17.1`,
  kernel `7.2.15`)
  was not determined.
- `zig fetch <local archive>` works and prints the content hash that the `build.zig.zon` file declares,
  but a following `zig build` that finds the stored `p/<hash>.tar.gz` then reports `hash mismatch: manifest declares
  <name>-<version>-<id> but the fetched package has N-V-__...`.
  A directory per package under `zig build --system <dir>` does not hit that.

## Workaround

`mise run //package/terminal/app:fetch:ghostty-offline-inputs` (`package/terminal/app/bin/fetch-ghostty-offline-inputs.ts`)
downloads each archive with Node's `fetch`,
checks its SHA-256 against `package/terminal/app/ghostty-offline-inputs.json`,
cross-checks the content hash with `zig fetch` of the verified local file,
and extracts it to `target/ghostty-offline/packages/<hash>`.
`libghostty-vt-sys` reads that directory through `GHOSTTY_ZIG_SYSTEM_DIR` (`zig build --system`)
and the Ghostty source through `GHOSTTY_SOURCE_DIR`,
so no build downloads while compiling.
The terminal's cargo tasks run the fetch first (a no-op when current),
and the `libghostty-offline-inputs` prerequisite of `monochromatic-lint` hands the same two variables to the closure check.

Removal condition:
a Zig release whose downloader connects here,
checked with `zig fetch <URL>` in a scratch cache.
Even then the checksum pins stay useful:
the build no longer downloads without a checksum of this repository.
