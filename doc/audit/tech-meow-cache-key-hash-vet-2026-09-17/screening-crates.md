# Screening appendix: every crates.io result

Appendix to [`tech-meow-cache-key-hash-vet-2026-09-17.md`](../tech-meow-cache-key-hash-vet-2026-09-17.md),
 section "Screening".
One entry per distinct crate returned by the crates.io queries CR01 to CR14 and XR01 to XR04,
 at its newest stable version on 2026-09-17.

How outcomes were assigned:

- Reviewed crates carry the outcome written in the report's candidate ledger.
- Crates with matches for both architectures or a SIMD abstraction,
   and not reviewed as hash libraries,
   were read by description and source layout and are category mismatches.
- Every other crate had intrinsic matches for at most one architecture
   and no SIMD abstraction in its non-test Rust sources,
   which fails HC1 regardless of category.

- `a1-ai` 2.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `a3s-box-mkext4` 3.2.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `a3s-power` 0.9.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `a3s-use` 0.3.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `a3s-vec` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aa_fastlink` 0.7.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aardvark-core` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aaron` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aaron-admin` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aaron-shard` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aatp` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ab-blake3` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `abao` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `abir_guard` 3.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `abt` 1.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `abundantis` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `accelerated-crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `accroitre` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `accroitre-cli` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `acdi` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `acdp-crypto` 0.13.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `acorde-analysis` 1.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `acronym-engine` 0.6.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `action_maps` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `acton-service` 0.43.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `adaptive-pipeline-domain` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `adler` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `adler2` 2.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `adler32` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `adler32-simd` 0.1.0:
   HC2:
  32-bit checksum.
- `aegis-common` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aegis-file` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aegis-ledger` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aerovault` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aes256ctr_poly1305aes` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aex` 0.1.21:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `affidavit` 26.6.22:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `agent-spreadsheet-render` 0.16.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `agent-store` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `agentic-data` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `agentroot` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `agentroot-cli` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `agk` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ahash` 0.8.12:
   HC2 (64-bit `Hasher` output) and HC3 (output documented as not stable across versions and platforms).
- `ahash_macro` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ahsah` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ai-audit` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ai2070-net` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aion_verify` 3.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `airframe_crypt` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `airlock-gateway` 2.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `AitSar` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ajour` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `akas` 2.4.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `algos` 0.6.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `algox` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `alice-analytics` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `alice-crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `alice-physics` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `alice-sync` 0.6.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `allthehashes` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `alphawinnow` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `alterion-encrypt` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `altsearch` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `am-fs-btrfs` 0.6.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `am-fs-erofs` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `am-fs-ext4` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `am-fs-xfs` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `amadeus-utils` 1.3.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `amaters-net` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `anchorhash` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `anchormap` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `anewer` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `angulu` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `angulu-rs` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `animsmith-fbx` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ann-search-rs` 0.8.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `anomstream` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `anomstream-core` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `anomstream-hotpath` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `anomstream-triage` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `ant-merkle` 1.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ant-quic` 0.27.52:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `antneuro` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `anubis-vault` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `anvil-ssh` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `any-base` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `any-gpu` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `anyhash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ap-noise` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aperion-shield` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `apex-sdk-types` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aphelion-core` 1.2.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `appcore-filemaker` 0.1.0-beta.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `apple-cryptokit-rs` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aprender-registry` 0.67.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aprender-test-js-gen` 0.67.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aprender-verify` 0.67.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aptos-bcs` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aptu-coder-core` 0.34.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arai` 1.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbitrary-type` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-core` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-graph` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-graph-cli` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-gui` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-mcp` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-server` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arbor-watcher` 2.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arcanum-hash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `archimedes-kernel` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `archivum` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arcthis` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arctic-map` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arctic-wt` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arcweight` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argon2-kdf` 1.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argon2-rust` 1.1.0:
   category mismatch:
  password hashing.
- `argon2rs` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argonautica` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argone-signing` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argus-guard` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argus-lens` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argus-llm` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `argus-verify` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arity-arrays` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ark-poly-commit` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ark-transcript` 0.0.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arkeion` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arkhe-kernel` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arkhe-rand` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arloader` 0.1.63:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arrdb-buffer` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arrow-digest` 59.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arrow-view-state` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arsenal-crypto` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `arthash` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aruco-rs` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `asap_sketchlib` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `asbytes` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ascon-hash` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `asmcrypto` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ast-bro` 4.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `astenn` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `astrid-crypto` 2026.9.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `astrograph-bin` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `astrs-data` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `astrs-wire` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `asupersync` 0.5.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `async-hash` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ate` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `athena-auth` 1.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `atlas-blake3-hasher` 3.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `atlas-entry` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `atlas-nohash-hasher` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `atomic-blob-store` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `atomic-cuckoo-filter` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `atomwrite` 0.1.36:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `audiofp` 0.4.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `audit-trail` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `auger` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authforge` 1.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authx-axum` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authx-cli` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authx-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authx-dashboard` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authx-plugins` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `authx-storage` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `autobahn-hash` 0.1.0:
   screening survivor,
  then HC4 at targeted evidence:
  HighwayHash through nightly `portable_simd`;
   fails to compile on nightly-2026-09-12.
- `automake-oracle-rs` 0.1.23:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `autoscale_cuckoo_filter` 0.5.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `autozig` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `avdumpr` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `avila-atom` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `avila-cli` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `avila-compress` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `awesome-rust` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `awkrs` 0.5.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `awq` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ax-cache` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ax-sym` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `axhash` 1.0.0:
   HC2:
  64-bit output (`axhash-core`).
- `axhash-core` 1.0.0:
   HC2:
  `axhash` returns u64 (src/hasher/api.rs:9).
- `axhash-dashmap` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `axhash-indexmap` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `axhash-map` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `axiom-query` 2.0.17:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aya_poker` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `aydee` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `azathoth-core` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `b3sum` 1.8.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `babbel_json` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `backbone-ml-dsa` 0.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `backbone-sphincs` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `backup-deduplicator` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bamboo-agent` 2026.9.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bamboo-cli` 0.5.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bamboo-ssg` 0.5.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `banuid` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bao` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bao_bin` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bao-tree` 0.16.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bar-store` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `baracuda` 0.0.1-alpha.79:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `base-d` 3.0.36:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `basecrawl` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basecrawl-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basecrawl-ffi` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basecrawl-fp` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basecrawl-proof` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basecrawl-render` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basecrawl-seal` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basis` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basis-acp` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basis-cli` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basis-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basis-host` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `basis-tasks` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bastion-ai` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bathroom` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `batpak` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bc-components` 0.31.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcinr-powl` 26.7.28:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcinr-powl-receipt` 26.7.28:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcmr` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcrypt` 0.19.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcs` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcs-cli` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcs-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bcs-link` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bee-tui` 1.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `beekem` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bellbook` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `belt-hash` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `better-blockmap` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `better-hex` 1.0.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `betterFANN` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bfield` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bhc-data-structures` 0.2.22:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bhed` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bidimap` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bidirectional_hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `big_space` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bigfiles` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bigsig` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bijux-gnss-infra` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bilrost` 0.1016.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `binary-ensemble` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bincode_derive-next` 3.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bincode-next` 3.1.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `binconf` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bindashtree` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bip_dht` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bip39_mnemonic_generator` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bishop` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bishop-cli` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bit-string` 0.6.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `bitbelay` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitbelay-cli` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitbelay-providers` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitbelay-report` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitbelay-statistics` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitbelay-suites` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitbelay-tests` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin` 0.32.102:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin_hashes` 1.2.0:
   outside the decision scope (cryptographic).
- `bitcoin-blockfilter` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-blockman` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-bloom` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-chainman` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-crc32c` 0.1.18:
   HC2:
  32-bit CRC.
- `bitcoin-crypter` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-golombrice` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-hash` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-hdchain` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-index` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-key` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-message` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-muhash` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-p2tsh-pqc` 0.32.6-p2tsh-pqc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-poly1305` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-random` 0.1.21:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-secp256k1` 0.1.16-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-sha256` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-sha3` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-sha512` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-siphash` 0.1.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoin-tools` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoincash` 0.32.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoinleveldb-compat` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoinleveldb-crc32` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoinleveldb-hash` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoinleveldb-logreader` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoinleveldb-logtools` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcoinleveldb-logwriter` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitcut` 1.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `bitdroid` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitende` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitfields` 3.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitfields-impl` 3.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitnuc` 0.5.4:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `bitrep` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bitsliced-op` 0.8.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `bittoku-bsv` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake` 2.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake-hash` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2_ce` 0.10.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2-rfc` 0.2.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2-rfc_bellman_edition` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2b_simd` 1.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2b-192` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2b-rs` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2s_const` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake2s_simd` 1.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3` 1.8.7:
   outside the decision scope (cryptographic);
   measured as the requested control;
   aarch64 NEON only through C (HC4 without C).
- `blake3_aead` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3_cipher` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3_enc` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3_guts` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3-lamport-signatures` 0.3.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3-pow` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blake3-std` 0.1.0:
   outside the decision scope (cryptographic);
   BLAKE3 through nightly `std::simd`,
   last release 2023.
- `blake512-hash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blakeout` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blasthttp` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blazehash` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blazehash-core` 0.2.5:
   category mismatch:
  file-hashing tool engine over other hash crates.
- `ble-data-struct` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blewm` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blockhash` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blockset` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloclawd` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloclawd-pow` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloclawd-schema` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloom_filter_simple` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloom-lib` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloomcraft` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bloomz` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bls12_381` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blumer` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blunders` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `blut` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bmrc` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bmw-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bn254` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-acme` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-cloudflare` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-cluster` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-container` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-core` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-docker` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-firecracker` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-handlers` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-mcp` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-node` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-rpktls` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-server` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-storage` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-types` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boatramp-vz` 0.4.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boomphf` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boreal` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boringauth` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bosshogg` 2026.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `botan-crypto-provider` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `boundary-compiler` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `box2d-rust` 1.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `bpht` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bqc` 0.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bqtools` 0.5.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `brec` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `brine-ed25519` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `browser-automation-cli` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `brutecraber` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `brynja` 0.20.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `brynja-core` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `brz-metrics` 0.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsql-arena` 0.27.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsql-core` 0.27.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsql-driver-postgres` 0.27.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsql-driver-sqlite` 0.27.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsql-macros` 0.27.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bssh` 3.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bstack` 0.4.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsv` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsv-sdk` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsv-storage-cloudflare` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsv-wasm` 1.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bsv58` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bte` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-core` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-disk` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-forensic` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-fs` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-mkfs` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-stream` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `btrfs-transaction` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bufhash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bugu` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bun_hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bun_wyhash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `bunsan` 1.3.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `busywork` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `buthash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `buup` 0.25.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `by_address` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `byteforge` 0.1.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `bzr` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `c2pa-html` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `c2pa-text-binding` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cached` 4.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cachedhash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cachekit-core` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cachesim-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cacrt` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cadi-core` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `Cadrs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cairn-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `caixa-core` 0.1.558:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `caixa-helm` 0.1.558:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `calabi-bcs` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `camus` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `canton-reserve-attest` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `canton-solvency-merkle` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `canton-solvency-report` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `canton-solvency-verify` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `captrack` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `captrack-macros` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `captrack-pgo` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `capycrypt` 0.7.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `car-memgine` 0.54.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `carbonado` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `card_catalog` 1.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cardinal-harness` 0.11.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cardinality-estimator-safe` 4.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-affected` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-bay` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-chronoscope` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-compete` 0.10.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-dry4rust` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-dupes` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-evidence` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-featalign` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-hold` 1.3.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-impact` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-maintained` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-mark-sweep` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-orphan-gc` 0.8.28:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-pgrx` 0.19.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-pgx` 0.7.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-reclaim` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cargo-sift` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cas-kit` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cascade-rhythm` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `casefold` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `casial-core` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `casper-vdb` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `casq` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `casq_core` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cavalier_contours` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cavs-hash` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cb2vec` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cc-lb-plugin-wire` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ccsum` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cdc` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cdx-core` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cedar-local-agent` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cellos-sink-spool` 0.6.0-pre:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cera-ffi` 0.5.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cert-dump` 3.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cesride` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cf-gears-rustls-corecrypto-provider` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cf-gears-toolkit-stable-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cf1-rs` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cgl-rs` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chacha20-blake3` 0.10.0:
   outside the decision scope (cryptographic AEAD).
- `chacha20poly1305` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chainfold` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `champ-trie` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chaos` 0.9.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chaotic_semantic_memory` 0.3.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `charter` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `checksum` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `checksum-tapestry` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `checksums` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chess` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chibihash` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chicago-tdd-tools` 26.8.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chie-crypto` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chisel-storage` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-core` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-md5` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-sha1` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-sha2` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-sha2-224` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-sha2-256` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-sha2-384` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-hash-sha2-512` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-md5` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-sha1` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-sha2` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-sha2-224` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-sha2-256` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-sha2-384` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chksum-sha2-512` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chordrift` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chromahash` 0.7.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `chromaprint-next` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `chronix` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chronix-analytics` 0.7.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `chronix-core` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chronix-encoding` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chronix-engine` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chronix-query` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chronix-security` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chronix-streaming` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chrono-merkle` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chunk-diff` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chunkrs` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `chunkshop-rs` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ciftl-rs` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ciphern` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `circom-witness-rs` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cityhash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cityhash-rs` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cityhash-sys` 1.0.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cityhasher` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cjc-lang` 0.1.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ck-meow` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ckb-rich-indexer` 1.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ckey` 0.4.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ckg` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `claim-ledger` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clark-hash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `classeve-rai-compress` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `classeve-rai-infer` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `claude_storage` 1.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clhash_rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clhash-sys` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cliff3-util` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cljrs-blake3` 0.2.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cljrs-runtime` 0.2.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clmm-swap-math` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clock-curve-math` 1.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clock-hash` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clock-rand` 1.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clone-solana-blake3-hasher` 2.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cloud-sdk` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cloudmapper` 0.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `clubcard` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cmn-substrate` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cmpf` 5.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cmtn` 1.2.5-alpha:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cnc-formats` 0.1.0-alpha.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cnfy-uint` 0.3.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cobre-stochastic` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cocoonfs-cli` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `code-context` 0.22.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `code-dupes` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `code-graph-cli` 3.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codecache-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codeskeleton` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codestyle` 0.2.81:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codetether-agent` 4.7.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codetree` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codeunlimited` 2.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codewandler-flux-lang` 0.59.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codex-memory` 3.0.15:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `codex-patcher` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `coding-agent-search` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cognitum-gate-kernel` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cognitum-gate-tilezero` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `colgrep` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `coll-birth` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `collect-with` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `collet` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `common_macros` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `common_traits` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `commoncrypto` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `commoncrypto-sys` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `community-id` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `compact-dict` 0.1.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `conf-hub` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `conhash` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `consistent_hash` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `consistent_hash_ring` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `consistent-choose-k` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `const_blake3` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `const-sha1` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `const-siphasher` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `constmap` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `constraint-theory-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `content-addressable` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `continuity-engine` 0.2.40:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `coordinode-lsm-tree` 5.8.6:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `copc_converter` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `copia` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `copybook-determinism` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cord` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cord-rs` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `corium-log` 0.1.90:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cornerstore` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cors-core` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cortiq-cli` 0.6.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cortiq-core` 0.6.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cortiq-engine` 0.6.8:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `cortiq-net` 0.6.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cortiq-server` 0.6.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmian_crypto_base` 2.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-alignment` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-batch` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-bio` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-conformer` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-cx` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-depict` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-descriptors` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-fingerprints` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-forcefields` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-io` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-macros` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-model` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-search` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-smiles` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-stereo` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-tautomer` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmolkit-types` 0.5.0-rc.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cosmostrix` 100.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `count-min-sketch-rs` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `courierust` 1.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cow_hashbrown` 0.14.21:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cow_hashmap` 0.1.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cow-sdk-browser-wallet` 0.1.0-alpha.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cozy-chess` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cp2` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crabgraph` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crabguard` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crabrl` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crafter` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crate2nix` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cratestack-exec` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc` 3.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc_all` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc-32c` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc-adler` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc-any` 3.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc-fast` 1.10.0:
   HC2:
  CRC-16,
  CRC-32,
  and CRC-64 outputs.
- `crc10-atm-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc10-cdma2000-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc10-gsm-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc11-flexray-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc11-umts-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-cdma2000-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-cms-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-dds110-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-dectr-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-dectx-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-en13757-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-genibus-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-gsm-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-ibm3740-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-m17-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-opensafetya-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-opensafetyb-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-profibus-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-spifujitsu-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-t10dif-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-teledisk-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-umts-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc16-xmodem-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-flexraya-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-flexrayb-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-interlaken-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-ltea-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-lteb-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-openpgp-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc24-os9-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32_light` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32-aixm-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32-bzip2-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32-cksum-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32-mpeg2-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32-v2` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32-xfer-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32c` 0.6.8:
   HC2:
  32-bit CRC.
- `crc32c-cli` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32c-fast` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32c-hw` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32c-sse42` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32csum` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc32fast` 1.5.2:
   HC2:
  32-bit CRC.
- `crc32fast-lib` 1.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc64` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc64-rs` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `crc64fast` 1.1.0:
   HC2:
  64-bit CRC.
- `crc64fast-nvme` 1.2.1:
   HC2:
  64-bit CRC.
- `crc8` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-autosar-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-cdma2000-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-dvbs2-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-gsma-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-gsmb-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-hitag-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-i4321-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-icode-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-lte-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-mifaremad-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-nrsc5-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-opensafety-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crc8-smbus-fast` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crcdir` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crcxx` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crev-recursive-digest` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crmprs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crmux` 0.20.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crrl` 0.9.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `crypt-io` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypta` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypticy` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptkit` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypto` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypto_lib` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypto_proto` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `crypto_secretbox` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypto-async-rs` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypto-hash` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `crypto-hashes` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptocol` 0.19.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptography-rs` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `cryptokit-rs` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptolib` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptonight-hash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptox` 1.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cryptr` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cshake` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `css-cat` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `csum` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `csv_lib` 1.0.7:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `csv-adapter-core` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `csv-diff` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ct-merkle` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ctfs` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cttps` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cu-hc12` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cu29-logstream` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cu29-logstream-serial` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cubehash` 0.4.1:
   outside the decision scope (cryptographic).
- `cuckoofilter` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cuda-rust-wasm` 0.1.7:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `cudaforge` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cuid2-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `curve25519-parser` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `custodian-kernel` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cvkg-layout` 0.3.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cvm` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cyb-jali` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `cyb-kuro` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `cyb-lens` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `cyb-lens-core` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `cyb-nebu` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `cyber-lens` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cyber-lens-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cyber-strata` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `cyfs-sha2` 0.8.4:
   outside the decision scope (cryptographic).
- `cyphera` 0.0.1-alpha.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dagex` 2026.23.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dalo` 0.15.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dandelion-random` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dartminhash` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dashmap-shard` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `data-racer` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `databoxer` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `datapipe-cli` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `datawal` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `daybook` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dce-router` 1.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dcrypt-algorithms` 4.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ddiff` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `deacon` 0.17.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `debrepo` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `decanter` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `decanter-crypto` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `decanter-derive` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `decorum` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dedcore` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dedup-cli` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `deduplicator` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dedups` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `defendor` 0.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dek` 0.1.40:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dekopon-brokerd` 0.17.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `delta-struct` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `delta-struct-macros` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `deputy-crypto` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `derec-cryptography` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `derec-library` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `derive_hash_fast` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `descriptor-codec` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `descriptor-encrypt` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `destructive_command_guard` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `deterministic-hash` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `device-fingerprint` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dgsp` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dhad` 1.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `diapause` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `diff_match_patch` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `differential-engine` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `differential-symbols` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `diffguard-analytics` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dig-wallet` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `digest` 0.11.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `digest-hash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `digestify` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dilithia-stark` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dilithium-rs` 0.4.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `dircs` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dirhash_fast` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `djangohashers` 1.8.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `djb_hash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `djpass` 1.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dms-rs` 0.5.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `doe` 1.1.90:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `doldskrift` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `domain-key` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dome-ward` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `donadb-x` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dotling` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dotslash` 0.5.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `doubleentry` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `downloader-rs` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `doze_common` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dpsi` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dream_archivetool` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-analytics` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-attention` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-audio` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-chronicle` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-cli` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-engine` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-fabric` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-gates` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-gpu` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-intelligence` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-launcher` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-loom` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-math` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-matter` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-media` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-mesh` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-metaphors` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-models` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-naga` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-projects` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-qttps` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-quantum` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-runtime` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-sdk` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-tesseract` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-ufbx` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-universe` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dreamwell-waymark` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `droidtui` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `drtahash` 0.0.17:
   HC3 (keyed per map for in-memory hash maps) and HC2 (64-bit).
- `drthashbrown` 0.0.17:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `drv` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dryoc` 1.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `dsfb-atlas` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dsfb-database` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dtt` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-acoustic` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-core` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-elastic` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-electrical` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-em` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-fluid` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-mechanics` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-molecular` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-optics` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-porous` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-quantum` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `dualis-scene` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-shape` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `dualis-thermal` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dualis-units` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dublette` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dumpling` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dup-indexer` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dupblaster` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `dupefinder` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dupehound` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dupes` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dupes-core` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dupes-rust` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `durable-streams` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `durust` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dusk-merkle` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dyn-hash` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `dynamo-kv-hashing` 1.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `e-macros` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eap-oxide` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `earthbucks_blake3` 0.8.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `easy_base64` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `easy_xxhash64` 1.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `easy-hasher` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eat-rocks` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ecfuzz` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ecies` 0.2.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ecies-ed25519` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ecies-ed25519-rev` 0.5.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ecies-ed25519-silene` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ed25519-dalek-blake3` 1.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `edgesentry-audit` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `edgestore` 1.9.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `edgestore-cli` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `edgestore-repl` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `edgestore-tier` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `edgestore-tokio` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eevee` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ego-chat` 0.2.33:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eidetic-engine` 0.15.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `elegance-rs` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `elements-frequency` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `elid` 0.4.51:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `elivagar` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `elworthy-codegen` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `elworthy-rt` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embark` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embark-codec` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embark-crypt` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embark-format` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embark-macros` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embed_it` 7.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embed_it_macros` 7.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embed_it_utils` 7.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embedded-crc-macros` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `embedded-crc32c` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `emdb` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eme2` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `emet` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `emixcrypto` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `emvedb` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `enc_file` 0.6.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `enchantress` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `enclave-pqc-primitives` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `encodec-rs` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `encodex` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `encoding_rs` 0.8.41:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `encoding_rs2` 0.8.36:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `endian-hasher` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `engo-ai` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `engo-cli` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `engo-core` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `enprot` 0.5.79:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `enrichr` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ensync` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `entity-tag` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `entro-hash` 1.0.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `entropy-map` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `entyra` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `envvault` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `envy-secrets` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `erc8004-events` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `etchdb` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eth` 0.60.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eth-id` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eth-signature-verifier` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eth-valkyoth-verify` 0.27.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ethdigest` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ethprim` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `eventfold` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evictor` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evidence-core` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evm-fork-cache-remote` 0.1.0-alpha.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evnx-crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evorule-cli` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evorule-governance` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evorule-hash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `evorule-reactor` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ewf-image` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `exaloglog` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `exaloglog-rs` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `exochain-sdk` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `expanse-capi` 0.6.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `expanse-trie` 0.6.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `expensive` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `expiring-atomic-filter` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `extendhash` 1.0.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ez-hash` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ezlz` 1.2.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ezu-core` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ezu-graph` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `f8` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fact-wasm-core` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `faest` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `falkordb` 0.10.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `farmhash-sys` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast_rsync` 0.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fast-des` 0.8.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fast-graph` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-hex-lite` 0.1.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fast-html-parser` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-md5` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-observe` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-shard` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-sparse-merkle-tree` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-thumbhash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fast-tlsh` 0.1.10:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fastbloom` 0.17.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastcdc-alt` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastcrypto` 0.1.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastcrypto-derive` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastdedup` 1.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastgraph` 0.1.21:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fasthash` 0.4.0:
   HC4:
  bindings compiling C and C++ sources through `fasthash-sys`.
- `fasthash-fork` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fasthash-sys` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fasthash-sys-fork` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastmap` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastmurmur3` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastripgrep` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fastsync` 0.10.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `faucet-core` 1.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fchashmap` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fclones` 0.35.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fcoreutils` 0.22.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fcrc` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `fdh` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fe2o3-highlight` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `featrs` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `feldera-size-of` 0.1.7:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `feox-ann` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `ferrijs-std` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferro-airflow-dag-parser` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferrosys` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferrous-di` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferrox-guards` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferry-cli` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferry-core` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ferry-tui` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ffsend` 0.2.77:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ffuzzy` 0.3.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fhash` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fhc` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fhrn` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fifi` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `filament-crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `filament-packet` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `file-deduplicator` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `file-hashing` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `file-mst` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `file-parse-cache` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `file-time-machine` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `filearco` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `filepack` 0.0.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `filesystem-mcp-rs` 0.1.25:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `find_duplicate_files` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `find-identical-files` 0.39.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fingerprint` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fingerprint-struct` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fips205` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `firmion` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `firmion-std-crc32c` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fission-ir` 0.14.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fitctl` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fixed-cache` 0.1.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fixed-type-id` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fixrs` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flac-io` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flaron-sdk` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flash-map` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flashsieve` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flate2-crc` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fleek-blake3` 1.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fleet-dedup` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flit` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flo_sparse_array` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `float-derive` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `float-derive-macros` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flowstats` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fluence-blake3` 1.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fluentbase-codec` 1.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fluentbase-codec-derive` 1.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flux-importer` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flux-vm-v3` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flynnel` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `flypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fn-dsa` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fn-dsa-comm` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fn-dsa-kgen` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fn-dsa-sign` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fn-dsa-vrfy` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-dtype` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-io` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-iter` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-linalg` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fnp-ndarray` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-python` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fnp-random` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-random-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-runtime` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnp-ufunc` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fnprint` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnv_rs` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnv0` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnv1` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fnv64-rs` 0.3.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foldhash` 0.2.0:
   HC1,
  HC2,
  and HC3:
  scalar,
  64-bit,
  output not stable.
- `foldhash-portable` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fomoxac` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `forage` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `forager-addr` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `forjar` 1.31.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `formal-ai` 0.351.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `formualizer-parse` 3.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fossic` 1.8.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foundry-compilers-core` 0.21.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foundry-rs` 0.6.24:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foxtive` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer` 0.22.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-bench` 0.22.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-common` 0.22.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-intrusive` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-memory` 0.22.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-storage` 0.22.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-storage-bench` 0.7.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-tokio` 0.22.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `foyer-workspace-hack` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-bench` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-columnar` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-conformance` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-dot-kernel` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fp-expr` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-frame` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fp-frankentui` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-groupby` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-index` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-io` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-join` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-nohash-hasher` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-python` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-runtime` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fp-types` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fr-rust` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fracsync` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fract` 3.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `frand` 0.10.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `franken_markdown` 0.4.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `franken_ocr` 0.9.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `franken_whisper` 0.8.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `frankenpandas` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `frankensearch-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `frankensearch-durability` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `freecs` 5.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `freeswitch-sofia-trace-parser` 0.10.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `frf` 0.1.86:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `frigg` 0.10.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fs-mcp-rs` 1.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fs-verity` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fsb` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fselect` 0.10.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fsindex` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fsqlite-mvcc` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fsqlite-pager` 0.4.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fsqlite-types` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fsqlite-wal` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fstool` 0.4.33:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-a11y` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-backend` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-core` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-extras` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-harness` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-i18n` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-layout` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-pty` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-render` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-runtime` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-simd` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-style` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-text` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-tty` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-web` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ftui-widgets` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fugue-ustr` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fundsp` 0.23.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `fusevm` 0.26.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fuzzyhash` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fx-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `fxhash` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gaoya` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `garbled-circuit` 1.3.1-pre.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gcs-rsync` 0.4.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gdelta` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `gearhash` 0.1.4:
   category mismatch:
  rolling hash for content-defined chunking.
- `gemini-protocol` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gemray-net` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `Gen_Prime` 1.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gen-secattest` 0.1.50:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `genesis-preflight` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `genome-rs` 1.0.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `geograph` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gf256` 0.3.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `gh-actions-updater` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghash` 0.6.0:
   category mismatch:
  GHASH universal hash.
- `ghost-flow` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghostflow-autograd` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghostflow-core` 1.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `ghostflow-cuda` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghostflow-data` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghostflow-ml` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghostflow-nn` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ghostflow-optim` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `giant-spellbook` 0.4.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gigachess` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gigagraph` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `git-internal` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `git-parsec` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `git-simple-encrypt` 3.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `githooks` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `githooks-fleet` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `githooks-runtime` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gitway-lib` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `givp` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gloam` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `globalalloc-model` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `glog-tui` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `glossa-dsl` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `glossia` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `glossia-cli` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `glry` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `glyph-rs` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gmcrypto-core` 1.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gmeow-gts` 0.9.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gmsm` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gnu-sort` 1.0.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `go-brrr` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `god-graph` 0.6.0-alpha:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `goish` 0.20.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gol_engines` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `golback` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `golomb-set` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `goosefs-sdk` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-checkpoint` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-cloud` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-core` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-correlation` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-crawl` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-dns` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-fleet` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-graph` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-headless` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-hidden` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-horizontal` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-js` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-origin` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-scm` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-subdomain` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gossan-techstack` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gost94` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gqls-cli` 0.25.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `graphy-analysis` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `graviola` 0.4.1:
   outside the decision scope (cryptographic library).
- `gravityfile` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gravityfile-analyze` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `greattraits` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `griddle` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grim-reaper` 1.0.35:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grit-datatype` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `groestl` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `groestl-aesni` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `groestlcoin` 0.31.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grommet` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grommet-core` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grommet-macros` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grommet-offload` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grommet-testkit` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grommet-topology` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grovedb-bincode` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `groxide` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `grust-graph` 0.20.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gsearch` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `gxhash` 3.5.0:
   HC9 (user decision);
   also no streaming API whose output equals the one-shot result (upstream issue #127),
   HC10.
- `haagenti-zstd` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hakanai` 3.0.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hakanai-lib` 3.0.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `halftime` 0.1.1:
   category mismatch:
  almost-universal hash needing key entropy and padded updates (src/hash.rs:75-81).
- `halter-config` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hana_clerestory` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hanfei-fa` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hannahanna` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hanzo-training` 0.11.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hanzonet-pqc` 1.1.21:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `happy-cracking` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `harlite` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash_by_ref` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash_hasher` 2.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash_histogram` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash_utils` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash-algorithms` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash-iter` 2.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash-rings` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash-sorted-map` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash32` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hash32-derive` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashavatar` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashbrown` 0.17.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashbrown_tstd` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashcodecs` 1.4.1:
   HC10 for XXH3-128 (one-shot,
  prepared,
  and batch APIs only);
   HC1 for MurmurHash3 x64-128,
   whose aarch64 body is scalar (src/murmur3/x64_128.rs:175-176).
- `hashconsing` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashcrew` 0.3.0:
   screening survivor:
  XXH3-128 with NEON,
  SSE2,
  and AVX2 kernels for inputs over 240 bytes,
   streaming `Xxh3_128`,
  Apache-2.0.
- `hashers` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashery` 0.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashes` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashguard` 5.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashify` 0.2.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashing` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashjunkie` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashjunkie-cli` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashkit` 0.1.5:
   category mismatch:
  adapter over other hash crates.
- `hashlen` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashline` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashlogs` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashmap_core` 0.1.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashmatch` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashmatch_macro` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashring` 0.3.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashsigs-rs` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashtree-rs` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hashwires` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hat_trie` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hauchiwa` 0.21.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hayahash` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hayate` 6.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `haz-cache` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hcp-dp` 0.2.0-alpha.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hctr2-rs` 0.9.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-cas` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-cli` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-engine` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-mount` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-oci` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-sandbox` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-spec` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hd-watch` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hekate-crypto` 0.34.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `helicase` 0.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `hematite-cli` 0.14.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `herolib-crypt` 0.3.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hfile` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hgg` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hibp-bin-fetch` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `highhash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hightower-kv` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `highway` 1.3.0:
   screening survivor:
  HighwayHash-128 with SSE4.1,
  AVX2,
  and NEON paths,
  streaming `append`/`finalize128`,
   MIT.
- `highwayhash` 0.0.14:
   HC4:
  bindings compiling C++ sources in build.rs.
- `hisi-crypto-ws63` 0.1.0-alpha.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hiss` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hknt` 1.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hlin-manifest` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hmac-sha1-compact` 1.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hmac-sha256` 1.1.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hmac-sha512` 1.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hnsw-stable` 0.10.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `hnswlib-rs` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holger-cli` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holger-ron` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holger-rust-repository` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holger-traits` 0.6.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holger-znippy-package-repository` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holo_hash` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holodeck` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holographic-memory` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `holt` 0.9.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `hop-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hotline-rs` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hotmint-crypto` 0.8.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hoverstare` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hrw` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hrw-hash` 2.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hsh` 0.0.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hud-slice-by-8` 1.0.10:
   HC2:
  32-bit CRC.
- `huddle` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `huddle-core` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `huddle-gui` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `huddle-protocol` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `huddle-server` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `huginn-net` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `huginn-net-http` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `humanhash` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hya-net` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hyperbitbit` 0.0.1-alpha.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hyperdir` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hyperloglockless` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hypernonsense` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hypertwobits` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hyphae-native-blobs` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `hyprsaver` 0.4.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ic-representation-independent-hash` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iced_nodegraph_sdf` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `icentral-mindexed-map` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ics23-blake3` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `id-slab` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iddqd` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `idempotent` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ident-mash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `identity-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `idhash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ids_service` 2.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `idt` 0.1.23:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ihi` 0.0.48:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ijson` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ikigai-sexpr` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `image_hasher` 3.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `imagehash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `imbh-storage` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `img_hash` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `img_hash_linker` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `img_hash_median` 4.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `imgdd` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `imgfprint` 0.4.6:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `immure` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `immutable-chunkmap` 2.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `importmap` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `incpush` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `indexmap` 2.14.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `indexmap-amortized` 1.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `indextreemap` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `indicatrix-net` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `indxr` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `inferadb-ledger-store` 0.1.0-dev.20260420:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `infiniloom` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `infiniloom-engine` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `inkjet` 0.11.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `integer-hasher` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `intern-mint` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `interned-string` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `internity` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `internment` 0.8.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `intmap` 3.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `inturn` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iota-crypto` 0.23.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ipfrs-network` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `irgx` 2.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iroh-blake3` 1.4.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iroh-blobs` 0.103.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iroh-bytes` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ironaccelerator` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ironaccelerator-cuda` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ironaccelerator-rocm` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ironcrypt` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `irontide` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iscc-lib` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iscsi-client-rs` 0.0.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iscsi-target` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `iso9660-forensic` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `itylos-cli` 2.0.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ix-id` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ja3` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ja4` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jamhash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `janice` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jax-bucket` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jbig2enc-rust` 0.5.4:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `jch` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jdb_xorf` 0.13.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `JenkHash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jevil` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jh` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jh-x86_64` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jhash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jja` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `josekit` 0.10.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jrn` 0.4.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `json_atomic` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `json_lib` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `json-tools-rs` 0.9.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jsonld` 0.22.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jsonld-cli` 0.22.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jsonld-core` 0.22.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jsonld-syntax` 0.22.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jump-consistent-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jumpch` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jumphash` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `justrng` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `jw` 2.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `k12` 0.5.1:
   outside the decision scope (cryptographic);
   scalar.
- `kaboodle` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaccy-db` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kache` 0.23.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kacrab-protocol` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kad` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kadcast` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kael_render_graph` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kafka-wire-records` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kangarootwelve` 0.1.3:
   outside the decision scope (cryptographic).
- `kasetto` 3.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kash` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kash_macros` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `katgpt-micro-belief` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `katgpt-personality` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `katgpt-sleep` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kawat-dedup` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-client` 0.1.113:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-core` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-engine` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-io` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-lsm` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-net` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-raft` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-server` 0.1.113:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-sim` 0.1.110:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kaya-wal` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `keccak-batch` 0.1.0:
   outside the decision scope (cryptographic).
- `keepass-rs` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kerbcore` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kevat` 0.4.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kevy-hash` 6.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kevy-seg` 6.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kevy-vlog` 6.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `keyhog` 0.5.86:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `keyhog-profile` 0.5.86:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `keyquorum-core` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `keystate-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `khata-rs` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `khive-vamana` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kiln-cache` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kimi-fann-core` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kindi` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kineti` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `kino-frequency` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kira_cdh_compat_cluster` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kira_cdh_compat_lsh` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kira_kv_engine` 0.6.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `kira-autolys` 0.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `kira-ls-aligner` 0.4.6:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `kira-mmcif` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `kira-pairs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kira-proteoqc` 0.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `kira-riboqc` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kira-shared-sc-cache` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kira-spatial-field` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `kitak` 3.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kk-crypto` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `KLPhash` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kmerutils` 0.0.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knishio-cli` 0.2.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knot` 1.9.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knotoid` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knx-rs-core` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knx-rs-device` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knx-rs-ip` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knx-rs-prod` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `knx-rs-tp` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kodex` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kokoro-cli` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `komihash` 0.5.0:
   HC1 and HC2:
  scalar,
  64-bit.
- `komoju-datadog` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kontor-storage-node` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `koopman-checksum` 1.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `korp-installer` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `korp-mkfs-xfs` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `koru-delta` 3.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kotoba-cid` 0.1.22:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kova-engine` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `krafka` 0.24.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kremis-core` 0.21.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `krypteia-arcana` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `krypteia-arcana-ffi` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `krypteia-quantica` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `krypteia-tessera` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `krypton-core` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kupyna` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kyberlib` 0.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kyberlib-wasm` 0.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `kyumdb` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `l-s` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `laburnum` 1.17.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lafs` 0.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `laminar-bcs` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lamxfs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lan-acp` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lan-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lang-parsing-substrate` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lasso` 0.7.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lasso2` 0.8.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `last-git-commit` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lattice-embed` 0.10.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lattice-kyber` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lattice-safe-suite` 0.4.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lau-measure-agents` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lava-outcome-chain` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lavinhash` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lcpfs` 2026.1.102:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `leakguard` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `leaktor` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lean-agentic` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `leantoken` 0.1.28:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ledvar-snapshot-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `leveled-hash-map` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lexindex` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `leyline-cas-ffi` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lf-gfx` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lfchring` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lgwks_std` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-aead` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-duplex-aead` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-fn-dsa-alg` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-fn-dsa-comm` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-fn-dsa-kgen` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-fn-dsa-sign` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-fn-dsa-vrfy` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-hash` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-hqc-traits` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-intrinsics` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-k12` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-keccak` 0.0.11:
   outside the decision scope (cryptographic).
- `lib-q-platform` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-plonky-batch-stark` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-plonky-keccak-air` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-plonky-lookup` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-plonky-multilinear-util` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-plonky-uni-stark` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-poseidon` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-rocca-s` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-romulus` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-saturnin` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-slh-dsa` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-air` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-challenger` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-commit` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-dft` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-field` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-stark-field-testing` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-fri` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-interpolation` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-matrix` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-mds` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-merkle` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-mersenne31` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-stark-monty31` 0.0.11:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `lib-q-stark-rayon` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-sha3-256` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-shake128` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-shake256` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-symmetric` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-stark-util` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-tweak-aead` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-utils` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lib-q-zk-encryption-proof` 0.0.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libargon2-sys` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libasp` 1.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `libchibi` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libdictenstein` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libeipc` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libfive` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libgrit-core` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libgrit-git` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libgrit-ipc` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libgrite-core` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libkrypton` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `liblevenshtein` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libmhash` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libphext` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `librefs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `librscrc` 0.1.0:
   HC2:
  32-bit CRC.
- `librsync_oxide` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `librustysigs` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libsm` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libsm_stzhang` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libsmx` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libsoliton` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libsumatracrypt-rs` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libvctrl_handler` 5.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `libvctrl_sha512` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ligerito` 0.6.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `light-poseidon` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lightcycle` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lightflow` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lightning-rapid-gossip-sync` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lightspeed_hash` 0.66.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lilsync` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `linear-hashtbl` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `linebench` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lineprior` 0.11.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lineprior-cli` 0.11.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lintrunner` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `listpack` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lite-strtab` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `litecoin` 0.32.8-rc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `litvc` 1.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lllv-core` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lllv-index` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `llmosafe` 0.7.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `llmsorting` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lloom-auth` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lms` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `localsend-rs` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lockmap` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lockstitch` 0.29.0:
   outside the decision scope (cryptographic).
- `logdb` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `logdbd` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `logdive` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `logdive-api` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `logdive-core` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lonkero` 3.7.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lossless-transform-utils` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `loupe-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `loupe-proto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `loupe-storage` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `loupe-tls` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lowdash` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lox-library` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `loxwebsocket` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lsm-db` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ltk_fantome` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ltk_hashtable` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ltk_meta` 0.8.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ltk_modpkg` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ltk_overlay` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ltk_wad` 0.5.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lumen-lang` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lurk` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lux-consensus` 1.23.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lz_fnv` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `lz4r` 1.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ma` 0.10.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ma-did` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `mac-digest` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `macchikane` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mach-siegbert-vogt-dxcsa` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `machine-cat` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `macro_pub` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mail-parser` 0.11.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `majik-file` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `majik-key` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `make_ultra` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `makechain-crypto` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mako_infinite_shuffle` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `malwaredb-murmurhash3` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `manifestus` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `manzana` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mappy-client` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mappy-core` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `marshal-rs` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mash-rs` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `massmap` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `matchy-literal-hash` 2.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mati` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `matrix256` 1.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mcaptcha_pow_sha256` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mcp-filesystem` 2.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mcp-memory` 5.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `mcp-memory-rs` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mcp-pdf` 3.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `md-5` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `md2` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `md4` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `md5` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `md5-many` 0.1.0-alpha.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `md6` 2.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mdbook-gitinfo` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mdbook-inplace-notes` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mediatime` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `melin-journal` 0.16.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mem4n6` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `memobuild` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `memoize` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `memvid-rs` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `meowhash` 0.3.0:
   HC1:
  aarch64 support disabled since 0.2 (src/arm.rs:9);
   HC3:
  MeowHash 0.5 output not declared final.
- `merc_aterm` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merc_data` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mercs2_population` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle_hash` 3.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle-forest` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle-helix-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle-lite` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle-root` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle-search-tree` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkle-tree-bulletin-board` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkleforge-hash` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `merkleproof` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mesh-core` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mesh-dht` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mesh-sieve` 4.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mesh-transport` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `meshanina` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `metaltile-std` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `metamorphic-crypto` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `metamorphic-log` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `metrohash` 1.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mfsk-core` 0.10.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mhinparser` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `micro_routing` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `micro-moka` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-client-sqlite-store` 0.16.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-crypto` 0.33.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `miden-crypto-derive` 0.33.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-field` 0.33.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-lifted-air` 0.33.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-serde-utils` 0.33.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-stark-transcript` 0.33.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miden-stateful-hasher` 0.33.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mielin` 0.1.0-rc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mielin-cells` 0.1.0-rc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mildly-basic-auth` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minarrow` 0.18.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `mincatcdc` 0.5.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `minhash-rs` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minibit` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minimime` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minimizer-iter` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minimizer-queue` 1.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minlz` 1.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minsync` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `minuet` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `miraland-nohash-hasher` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mirl` 9.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mixed-signals` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mizaru2` 0.2.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mkext4` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mkit-core` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mm3h` 0.1.3:
   HC2:
  MurmurHash3 behind `Hasher::finish`,
  64-bit output.
- `mmap-bitvec` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mnemosyne-rs` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mntime` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mockalloc` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mockalloc-macros` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mod-compatibility-checker` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `modelwarden` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `modern-crypto-toolkit` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mokosh` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `moloon` 1.2.5-alpha:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mongo_common` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `monolithium` 1.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `monotree` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `moonpool-transport` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `morf` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mos-eval` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mos-lsp` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mothcdc` 0.7.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `mpchash` 2.0.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mphf_benchmark` 0.3.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mq-db` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mqtt5` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mrkl` 0.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `msoffice-crypto` 0.1.0-rc.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `msrt` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mule-map` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `multi_index_map` 0.15.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `multi_index_map_derive` 0.15.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `multi-key` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `multimixer-128` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `multiprobe` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `murmur2` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `murmur3` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `murmur3_32` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `murmurhash3` 0.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `murmurhash64` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `murmurs` 1.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `museair` 0.6.0:
   HC1:
  scalar 64-bit multiply design,
  no architecture intrinsics.
- `mwhash` 0.1.1:
   HC1:
  no architecture intrinsics.
- `mycrc` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mydi` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mydi_macros` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `mytheclipse-crypto` 1.21.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `n01d-forge` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nahui` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nam-indexmap` 2.7.1-nam.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nanobook` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nanogbm` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `native-devtools-mcp` 0.10.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `nautivecs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nave` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ncr-crypto` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nea-esi` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nectar-postage-usage` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nefaxer` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nekohash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `neleus-db` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `neo-crypto` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `neocache` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `neon-rs` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `neptune` 13.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `net-mesh` 0.36.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `netviper-talos` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `neuron-encrypt` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `newton-aggregator` 0.5.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `next-plaid-cli` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nexus-ascii` 1.6.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nexus-bits` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nexus-shield` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nexusnet-cache` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-engine` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-formalize` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-import` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-kr` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-lexicon` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-protocol` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-reason` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-render` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-semantics` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-session` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-store` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nibli-types` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nitrite_vector` 1.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `nivalis` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nix-base32` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nix-derivation` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nix-narinfo` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nmaprs` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nodedb-columnar` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nodedb-wal` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `noethers-turnstile-core` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nohash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nohash-hasher` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nohasher` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `noise-aws-lc-rs` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `noiz` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `noncrypto-digests` 0.4.0:
   category mismatch:
  `Digest` trait adapters over other hash crates.
- `norgolith-tree-sitter-highlight` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nornir-catalog` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nostringer` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `noxtls-crypto` 0.2.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `noyalib` 0.0.44:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `ntds-parse` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ntk-common` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nucleus-receipt` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nulid` 0.10.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `num-order` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `num-prime` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `num-valid` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `numext-constructor` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `numext-fixed-hash` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `numext-fixed-hash-core` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `numext-fixed-hash-hack` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `numkong` 7.8.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `nybl-vm` 0.4.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oakvcs-core` 0.102.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oas-crypto` 1.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `obelyzk` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `obfstr` 0.4.6:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `objecthash` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oboron` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oboron-py` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `obsidian-512` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oci-api` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ocr` 0.1.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `of_execution_core` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ogentic-audit` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ogentic-audit-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ogentic-audit-keychain` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ohash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `okid` 0.26.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `omango-wyhash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `omnibor` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `omnicat` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `omnicli-app` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `omnizip-codecs` 0.21.98:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `one-saves` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `one-saves-cli` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `onelastleaf-plugin-sdk` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ontocore-catalog` 0.26.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oomfi` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `open-eeg-codec-standard` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `open-mpm` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `open-wal` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `openagent-crypto-wasm` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `opencrabs` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `opendict-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `openjd-snapshots` 0.1.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `openjpeg2-pure-rs` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `openvet-crypto` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `openxos-probe` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `opthash` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `optimesh` 1.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `optimus-cli` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `opus-pure` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `orbinum-protocol-core` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ordered-segment-map` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ordermap` 1.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ordnung` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ordsum` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `origin-crypto-sdk` 0.6.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `osmo` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `otaripper` 3.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `outdatty` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oversync` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxc-browserslist` 5.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxgraph-layout-util` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxgraph-snapshot` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiarc-core` 0.4.2:
   category mismatch:
  archive components;
   its CRCs are at most 64 bits (HC2).
- `oxiarc-lz4` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiarc-lzhuf` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiarc-lzma` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiarc-snappy` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxibase` 0.5.15:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxibonsai-cli` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicrypto` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicrypto-adapter-aws-lc` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicrypto-bench` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicrypto-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicrypto-hash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicrypto-mac` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxicuda-sketch` 0.5.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxideav-aacs` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxideav-core` 0.1.36:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxideav-pdf` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxidecop` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxidelm` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxidite-security` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `oxieml` 0.1.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oxify-vector` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oxigeo-index` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxigrid` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxihuman-morph` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oxilean-build` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxillama-gguf` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-archive` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-audio` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-cache` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oximedia-conform` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-dedup` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-forensics` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-io` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-search` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-video` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oximedia-watermark` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oximg` 0.11.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oxionnx` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiphysics-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiquic-crypto` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxirs-cluster` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxirs-embed` 0.4.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oxistore-cache` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxistore-encrypt` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiui-iced` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxixml-qname` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `oxiz-core` 0.3.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `oxymcts` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `p2panda-blobs` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `p2panda-core` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `p2ps` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `p3-blake3` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `p3-blake3-air` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `paccel` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pacchetto` 0.0.1-beta:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pacha` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `packet_parser` 10.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `packmap` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `packrat-tui` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `packx` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `page-db` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pallet-bridge-parachains` 0.29.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `paq` 1.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `par-particle-life` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `par2-rs` 0.10.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `par3-rs` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `parallax-pipeline` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `paraoxidizer` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `parity-db` 0.5.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `parmesan-par2` 0.5.7:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `parquet` 60.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `parsanol` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `passgenz` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `passman-rs` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `passwd-derive` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `password_manager` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `patchy` 0.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `patent-hash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pathfinder-mcp` 0.23.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pbrt-r4` 4.3.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `pcap-toolkit` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pcf-sig` 0.0.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pco_pack` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pct` 4.0.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `pdf2john` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pdfplumber` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pdfrum-crypt` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pdq-rs` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pdqhash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pebble-cms` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pebbledb` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pegase` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `percepthash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `perfgate-sha256` 0.15.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `peroxide-cryptsetup` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `personae` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pezpallet-bridge-teyrchains` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pflow` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pg_ripple_http` 0.136.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pgen` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ph` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ph-temp` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `phext-shell` 0.1.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `philharmonic-connector-common` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `philharmonic-policy` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `phonelib` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `photara` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `photofold` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `photon-messenger` 0.0.39:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `phyllium-fingerprint` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `phylo` 6.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `physis-core` 0.1.25:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pi_agent_rust` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pi_hash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `picovolt` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pigdb` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pil2-stark-setup` 1.2.0-alpha:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pin-derive-core` 0.11.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pingly` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pingora-ketama` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pithanos` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pkgar` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `plain_hasher` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `plakat` 6.31.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `planar_convex_hull` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pleat` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pleco` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `plexus-substrate` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `plonky2_por` 1.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `plugmem-core` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `plugmem-host` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pmat` 3.40.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `pmmlruntime` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `png-spark` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `poc-build` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pocx_hashlib` 1.0.5:
   outside the decision scope (cryptographic).
- `pocx_miner` 1.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `poet-rhai` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pojoc` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pojoc-build` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pojoc-cli` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `poly1305` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `poly1743` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `polydup` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `polydup-cli` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `polydup-core` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `polyhash` 0.3.1:
   category mismatch:
  POLYVAL and GHASH universal hashes.
- `polymur-hash` 0.2.2:
   HC1 and HC2:
  scalar,
  64-bit.
- `polyval` 0.7.3:
   category mismatch:
  POLYVAL universal hash over GF(2^128),
  a MAC building block needing a key and padding.
- `portable-hash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `portail` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `poseidon-hash` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `posvault_handler` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pow` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pow_sha256` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pow-buster` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb-auth` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb-backup` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb-query` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb-server` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb-storage` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `powdb-sync` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `power-plant-semantic` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ppar` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pq-mceliece` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `pq-modern-rust-tls` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqc_sphincsplus` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqc-combo` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqc-fips` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqc-privacy` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqc-sig` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqfile` 4.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqguard` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqrascv-bitcoin-anchor` 1.0.0-rc.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqrascv-cli` 1.0.0-rc.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqrascv-core` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqrascv-hardware` 1.0.0-rc.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqrascv-sigstore-client` 1.0.0-rc.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqrascv-verifier` 1.0.0-rc.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pqty` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `praefectus` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `precursor` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `preflate-rs` 0.7.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prehash` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pretender` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prime-radiant` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `probemap` 0.1.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `probminhash` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `process_consistency` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `project-rag` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prolly-map` 0.7.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `prolly-store-dynamodb` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prolog8` 26.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prople-crypto` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prosody` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prosopo-ja4` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `prosto_derive` 0.11.26:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `proto_rs` 0.11.26:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `proton-drive-rs` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `proton-sdk` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `provenance-mark` 0.24.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `proxy-protocol-rs` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `proxyauth` 1.2.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pruefung` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ps-hash` 0.1.0-27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `psitool` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pso-antispam` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pso-vdf` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `psummary` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ptr_hash` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pulith` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pulse_map` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pulsedb` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `punchline` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `punchline-proto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `punchline-signald` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `punchline-stund` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pure_rng` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `purecrypto` 0.9.0:
   outside the decision scope (cryptographic library).
- `purrdf-entail` 2.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pwhash` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pxs` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pyde-rust-sdk` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `pyo3` 0.29.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qasa` 0.0.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qatsi` 1.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qbice` 0.6.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qbix` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qfilter` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qids` 1.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qjl-sketch` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qndx-cli` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qndx-core` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qndx-git` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qndx-index` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qndx-query` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qorx` 1.0.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qr-base44` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qrc-opensource-rs` 0.3.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qrcode53bytes` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qssl` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qssm-proofs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qssm-utils` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quack-rs` 0.16.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quad-compat-rhai` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quai-abi` 0.1.0-alpha.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qual` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quant-codec-core` 0.1.0-alpha.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quant-eval` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quanta-wasm` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quantacore-sdk` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quantica` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `quantize-rs` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quantogram` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quantom_value` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quarto-yaml` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qubit-value` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qudag-crypto` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qudag-vault-core` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `query-flow` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `query-flow-inspector` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `query-flow-macros` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quichash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quichash-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quickbloom` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quickcrypt` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quickdash` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quickphf` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quickxorhash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quill-core` 0.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quilt-rs` 0.39.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quipu` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `quipu-cnsa` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `quo-rust` 0.1.95:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `quorum-sdk` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `qux-pqc` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `r14-poseidon` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `r255b3` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `raasta` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `radbeeper` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `radicle-artifact` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `radicle-artifact-client` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `radicle-artifact-core` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `radicle-artifact-node` 0.18.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-api` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-assets` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-base` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-framework` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-plugins` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-renderer` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-shader-processor` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rafx-visibility` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rag-cli` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rag-server_blf` 2.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rag-server-mcp` 2.3.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ragloom` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ragrig` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rahashmap` 0.2.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rand_blake3` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rand_hash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rand_krull` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rand_seeder` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rand-wyrand` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `random_access_rng` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rankfusion` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rapidgzip-core` 0.3.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rapidgzip-rust-cli` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rapidhash` 4.5.1:
   HC1 and HC2:
  scalar 64-bit multiply design,
  64-bit outputs.
- `rapidhash-u128` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rapidrand` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rart` 0.11.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rasn-compiler` 0.16.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rasn-compiler-derive` 0.16.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rasterrocket` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rasterrocket-cli` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rasterrocket-color` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `rasterrocket-encode` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `rasterrocket-font` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `rasterrocket-interp` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `rasterrocket-parser` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `rasterrocket-render` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `ratatui-wgpu` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rater` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ratify` 2.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ratiometer` 0.12.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rattler_digest` 1.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rav1d-safe` 0.6.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rayon-hash` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rbt-datalake` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rclip-codepage` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcn` 1.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rcp-tools-common` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-congestion` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-filegen` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-rchm` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-rcmp` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-rcp` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-remote` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-rlink` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-rrm` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rcp-tools-throttle` 0.40.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rdebootstrap` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rdpe-editor` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `readcon-db` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `realpix` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rebgzf` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `receipt-bench` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `recoco-utils` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `reconcile` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `redb-extras` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `redis_config` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `redis-work-queue` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `redisesh` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `reed-solomon-simd` 3.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `reelforge` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `reflicate` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `reflow_assets` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `reishi-handshake` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `relayrl_types` 0.9.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `remarkable-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rendezvous_hash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `replication-engine` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `repocert` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `repolith-actions` 0.0.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `repolith-cache` 0.0.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `repolith-cli` 0.0.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `repolith-core` 0.0.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `repolith-engine` 0.0.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `req-cli` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `requirements-manager` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `residiuum-format` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `residiuum-snapshot-format` 0.1.0-alpha.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `resq-bin` 0.1.21:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `retree` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `retrospector` 1.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `retsu` 0.5.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `revelo-util` 0.5.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rewal` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rex` 4.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rezzy` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rfe-types` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rhai` 1.26.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rhai-dylib` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rhythm-open-exchange` 0.6.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rialo-s-blake3-hasher` 0.18.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ribbon-filter` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rigorix-engine` 1.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rijndael` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rime-xds` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ringmap` 0.2.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ripemd` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `risc0-crypto` 0.1.0-rc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `risc0-sppark` 5.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ritehash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `riverctl` 0.2.18:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rivven-connect` 0.0.22:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rivven-core` 0.0.22:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rldb` 0.1.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rllvm` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rln` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rlnc-cat-rs` 0.2.4:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rlx-fft` 0.2.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rmqtt-acl` 0.23.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rn-ota-server-rust` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `robinxx_map` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `robust_downloader` 0.0.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `robustack-dl` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rok-crypto` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rollblock` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rolling-dual-crc` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rolodex-dns` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ronnie-crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rootchain-crypto` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rosalind-receipt` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rotohash-rs` 0.1.2:
   HC10:
  one-shot `hash` and `hash_with_seed` only (src/lib.rs:87,
  :158);
   HC3 not established:
  the reference README calls the algorithm unlikely to change,
   with quality analysis ongoing.
- `rotortree` 0.20.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roughly` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `routemap` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-audio` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-cavegen` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-cli` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-core` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-formats` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-gpu` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-render` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `roxlap-scene` 0.32.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rpm` 0.28.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_aes` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_merkle` 1.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_sha256` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_sha3_256` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_sha512_256` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_shield` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs_ssl` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `rs-ali-oss` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs-io` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs-merkle-tree` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs-x11-hash` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rs3gw` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsa-msg` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsa-msg-server` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsbloom` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rscrypto` 0.9.0:
   outside the decision scope (cryptographic library).
- `rsdedup` 0.1.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-common` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-index` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-ingest` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-metastore` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-search` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-server` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsearch-storage` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsemu` 0.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsfn-file` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsfulmen` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rskit` 0.0.37:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsomics-vcf` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsos` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rssn-advanced` 0.1.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rstmdb-protocol` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rstmdb-storage` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rstmdb-wal` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rsupd` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rta-derive` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rttp` 1.2.5-alpha:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rubato` 5.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rubylang` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-common` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-encoding` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-exec` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-io` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-ir` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-kernels` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-parquet` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-storage` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rudb-vector` 0.3.30:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruitl` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruitl_compiler` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rulake` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rullst-auth` 12.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `run-kit` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rune-fnv` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rune-murmur` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rune-ring` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rune-xxhash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruprizzle-core` 1.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruqu` 0.1.32:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruscrypt` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusdox` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust_h265` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust_keylock` 0.18.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-argon2` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-bottle` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-crypto-utils` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-featurecounts` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-filesearch` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-native-obf` 0.1.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rust-par2` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rust-zstd` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustafits` 1.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rustbinary` 0.1.8:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rustbinary-derive` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustc_tools_util` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustc-hash` 2.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustc-stable-hash` 0.1.2:
   HC1:
  scalar SipHasher128.
- `rustchain-wallet` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustfs-crypto` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustfuscator` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustgenhash` 0.16.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustgym` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustic_cdc` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustic-git` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustium-config` 0.1.0-alpha.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustium-sqlserver` 0.1.0-alpha.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustledger` 0.24.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustledger-ops` 0.24.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustls-ccm` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustnetconf` 0.17.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustpix-algorithms` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustra` 0.10.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusty_av1e` 0.8.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rusty_dds` 0.9.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rusty_h265` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusty_json_turbo` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusty_mp3` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusty_vault` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusty-page-indexer` 0.5.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rusty-stack` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustybara` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustycache` 2.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rustyhdf5-accel` 1.93.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `rustypyxl` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruvector-fpga-transformer` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruvector-robotics` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruvector-scipix` 2.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruvector-verified` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ruvllm-esp32` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rvf-manifest` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rvf-runtime` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rvf-types` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rvf-wire` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `rvoip-sip-core` 0.3.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ryg-rans-rs-casefile` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ryg-rans-rs-cli` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ryg-rans-rs-parallel` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ryg-rans-rs-simd` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ryu-tracing` 0.1.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s3-endpoint` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s3-unspool` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s3-wire` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s3ls-rs` 1.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s3util-rs` 1.10.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s4-codec-py` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s5_fs` 1.0.0-beta.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `s7cmd` 1.8.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sa-token-rust` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sacp-cbor` 0.18.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `salamander-db` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `samesame-rolling` 1.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sanitization-crypto-interop` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `santh-encodex` 0.1.15:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `saorsa-core` 0.27.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `saorsa-fec` 0.4.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `saorsa-gossip` 0.5.82:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `saorsa-gossip-types` 0.5.82:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `saorsa-pqc` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `saorsa-transport` 0.36.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sarlacc` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sascar` 1.2.5-alpha:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `satsnet` 0.32.17:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sc_neurocore_engine` 3.16.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scalo` 2.12.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scarb-stable-hash` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scc` 3.8.8:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scc2` 2.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scenesdetect` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `schemapin` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `schnorr48` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scintia-96` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scirs2-core` 0.6.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scirs2-graph` 0.6.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scirs2-io` 0.6.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scirs2-spatial` 0.6.5:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scope-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scribe-scaling` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scribe-scanner` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scriptr` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scry-gpu` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scrypt-opt` 0.3.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `scsh` 1.46.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `scytale` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sdbm` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `seahash` 4.1.0:
   HC1 and HC2:
  scalar,
  64-bit.
- `seal-crypto` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `seal-crypto-wrapper` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sec-mem` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `secret-vault` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `secretx` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `secure-auth` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `secure-edit` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `SEDSnet` 4.0.33:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `seedable_hash` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `seekstorm` 3.3.13:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `self_encryption` 0.36.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `self-awareness` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sem-cli` 0.25.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sem-cloud-client` 0.25.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sem-core` 0.25.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sem-mcp` 0.25.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `semdup` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `semiuniq` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `senax-encoder` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `senba` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sensor-db` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sentinel-cli` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sentinel-crypto` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sentinel-dbms` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sentinel-wal` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `seq-hash` 0.2.0:
   category mismatch:
  DNA k-mer hashing.
- `seqsum` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `seqtable` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sequence_trie` 0.3.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `serverctl` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `set_associative` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha` 1.0.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `sha-1` 0.10.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha1` 0.11.0:
   outside the decision scope (cryptographic).
- `sha1-checked` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha1collisiondetection` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha2` 0.11.0:
   outside the decision scope (cryptographic);
   measured as the R1 control (SHA extensions on both architectures).
- `sha2_ce` 0.10.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha256` 1.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha2ni` 0.8.5:
   outside the decision scope (cryptographic);
   x86 only.
- `sha2raw` 14.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha3` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha3_ce` 0.10.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha3-kernel-hasher` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sha3-selkie` 0.0.0:
   outside the decision scope (cryptographic).
- `sha3sum` 1.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shabal` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shacl2cypher` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shacl2cypher-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shacl2cypher-runner` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shad3` 1.1.51:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shadowhare` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shadowsocks-crypto` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shadowsocks-rust` 1.25.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shaha` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shake` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shaman` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `SHARAG256` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sharc` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shared-aes-enc` 0.3.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sharezed` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shasha` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shex` 2.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shield-core` 2.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shiguredo_s3` 2026.1.0-canary.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ship-shape` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shipmates` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shitty-vt` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shitty-vt-sys` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shoes` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shogi_core` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shrimps-signer` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `shuflr-wire` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sigma_fun` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sigma-protocols` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-aptos` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-arweave` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-btc` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-casper` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-cli` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-cosmos` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-evm` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-fil` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-nostr` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-primitives` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-spark` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-sui` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-svm` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-ton` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-tron` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signer-xrpl` 3.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `signinwithethereum` 0.8.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `silk-graph` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `simd-adler32` 0.3.10:
   HC2:
  32-bit checksum.
- `simd-json` 0.18.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `simd-minimizers` 3.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `simd-r-drive` 0.17.1-alpha:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `simd-sketch` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `simhash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `simple_term_rewriter` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `simplehash` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sinteflake` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `siphash_c_d` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `siphasher` 1.0.3:
   HC1:
  scalar SipHash,
  including its 128-bit variant.
- `sirraya-crypto` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `size_lru` 0.1.38:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `size-of` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sk_dkim` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `skeg-hull` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `skein` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `skein-hash` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sketch-spgemm` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `skippy-cache` 0.76.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `skippydb` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sklears-neighbors` 0.2.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `slahasher` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `slate-kv-core` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `slatec-sys` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `slice-by-8` 1.0.11:
   HC2:
  32-bit CRC.
- `slowlock` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sm3` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `small_hash_map` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `small-collections` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `small-map` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smallperm` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smallrand` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smbus-pec` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smchash` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smeltr` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smugglr` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smugglr-core` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smugglr-http-sql` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smugglr-plugin-sdk` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `smugglr-wire` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sn0int` 0.26.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `snapfire_fsr_core` 0.6.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `snapfire_fsr_runtime` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `snappy_framed` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sniplab` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `snow` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sntrup` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `sntrup761` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `snyd` 0.2.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sokm-multimodal` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solana-awesome` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solana-blake3-hasher` 3.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solana-ecies` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solana-ed25519` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solana-falcon512` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solana-nohash-hasher` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `solidb` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `soma-infra` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sorted-iter` 0.1.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sosistab2-obfsudp` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sounding-wyoming-text-list` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `souphash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sp1_bls12_381` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sp800-185` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `space_trav_lr_rust` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spacemap` 1.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spacetravlr` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparkid` 2.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparrowdb` 0.1.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparrowdb-catalog` 0.1.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparrowdb-common` 0.1.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparrowdb-cypher` 0.1.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparrowdb-execution` 0.1.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparrowdb-storage` 0.1.27:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparse-ngrams` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparsemap` 5.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparsl` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparsync` 0.1.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spartan` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spartan2` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sparx` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spatialtree` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spectral-fingerprint` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spectre_pdf` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `spectrex` 0.3.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spekter` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spekter-cli` 1.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spekter-core` 1.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spellbook` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spg-audit` 8.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spg-crypto` 8.0.4:
   outside the decision scope (cryptographic).
- `spider_agent_types` 2.53.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `splashsurf` 0.14.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `spo-rhai` 1.17.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sponge-hash-aes256` 1.10.7:
   outside the decision scope (cryptographic).
- `spow` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sqlite-compressions` 0.3.12:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sqlite-forensic` 0.12.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sqlite-functions` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sqlite-hashes` 0.10.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `squall-router` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `squeez` 1.48.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `srx-rs` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ssdeep` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sse2neon` 0.1.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `ssg` 0.0.63:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ssg-core` 0.0.63:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ssri2` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sssync` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stacks-dex-pools` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stacksat128` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `star-toml` 26.7.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `starcat` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stardex` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stateset-crypto` 1.35.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stats-claw` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `stdpython` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stdrandom` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stdto` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stdto_core` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stdto_derive` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `steam-vdf-parser` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stego_rust` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stenoxide-cli` 3.31.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sthash` 0.2.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stingy` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stoolap` 0.4.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `storage-guardian` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `strata-jali` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `strata-kuro` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `strata-nebu` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `streambed-logged` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `streebog` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stringzilla` 5.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `strixonomy-catalog` 0.28.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `strobemers` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `structdiff` 0.7.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `structured-zstd` 0.0.53:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `strx` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `stryke_web` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `strykelang` 0.17.52:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `student-key-manager` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `subetha-ffi` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sublinear` 0.3.3:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `subms-cuckoo-filter` 0.10.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `substrate-zainium` 2.3.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sudp` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sui-cache-eval` 0.1.219:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sui-graph-store` 0.1.219:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sumchain-crypto` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sundog` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `SuperBit` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `superbit_lsh` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `superbloom` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `superboring` 0.1.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `supply-chain-trust-crate-000003` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `supply-chain-trust-crate-000013` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `supply-chain-trust-example-crate-000003` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `supply-chain-trust-example-crate-000013` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `supply-chain-trust-example-crate-000069` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `svid` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `svm-hash` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `swap-pool` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `swiss-table` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `swt3-ai` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sxurl` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `symbios-tensor` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `symbios-texture` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `symspell_complete_rapid` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `symspell_complete_rs` 0.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `synadb` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `synaptic-wiring` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `syndrome` 0.6.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `syntarq-core` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `syntheca` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `synx` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `systemprompt-traits` 0.54.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `systile` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `sz-orm-crypto` 6.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `t1ha` 0.1.2:
   HC1:
  x86 intrinsics only;
   HC3:
  the AES variant t1ha0 differs by platform.
- `tab-hash` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tack-pins` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tacos` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tagotip-secure` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tameshi` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tanton` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tape-sha256` 0.2.1:
   outside the decision scope (cryptographic).
- `tari-tiny-keccak` 2.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tarsum` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tasktree` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tatara-lisp-source` 0.3.58:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tatara-reconciler` 0.2.701:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tattler` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `taudit` 1.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tauler-configgen` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tayf` 0.12.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tbz-airlock` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tbz-cli` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tbz-core` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tbz-jis` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tbz-mirror` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tdln-ast` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tdln-proof` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tectonic-fn-dsa` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tectonic-fn-dsa-comm` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tectonic-fn-dsa-kgen` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tectonic-fn-dsa-sign` 0.3.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `tectonic-fn-dsa-vrfy` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `telepath-wire` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tenrso-ooc` 0.1.0-rc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensogram-ffi` 0.24.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensor-id` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensor-wasm-artifacts` 0.3.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensor-wasm-jit` 0.3.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensordb` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensorlogic-ir` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tensorlogic-scirs-backend` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tenthash` 1.1.0:
   HC1:
  scalar 160-bit TentHash.
- `tenzro-crypto` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tenzro-types` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ter-music-rust` 2.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tessera-embeddings` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `test_kms_server` 5.27.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tetra3` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tetsy-plain-hasher` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `text-mirror` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `the-q` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `thenodes` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `thread` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `thread-utilities` 0.1.3:
   HC1:
  adapter over `rapidhash`.
- `threatflux-hashing` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `threers` 0.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `throbber-widgets-tui` 0.11.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `throbber-widgets-tui-julien-cpsn` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tibet-trust-kernel` 1.0.0-alpha.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tibet-zip-airlock` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tibet-zip-cli` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tibet-zip-core` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tibet-zip-jis` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tibet-zip-mirror` 2.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tide-fn-dsa` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tide-fn-dsa-comm` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tide-fn-dsa-kgen` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `tide-fn-dsa-sign` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `tide-fn-dsa-vrfy` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tidecoin` 0.33.0-beta:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tidepool-version-manager` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tidesdb` 0.11.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tiger` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tightbeam-rs` 0.14.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tilecoding` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `timebased-id` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `timecapsule` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `timestretch` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tiny-keccak` 2.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tinyhash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tinyptrmap` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tinysearch-cuckoofilter` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-core` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-infer` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-ledger` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-net` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-proto` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-sdk` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tirami-shard` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `titor` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tlpsign` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tlsh2` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tmdp` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tmpl-resolver` 0.0.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tmuxpulse` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `toa` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tobj` 4.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tohu` 0.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tok-grammar` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `toka` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tokitai-filekv` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tokmd-content` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tokmd-redact` 1.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ton-net-cell` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `toolkit-zero` 5.11.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `toolpath-codex` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `topon` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tor-metrics-aggregator` 1.1.31:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `torchbear` 0.11.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `torrentdht` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `torsh-package` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `toxi-security` 3.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tpt-cv-feature` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tracel-deploy` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tracing-datadog` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `transportal` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `trash_parallelism` 0.1.102:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `treant` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `trezoa-blake3-hasher` 3.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `trezoa-nohash-hasher` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `trident-lang` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `triplets` 0.27.1-alpha:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `triviumdb` 0.8.8:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `truestack` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `trustformers` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `trustformers-tokenizers` 0.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `trusty-search` 0.52.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tsafe-attest` 4.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tsoracle-driver-file` 2.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tulna-rs` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tunnr-vm` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turbo_crc` 0.0.4:
   HC2:
  32-bit CRC.
- `turbo-cdn` 0.8.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turboani` 0.1.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turbomcp` 3.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turboquant-rs` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turboshake` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turbovault-graph` 2.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `turbovec` 1.0.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `twenty-first` 2.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `twox-hash` 2.1.4:
   screening survivor:
  XXH3-128 with run-time AVX2 and SSE2 dispatch and NEON,
  streaming `XxHash3_128`,
   MIT.
- `twox-hash-cli` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tx2-link` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tx2-pack` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `txmap` 3.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `txtfp` 0.3.2:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `type_hash` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `type-signature` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `type-signature-derive` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `typebit` 0.1.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `typed-session` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `typed-ski` 0.14.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `tzst` 0.1.20:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubl-codec` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubl-crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubl-id` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubl-ledger` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubl-types` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubq` 7.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ubt` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ucfp` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uchikomi` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ucp-local` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uddsketch-rs` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `uefivars` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uffs-text` 0.6.40:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uldren-loom-cli` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ultra_hash_0x` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ultraloglog` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ultranix-mcp` 1.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umac` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umash` 0.6.1:
   HC4:
  bindings to the C library through `umash-sys`.
- `umash-sys` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-chem` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-coordgen-sys` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-edn-macros` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-geometric` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-geometric-core` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-geometric-graph` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-graph` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-graph-ir` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-graph-ir-macros` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-io` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-msym` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-msym-sys` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-nauty-sys` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-params` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-perm` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-py` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `umol-utils` 0.8.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `unfhash` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `unigateway-session` 2.15.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `universal-hash` 0.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `unrar-rs` 0.10.7:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `unstrip` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `unsynn` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uor-foundation` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uor-foundation-sdk` 0.5.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uri-register` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `urlsieve` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `urng` 0.13.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `ursa` 0.3.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `use-xz` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `usearch` 2.26.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uselesskey-core-hash` 0.7.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `usf` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ustr` 1.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ustr-fxhash` 1.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uts-cli` 0.1.0-alpha.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uuidv5` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uuinfo` 0.7.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `uxi` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `v_flakes` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vacp2p_pmtree` 3.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vakint` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `valknut-rs` 1.5.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `vantadb` 0.5.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `varchain` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `varta-vlp` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vecstasy` 0.1.10:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `vector-core` 0.9.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `vectorized-markdown` 1.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `veilus-fingerprint` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `veilus-fingerprint-core` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `veilus-fingerprint-data` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `veles-core` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `velo-common` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vendetta-chess-engine` 1.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verify-beacon` 0.1.2:
   outside the decision scope (cryptographic).
- `verit` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verit-cli` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verit-core` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verit-derive` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `veritate` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verkle_pq` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verum` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verum-arbiter` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verum-faber` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verum-lumen` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verum-mappa` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `verum-nucleus` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vespetrel-core` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vex-algoswitch` 1.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vhdx-forensic` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `viator` 0.1.19:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vibe-fs` 0.0.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vibe-index` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vibe-style` 0.2.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vibesql` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vid_dup_finder_lib` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vigil-sdk` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vil_hash` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `visual-hash` 3.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `visual-hashing` 0.1.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vize_doctor` 0.424.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vmd` 1.0.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `voided-core` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vole-audio` 0.90.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vole-gfx` 0.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vole-video` 0.22.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `voxel-light` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `voxelis` 25.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `vpack` 1.0.0-rc.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vpod` 0.8.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vrd` 0.0.12:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `vsf` 0.9.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vuke` 0.9.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vwh-core` 4.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vyre-grammar-gen` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vyre-libs` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `vyre-macros` 0.7.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `waddling-errors-hash` 0.7.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wal-db` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wallet-generator-cli` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wallswitch` 0.66.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `warmplane` 0.30.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wasi-crypto-wasmtime` 47.0.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wasm_web_crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wasm-rquickjs-cli` 0.4.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `waterui-ffi` 0.3.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wavio` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wbftree` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `we-trust` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weak-table` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weak-table2` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-clone` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-git` 0.3.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-graph` 0.6.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-memory` 0.3.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-scan` 0.5.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-search` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weavatrix-worktree` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weaver-core` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weaver-dvr` 1.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `webbuf_blake3` 0.15.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `webcentral` 2.4.15:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `webdataset-tar` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `webgates-secrets` 1.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `webtorrent-rs` 1.0.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `webview-bundle` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wechat_work_crypto` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wechat-pub-rs` 0.7.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wedb_embed` 0.1.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `weebill` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `whale` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `whasher` 0.1.8:
   HC9:
  wraps `gxhash` 3.5.0.
- `whirlpool` 0.11.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `whirlwind` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `whycache` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wickra-benchmark-cli` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wickra-benchmark-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wickra-proof-cli` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wickra-proof-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wickra-verify-cli` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wickra-verify-core` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wimcc` 1.6.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `win-crypto-ng` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `windex` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `winter-air` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `winter-crypto` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `winter-fri` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `winter-verifier` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `winterfell` 0.13.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wirespec-rt` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `witchauth` 0.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wlgen-rs` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wnfs-namefilter` 0.1.23:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wod` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wombatkv-cabi` 0.1.0-alpha.pre1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wombatkv-core` 0.1.0-alpha.pre1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wombatkv-daemon` 0.1.0-alpha.pre1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wombatkv-format` 0.1.0-alpha.pre1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `woofmt` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wpa-psk` 0.2.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wpawolf` 1.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wrecord` 0.1.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `writeahead` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wsi-dicom` 0.7.5:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wvb` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wy` 1.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wyhash` 0.6.0:
   HC1 and HC2:
  scalar,
  64-bit.
- `wyhash-final4` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `wyhash2` 0.2.1:
   HC1 and HC2:
  scalar,
  64-bit.
- `wyrand` 0.5.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `x25519-parser` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `x402-chain-tron` 0.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xaor` 0.2.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xecrypt` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xfs-core` 0.1.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xfs-forensic` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xgraph` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xgx_intern` 0.6.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xhash` 0.1.38:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xlb` 0.8.37:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xlsynth-driver` 0.70.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xmltv` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xorf` 0.13.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xorsum` 4.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xs_foundation` 0.4.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xsalsa20poly1305` 0.9.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xtask` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xtask-kit` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xuniq` 0.1.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xx-hash` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxblake3` 0.0.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxh` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxh3` 0.1.1:
   HC10:
  one-shot `hash128_with_seed` only (src/xxh3.rs:86);
   last release 2022.
- `xxh3_hashid_macro` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxhash` 0.0.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxhash-c` 0.8.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxhash-c-sys` 0.8.7:
   HC4:
  compiles the C reference.
- `xxhash-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxhash-rust` 0.8.18:
   screening survivor:
  XXH3-128 with SSE2,
  AVX2,
  AVX-512,
  and NEON paths selected at compile time,
   streaming `Xxh3`,
  BSL-1.0.
- `xxhash-sys` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxhash2` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xxhrs` 2.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xynth` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `xz4rust` 0.2.3:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `yadf` 1.4.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `yama` 0.4.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `yo-common` 0.3.33:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ys-kcp` 1.2.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ytsaurus-rpc` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `yume-pdq` 1.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `yykv-event` 0.0.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `yykv-layout` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zakura-halo2-poseidon` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zakura-sinsemilla` 1.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zbox` 0.9.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zca-rs` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zcash-memo-decode` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zccache-fingerprint` 1.12.14:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zeldhash-parser` 0.5.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `Zencore-rs` 1.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenflate` 0.4.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `zenith-api` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-cache` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-capability` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-ebpf` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-fingerprint` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-forward` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-foundation` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-http1` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-http2` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-http3` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-linux` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-net` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-observability` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-proxy` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-runtime` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-stack` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-testkit` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-tls` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-waf` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenith-web` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenoh_raft` 0.1.7:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenpixels` 0.2.16:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenpixels-convert` 0.2.16:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `zenpng` 0.1.4:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `zenpredict` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zensim-regress` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zenzop` 0.4.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zeph-durable` 0.22.4:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zeppelin_core` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zerodds-foundation` 1.0.0-rc.6:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zerokit_utils` 3.0.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zerostyl-runtime` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zexc` 1.2.1:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `zfs-recompress` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zipher` 0.3.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zipora` 4.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `zk_disorder` 0.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zk-psi-verifier` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zkboo-bip32` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `zkboo-circuit-hash` 0.0.0:
   no downloadable stable version (no stable release,
  or the download failed);
   name and description show no byte-string hash library.
- `zkforge` 1.1.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `znippy` 0.9.9:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `znippy-cli` 0.9.10:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `znippy-common` 0.9.15:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `znippy-compress` 0.9.13:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `znippy-decompress` 0.9.11:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zrip` 0.8.8:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zrtp` 0.1.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zsh` 0.12.63:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zshrs` 0.12.64:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zstash` 0.3.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zstd-complete` 0.2.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `ztensor` 2.1.1:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zvec-core` 0.1.0:
   category mismatch:
  not a byte-string hash library (manual review of description and source layout).
- `zwohash` 0.1.2:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
- `zync-core` 0.6.0:
   HC1:
  source scan found intrinsics for at most one of x86_64 and aarch64 and no SIMD abstraction.
