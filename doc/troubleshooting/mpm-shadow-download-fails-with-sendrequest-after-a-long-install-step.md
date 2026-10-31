# `mpm-shadow install` fails a download with `client error (SendRequest)` after a long install step

## Symptom

Observed 2026-10-31 in a `podman build` of the repository `Dockerfile`
(`mpm-shadow install --frozen-lockfile`,
64 tools).
Two archive tools failed in every run,
the others installed:

```text
toolchain install: mpm-shadow failed: download failed: https://github.com/Aquaticats/Monochromatic/releases/download/monochromatic-mpm-shadow-v0.2.0/monochromatic-mpm-shadow-0.2.0-x86_64-unknown-linux-gnu.tar.gz: the request failed: client error (SendRequest)
toolchain install: llama.cpp failed: download failed: https://github.com/ggml-org/llama.cpp/releases/download/b9829/llama-b9829-bin-ubuntu-x64.tar.gz: the request failed: client error (SendRequest)
```

The same two downloads succeed when `mpm-shadow install` is given a plan that holds only those tools,
with or without `GITHUB_TOKEN`.

## Cause

Not isolated to a host or a token:
a plan of three tools reproduces it in a clean image.
In order,
`bin` (an archive download from github.com),
`rust` (a `rustup toolchain install` with five components,
about 30 seconds),
`age` (an archive download from github.com):
`age` fails with the message above on `mpm-shadow` 0.2.0.
The same plan with a bare `rust` entry (a gap of about 20 seconds) installs all three.

`HyperClient` (`package/mpm-shadow/cli/src/toolchain/http.rs`) owns a current-thread `tokio` runtime that runs only
while a call blocks on it,
and `hyper-util`'s client keeps finished HTTP/1.1 connections in a pool.
While `cargo binstall`,
`rustup` or `uv` runs,
the runtime is not polled,
so the connection task never sees the
server close the idle connection.
The next request takes that pooled connection and fails to send.
Whether a download fails therefore depends on how long the install step before it took.

## Fix

`HyperClient::new` builds the client with `pool_max_idle_per_host(0)`,
so no connection is kept for reuse;
each request opens its own (one more TLS handshake per request).
The dev build ran the same three-tool plan in the same image:
3 installed,
0 failed.
The unit tests cannot cover it,
because the client is HTTPS-only and no loopback TLS server exists in the tests;
the container reproduction above is the check.
Released builds up to 0.2.0 have the defect,
and a lock that names one of them as its `mpm-shadow` bootstrap runs it
until a release with the fix is locked.
