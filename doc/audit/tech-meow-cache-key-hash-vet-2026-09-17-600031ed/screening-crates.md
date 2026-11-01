# Screening appendix: every crates.io result

Appendix
to [`tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md`](../tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md),
 section "Screening".
One entry per distinct crate returned by the crates.io queries of this vet
 (CC01 to CC24 and XC01 to XC04)
 and of the prior vet
 (CR01 to CR14, XR01 to XR04, reused under "Reuse of prior evidence"),
 at its newest stable version on 2026-09-17.

How outcomes were assigned:

- Crates reviewed by hand carry the outcome written in this vet's review map,
   `~/temp/agent/hashvet2-2026-09-17/scripts/review.json`.
- A crate whose runtime dependency is itself an accelerated hash crate is a consumer,
   not a hash library of its own;
   the underlying crate carries the screening outcome.
- Crates with intrinsic matches for both architectures or a SIMD abstraction,
   and not reviewed as hash libraries,
   were read by description and source layout and are category mismatches.
- Every other crate had intrinsic matches for at most one architecture,
   no SIMD abstraction,
   and no accelerated dependency in its non-test Rust sources,
   which fails HC1 regardless of category.

- `a3s` 0.15.16:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `a3s-acl` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `a3s-boot` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `a3s-deep-research` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `a3s-power` 0.9.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `a3s-sentry` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `a3s-use` 0.3.12:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `a3s-vec` 0.1.0:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aarch64-dit` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ab-blake3` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ab-merkle-tree` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `abcrypt` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `abootcrafter` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `abrute` 0.1.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `abtc-domain` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `accelerated-crypto` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `accroitre` 0.2.4:
   category mismatch: consumer of `blake3`, `xxhash-rust` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `accumulators` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `acdi` 0.5.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `acdp` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `acdp-crypto` 0.13.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `actix-csrf-middleware` 0.9.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `actix-error` 0.2.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `actix-error-derive` 0.2.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `actix-tls` 3.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `acton-service` 0.43.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `actr-framework-protoc-codegen` 0.4.21:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `adamo-sys` 0.1.104:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `adaptive-pipeline` 2.0.0:
   category mismatch: consumer of `crc32fast`, `ring`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `adaq-trading-crypto` 1.1.2:
   category mismatch: consumer of `sha2`, `sha3`, `openssl` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `addchain` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `addresshashing` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `adguard-flm` 2.6.2:
   category mismatch: consumer of `blake3`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `adhammer-secrets` 1.5.1:
   category mismatch: consumer of `aes`, `md-5`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `adk-awp` 2.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `adk-deploy` 2.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `adk-server` 2.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `advmac` 1.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `advmac-rs` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aead` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aegis` 0.9.16:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `aegis_vm` 0.2.52:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aegis-core-pqc` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aegis-crypto` 0.1.5:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aegis-password-generator` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aegis-ratchet` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aegis-vault-pqc` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aegisvault` 0.5.5:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aeonflux` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aerorsync` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aerovault` 0.6.4:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aerovault-cli` 0.6.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aes` 0.9.3:
   block cipher providing the AES-NI, VAES, and aarch64 AES backends for the CMAC and PMAC
   candidates; not a hash itself.
- `aes_frast` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aes-ccm` 0.5.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aes-gcm` 0.11.1:
   category mismatch: consumer of `aes`, `ghash` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aes-gcm-siv` 0.12.1:
   category mismatch: consumer of `aes`, `polyval` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aes-keywrap` 0.9.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aes-kw` 0.3.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aes-prng` 0.2.2:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aes-siv` 0.8.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aes-soft` 0.99.99:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aes-stream` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aes-wasm` 0.1.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aes256ctr_poly1305aes` 0.2.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aescrypt-rs` 0.2.0-rc.11:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aesni` 0.99.99:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aethel-core` 0.6.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aetheris-encoder-bitpack` 0.2.24:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aetheris-encoder-serde` 0.2.24:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aetheris-protocol` 0.2.24:
   category mismatch: consumer of `twox-hash` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aethershell` 12.0.2:
   category mismatch: consumer of `sha2`, `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `affidavit` 26.6.22:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `affinidi-data-integrity` 0.7.12:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `affinidi-messaging-core` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `affinidi-messaging-didcomm-service` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `affinidi-messaging-didcomm-v1` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `affinidi-sd-jwt` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `affinidi-tsp` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `afi-cli` 0.24.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `afterburner-core` 0.2.8:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `afterburner-ignite` 0.2.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `afterburner-node-compat` 0.2.8:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `afterburner-thrust` 0.2.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `afterburner-wasi` 0.2.8:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ag_id` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `age` 0.12.1:
   category mismatch: consumer of `aes`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `agent-block` 0.38.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `agent-doc` 0.34.65:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `agent-phone` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `agent-publish` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `agent-store` 0.1.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `agent-top` 0.18.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `agentic-coding-protocol` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `agentic-identity` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `agentix` 0.29.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `agentknock` 0.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `agentmesh` 4.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `agentplane` 0.38.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `agentsync` 1.49.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ahash` 0.8.12:
   HC2 and HC3: 64-bit `Hasher` output documented as unstable across versions and platforms.
- `ahl-mirror` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ahsah` 2.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ai-memory` 0.10.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ai-model-vault` 4.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ai-usagebar` 1.18.1:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ai2070-net` 0.18.0:
   category mismatch: consumer of `blake3`, `xxhash-rust` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `aicommit` 0.1.143:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aignt-solana-rpc` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aigw-openai` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aion-package` 0.30.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `airframe_crypt` 1.0.2:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aivault` 0.1.38:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alat-rs` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `aleph-types` 0.19.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `alfred-crates` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alfred-kill-process` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alfred-qr` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alfred-thesaurus` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alibabacloud` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alice-crypto` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aligned-cmov` 2.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alint` 0.16.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alint-core` 0.16.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alint-dsl` 0.16.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alint-lsp` 0.16.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alint-output` 0.16.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alint-rules` 0.16.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alkali` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `all-smi` 0.26.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-chains` 0.2.39:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-core` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-dyn-abi` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-json-abi` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-merkle-tree` 0.7.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-monad-evm` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-primitives` 1.7.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alloy-pubsub` 2.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-rlp` 0.3.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-rlp-derive` 0.3.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-signer-gcp` 2.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-sol-macro` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-sol-macro-expander` 1.7.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alloy-sol-macro-input` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-sol-type-parser` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `alloy-sol-types` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `allthehashes` 0.1.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `altcha` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `altcha-lib` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `altcha-lib-rs` 0.3.3:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `alterion-encrypt` 1.9.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `alula-wasm32-unknown-unknown-openbsd-libc` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `am-fs-btrfs` 0.6.2:
   category mismatch: consumer of `blake2b_simd`, `sha2`, `twox-hash` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `am-fs-erofs` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `amacs` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `amaru-curve25519-dalek` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `amaru-vrf-dalek` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `amaters-net` 0.2.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ambient_profiling_procmacros` 1.0.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `amiss` 0.29.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `analiticcl` 0.4.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `anamnesis-core` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `anasemble-core` 0.1.0-rc.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `angulu` 0.1.3:
   category mismatch: consumer of `crc32fast`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `angulu-rs` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `animsmith` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `animsmith-engine` 0.14.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `anker_solix` 0.4.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ann-search-rs` 0.8.5:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `anodizer-stage-checksum` 0.28.1:
   category mismatch: consumer of `blake3`, `crc32fast`, `md-5`, `sha1`, `sha2`, `sha3` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `anomstream` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `anomstream-core` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `anomstream-hotpath` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `anomstream-triage` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `anonymous-credit-tokens` 0.4.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `anp-identity` 0.2.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ansi-x963-kdf` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ansible-vault` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ant-node` 0.19.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ant-quic` 0.27.52:
   category mismatch: consumer of `aws-lc-rs`, `blake3`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `antelope` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `anthropic-agent-sdk` 0.2.75:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `antlr-rust-codegen` 0.34.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `antlr-rust-runtime` 0.34.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `anubis-rage` 1.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `anvil-ssh` 1.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `anychain-ethereum` 0.1.41:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `anyllm_batch_engine` 0.16.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `anytls` 0.3.17:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ap-proxy-client` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ape-decoder` 0.3.2:
   category mismatch: consumer of `crc32fast`, `md-5` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `aperion-shield` 1.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `apex-sdk-types` 0.1.6:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `apex-solver` 1.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `api_claude` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `api-bones` 6.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `api-error` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `api-error-derive` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `api-keys-simplified` 0.5.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `api-signature` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `apotheca` 0.3.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `appattest` 0.1.1:
   category mismatch: consumer of `aws-lc-rs`, `openssl` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `appcore-dnt` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `appcore-gateway` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `appimageupdate` 0.4.1:
   category mismatch: consumer of `blake3`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `apple-cryptokit-rs` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `applite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `apr` 0.4.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aprender-registry` 0.67.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `apt-parser` 1.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aptos-bcs` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aptos-crypto` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `aqua-template-registry` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arabic_text_utils` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arbitrary-type` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arcanum-hash` 0.1.2:
   category mismatch: consumer of `blake3`, `ring`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `arcanum-primitives` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arcbox-boot` 0.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `arche` 4.17.3:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `architect-api` 11.6.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `architect-derive` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `architect-sdk` 11.6.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `archive-it-client` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `archkeep-rule-sdk` 0.30.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arcly-http-macros` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arcnow-sdk` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arctgz` 0.9.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `areion` 0.1.0:
   HC1 and HC7: x86-only AES-round hash, last release 2023.
- `arete-hash` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `argon_cipher` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `argon2` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `argon2-rust` 1.1.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `argon2rs` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `argonautica` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `argone-signing` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `arithmetic` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-algebra-test-templates` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ark-bls12-377` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-bls12-377-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-bls12-381` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-bls12-381-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-bn254` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-bw6-761` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-bw6-761-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-crypto-primitives` 0.6.0:
   category mismatch: consumer of `blake3`, `sha2`, `ahash` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `ark-curve25519` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ec` 0.6.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ark-ec-zypher` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bls12-377` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bls12-377-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bls12-381` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bls12-381-bandersnatch` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bls12-381-bandersnatch-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bn254` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-bw6-761` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ed-on-cp6-782` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ff` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ff-asm` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-ff-macros` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-groth16` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-grumpkin` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-mnt4-753` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-mnt6-753` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-models-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-pallas` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-pallas-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-poly` 0.6.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ark-poly-commit` 0.6.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ark-r1cs-std` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-relations` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-secp256k1` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-secp256r1` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-serialize` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-serialize-derive` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-snark` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-sponge` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-srs` 0.3.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ark-std` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-test-curves` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-transcript` 0.0.6:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ark-vesta` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-vesta-ext` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ark-vrf` 0.5.3:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `arkeion` 0.13.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `arkhe-rand` 0.15.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `arkworks-mimc` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arrow-avro` 60.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `arrow-digest` 59.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arrowmetal` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arrowmetal-sys` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arti` 2.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `arti-client` 0.46.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ascent-interpreter` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ascon` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ascon-core` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ascon-hash` 0.4.0:
   HC1: scalar Ascon permutation.
- `ascon-hash256` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ascon-rs` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ascon-xof128` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `asic-rs` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `asm330lhh-rs` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `asm330lhhx-rs` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `asmcrypto` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `asry` 0.1.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `assay-adapter-api` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-canonical` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-core` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-evidence` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-mcp-server` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-metrics` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-policy` 6.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `assay-registry` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-runner-core` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `assay-runner-linux` 6.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `assay-runner-schema` 6.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `assay-sim` 6.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `asset_procmac` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `assetpack-core` 0.3.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `assetpack-transform-precomp2` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `astraguard` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `astrid-audit` 2026.9.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `astrid-crypto` 2026.9.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `astroframe` 0.2.2:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `astrs-rtps` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `asupersync` 0.5.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `asx-rs` 0.14.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `async_bagit` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `async_sync_trait_procmacro` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `async-hash` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `async-native-tls` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `async-signatory` 1.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `async-signature` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `async-snmp` 0.18.1:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `md-5`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `async-tls` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ate` 1.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `atl-core` 0.30.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `atlas-c2pa-lib` 0.1.2:
   category mismatch: consumer of `openssl`, `openssl-sys`, `ring` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `atlas-common` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `atlas-keccak-hasher` 3.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `atlas-rs` 0.2.0:
   category mismatch: consumer of `sha2`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `atlas-sha256-hasher` 3.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `atproto-record` 0.14.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `atrep` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `atrium-crypto` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `attest-ledger-types` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `attrctl` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `attrouter` 0.1.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `audiofp` 0.4.3:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `audit-trail` 1.0.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aur-scanner-cli` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aur-scanner-core` 0.1.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `auria-security` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `aurora-engine-modexp` 1.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aurora-engine-precompiles` 2.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aurora-engine-sdk` 2.0.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aurora-engine-transactions` 1.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `aurora-engine-types` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `auth-framework` 0.4.3:
   category mismatch: consumer of `aws-lc-rs`, `crc32fast`, `ring`, `sha1`, `sha2` rather than
   a hash library of its own; the underlying crate carries the screening outcome.
- `auth-tarball-from-git` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `authnz-totp` 5.6.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `authoscope` 0.8.1:
   category mismatch: consumer of `md-5`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `authrs` 0.1.2:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `automake-oracle-rs` 0.1.23:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `autumn-cli` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `autumn-macros` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `autumn-web` 0.7.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `avdumpr` 0.1.0:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha1`, `sha2`, `sha3` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `ave-identity` 0.3.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `avro-rs` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `avz` 0.1.2:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `awiki-im-core` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `awol2005ex3-kerberos-crypto` 0.4.2:
   category mismatch: consumer of `aes`, `md-5` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `aws-esdk` 1.2.4:
   category mismatch: consumer of `aws-lc-rs`, `aws-lc-sys` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `aws-lc-rs` 1.18.1:
   HC4: builds AWS-LC C sources with `cmake` or prebuilt bindings.
- `axhash-core` 1.0.0:
   HC2: 64-bit output.
- `axiam-opaque` 1.0.0-beta15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `axiam-sdk` 1.0.0-beta15:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `axiom-core` 2.0.13:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `axiom-eth` 0.4.3:
   category mismatch: consumer of `blake3`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `axiom-query` 2.0.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `axon-csys` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `axtra` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `axum-auth` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `axum-gate` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `axum-route-macros-procmacros` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `az-snp-vtpm` 0.8.2:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `azure_security_keyvault_certificates` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `azure_storage_queue` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `azure-guest-attestation-sdk` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `azure-tpm` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `b2sum` 0.4.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `b2sum-rs` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `b2sum-rust` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `b3sum` 1.8.7:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `b4ae` 2.1.3:
   category mismatch: consumer of `ring`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `b58` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `babyjubjub-ec` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `backbeat` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `backtestlite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `backup-deduplicator` 0.3.0:
   category mismatch: consumer of `sha1`, `sha2`, `xxhash-rust` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `baddie-csprng` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `badtouch` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `bagr` 0.3.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `baid58` 0.4.4:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bal-layout` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `balloon-hash` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `balq` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `baltic-id` 0.0.2:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bamboo-agent` 2026.9.19:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bambu-rs` 0.1.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bancho-packets` 5.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `banditdb` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bare-metal-evm-keccak` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barretenberg-rs` 5.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barter` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barter-data` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barter-execution` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barter-instrument` 0.3.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barter-integration` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `barter-macro` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `base-d` 3.0.36:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `base16ct` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `base32ct` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `base58ck` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `base64-ng-password` 2.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `base64ct` 1.8.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `base64url` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `basecrawl` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `basecrawl-core` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `basecrawl-ffi` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `basecrawl-fp` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `basecrawl-proof` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `basecrawl-render` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `basecrawl-seal` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bash-hash` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bash-prg-hash` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `batch-mode-batch-schema` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `batpak` 0.10.0:
   category mismatch: consumer of `blake3`, `crc32fast` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `bbcloud` 0.21.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bbs` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bc-components` 0.31.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bc-crypto` 0.14.0:
   category mismatch: consumer of `crc32fast`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `bc-shamir` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bcinr-powl` 26.7.28:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bcinr-powl-receipt` 26.7.28:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bcrypt-pbkdf` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bcs` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bdstorage` 1.0.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `beads_rust` 0.6.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `beamdb` 0.19.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bech32` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bee-crypto` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `beekem` 0.3.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `beetswap` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bellbook` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bellperson-sha512` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `belt-block` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `belt-hash` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `belt-mac` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-c14n` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-core` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-crypto` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-dsig` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-enc` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-keys` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-pkcs12` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-transforms` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bergshamra-xml` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bevy_carnage` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bevy_sdf_klown` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bevy_skein` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bevy_sonus` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bevy_symbios_multiuser` 0.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bgpkit-broker` 0.12.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bhc-data-structures` 0.2.22:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `big-hash` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bigtable_rs` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `binary-ff1` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bindcar` 0.7.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bioassert` 5.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bioformats` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bip0039` 0.14.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bip157` 0.6.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bip32` 0.5.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bip39` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bip39-dict` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bishop` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bita` 0.15.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitbelay-providers` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitbelay-report` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcheck` 0.2.8:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bitcoin` 0.32.102:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin_hashes` 1.2.0:
   screening survivor for SHA-256 (x86 SHA-NI, AVX2, SSE4.1 and aarch64 sha2 with run-time
   detection), streaming `HashEngine`, CC0-1.0.
- `bitcoin_hd` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin_onchain` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin_scripts` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-addr` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-address-book` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-bench` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-block` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-blockencoding` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-blockpolicy` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-bosd` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-chacha` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-coincontrol` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-consensus-encoding` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-crypter` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-dns` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-dogecoin` 0.32.7-doge.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-golombrice` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-hash` 0.1.20:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-hdchain` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-hmac-sha256` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-hmac-sha512` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-hpke` 0.13.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bitcoin-indexed-chain` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-internals` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-key` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-merkle` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-miner` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-muhash` 0.1.20:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-net` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-poly1305` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-private` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-random` 0.1.21:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-ripemd` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-script` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-scripting` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-secp256k1` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha1` 0.1.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha256` 0.1.20:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha256-avx2` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha256-hkdf` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha256-shani` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha256-sse4` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha256-sse41` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha3` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-sha512` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-signingprovider` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-siphash` 0.1.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-tools-core` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bitcoin-tx` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-u256` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoin-units` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoincash` 0.32.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoincore-rpc` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoincore-rpc-async` 3.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoincore-rpc-json` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoincore-rpc-json-async` 3.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoind-async-client` 0.15.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoinleveldb-hash` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoinleveldb-versionedit` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoinsecp256k1-group` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoinsecp256k1-keys` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoint4_hashes` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitcoinwallet-library` 0.1.16-alpha.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bitdroid` 0.7.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bitrep` 0.5.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bity-ic-icrc3` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bitz` 3.4.27:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `black-bagg` 0.4.10:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `black-bagg-sigs` 0.4.10-qsig.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blahaj` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake-hash` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2` 0.11.0:
   HC1: the `simd.rs` module is a scalar four-word abstraction that relies on
   auto-vectorization, with no architecture intrinsics.
- `blake2_bin` 1.0.5:
   category mismatch: consumer of `blake2b_simd`, `blake2s_simd` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `blake2_c` 0.3.3:
   HC4: deprecated wrapper for the BLAKE2 C implementation.
- `blake2_ce` 0.10.6:
   HC1: portable implementation with no aarch64 intrinsics.
- `blake2-mac` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2-rfc` 0.2.18:
   HC1: portable implementation.
- `blake2-rfc_bellman_edition` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2b` 0.99.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2b_halo2` 0.2.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blake2b_simd` 1.0.5:
   HC1: AVX2 and SSE4.1 kernels on x86 only.
- `blake2b-192` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2b-pow` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2b-ref` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2b-rs` 0.2.0:
   HC4: bindings to the official BLAKE2b C implementation.
- `blake2b256-balloon` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2s` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2s_const` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2s_simd` 1.0.5:
   HC1: AVX2 and SSE4.1 kernels on x86 only.
- `blake2s-cli` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2x` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake2ya` 1.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blake3` 1.8.7:
   screening survivor with a build note: Rust SSE2, SSE4.1, and AVX2 kernels; AVX-512 and NEON
   only through C and assembly compiled by `build.rs` (`c/blake3_neon.c`,
   `c/blake3_avx512_x86-64_unix.S`), so a rustc-only build has no aarch64 vector path.
- `blake3_aead` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blake3_cipher` 0.1.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blake3_enc` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blake3_merkle` 0.0.6:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blake3-pow` 1.0.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blake3-std` 0.1.0:
   HC1 and HC3: BLAKE3 through nightly `std::simd` with no aarch64 path of its own; last
   release 2023.
- `blake512-hash` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blakeout` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blash` 0.1.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blasthttp` 0.10.0:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blazehash` 0.2.5:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3`, `xxhash-rust`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `blazehash-core` 0.2.5:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3`, `xxhash-rust`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `blend-contract-sdk` 2.25.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blind-rsa-signatures` 0.17.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bliss-crypto` 0.1.1:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `block-buffer` 0.12.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `block-cipher` 0.99.99:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `block-padding` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blockbucket` 0.2.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blockchain-cli` 1.1.2:
   category mismatch: consumer of `sha2`, `twox-hash` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `bloom-server` 1.39.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blossom-rs` 0.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `blot` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blot-lib` 0.1.2:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `blowfish` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bls_bulletproofs` 1.1.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bls_ringct` 1.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bls-signatures-rs` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `blsful` 4.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `blst` 0.3.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blstrs` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `blueprint-crypto-hashing` 0.2.0-alpha.7:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `blueprint-webhooks` 0.2.0-alpha.12:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `blvm-sdk` 0.1.17:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bma-benchmark` 0.0.24:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bmw-hash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bn` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bn-plus` 0.4.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bn254_hash2curve` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bnum` 0.14.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `boatramp-http` 0.4.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `boha` 0.19.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `boltffi` 0.30.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `boltffi_cli` 0.30.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `boltz-client` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `boohashing` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `boojum` 0.32.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `boolnetevo` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bore-cli` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `boring` 5.2.0:
   category mismatch: consumer of `boring-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `boring-imp` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `boring-rustls-provider` 5.0.1:
   category mismatch: consumer of `boring`, `boring-sys` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `boring-sys` 5.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `boring2` 4.15.15:
   category mismatch: consumer of `boring-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `boringauth` 0.9.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `botan` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `boxdd-sys` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bp256` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bp384` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bqtools` 0.5.14:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `br-crypto` 0.4.8:
   category mismatch: consumer of `openssl`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `brazen` 0.0.18:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `breakmancer` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brevo` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brine-ed25519` 0.9.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `brk_structs` 0.0.111:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `broadcast-auth` 0.3.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `brokk-anvil` 0.28.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `brokk-bifrost-semantic-packs` 0.11.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bromberg_sl2` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brute-force` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brutecraber` 0.9.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `brynja` 0.20.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brynja-core` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brynja-crypto` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `brynja-crypto-cpu` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bssh-russh` 0.63.1:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `ghash`, `keccak`, `polyval`, `ring`,
   `sha1`, `sha2`, `sha3` rather than a hash library of its own; the underlying crate carries
   the screening outcome.
- `bsv` 2.1.1:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bsv-middleware-cloudflare` 0.3.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bsv-primitives` 0.5.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bsv-rs` 0.3.24:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `bsv-sdk` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bsv-wallet-toolbox-rs` 0.3.66:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `bsv-wasm` 1.3.4:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `btrfs-core` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `btrfs-mkfs` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `buildfix-hash` 0.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bulletproof-kzen` 1.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bulletproofs` 5.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `bun_hash` 0.1.0:
   category mismatch: consumer of `twox-hash` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `burn_depth` 0.4.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `burn_siglip2` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `burr` 0.29.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `buup` 0.25.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `bxx` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `c2-chacha` 0.3.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `c255b3` 0.0.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `c2pa-html` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `c2pa-structured-text` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `c32` 0.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `c4id` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cacache` 13.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cacache-sync` 11.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cachekit-core` 0.6.0:
   category mismatch: consumer of `aes`, `sha2`, `xxhash-rust`, `ring` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `cachekit-rs` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `caden` 0.9.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cairn` 0.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cairn-core` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `caligula` 0.5.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `callghost` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `camel-bridge` 0.48.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `camel-component-cxf` 0.48.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `camel-component-jms` 0.48.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `camellia` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `camo-rs` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `canic-cli` 0.110.22:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `canon-mcp` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `canonical` 0.7.1:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `canton-solvency-merkle` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `canton-solvency-report` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `caplite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `captrack` 0.1.1:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `captrack-macros` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `capycrypt` 0.7.5:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `car-memgine` 0.54.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `car-sync` 0.54.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `carbon-curve25519-dalek` 3.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `carbon14` 0.3.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `carbonado` 0.7.1:
   category mismatch: consumer of `aes`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `cardano-crypto` 1.0.8:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cardanowall` 0.12.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cardanowall-cli` 0.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cardinality-estimator-safe` 4.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-cook` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-espflash` 4.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-export` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-hack` 0.6.45:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-ledger` 1.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-llvm-cov` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-maintained` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-minimal-versions` 0.1.37:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-mtime` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-pmcp` 0.24.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cargo-rail` 0.28.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `cargo-rapk` 0.22.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-resources` 1.4.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cargo-rullst` 12.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-show-asm` 0.2.63:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cargo-tangle` 0.3.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cartog-indexer` 0.34.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cas-kit` 0.2.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cas-lib` 0.2.89:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cascade-cli` 0.1.152:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cashu` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cashweb-secp256k1` 0.19.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `casper-client` 5.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `casper-eip-712` 1.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cast5` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cat198x` 0.5.3:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `catalysh` 0.9.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cavp` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cavs-hash` 1.7.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cb2vec` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cbc` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cbc-mac` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cbdr` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cc-check` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cc-downloader` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cc-me` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ccalc` 0.48.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ccl-fxhash` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cclang` 0.4.0:
   category mismatch: consumer of `sodiumoxide` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ccm` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ccnext-abi-encoding` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ccsum` 0.2.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `xxhash-rust` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `cdk-cli` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cdr-encoding-size` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cedar-local-agent` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cedar-policy-cli` 4.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `celers-core` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `celers-protocol` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cellos-core` 0.7.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cellrune` 0.1.20:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `centipede` 0.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cerbero` 0.0.20:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cert_by_host` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cert-dump` 3.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cert-helper` 0.5.2:
   category mismatch: consumer of `openssl`, `openssl-sys` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `cesride` 0.6.4:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `cess-sha2raw` 0.1.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `cf-gears-credstore` 0.5.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cf-gears-rustls-corecrypto-provider` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cf-gears-toolkit-stable-hash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cfb-mode` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cfb8` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cfe_progmacro` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cfr_protocol` 1.0.0-rc.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chacha12` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `chacha12-blake3` 0.9.10:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chacha12blake3` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `chacha20` 0.10.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `chacha20-blake3` 0.10.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `chacha20-poly1305` 0.2.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `chacha20-poly1305-aead` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chacha20poly1305` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chacha20poly1305-nostd` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chain-signatures-solana-program` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chainlink-tracker` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `challenge_response` 0.5.46:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chameleon-pq` 0.8.5:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `champ-trie` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chaotic_semantic_memory` 0.3.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chartjs_image` 6.1.111:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `checkpipe` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `checksum_dir` 1.0.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `chf` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sdk-client` 0.36.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chia-sdk-coinset` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sdk-derive` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sdk-driver` 0.36.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `chia-sdk-signer` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sdk-test` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sdk-types` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sdk-utils` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chia-sha2` 0.49.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chia-wallet-sdk` 0.36.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chicago-tdd-tools` 26.8.9:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chie-crypto` 0.2.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chik-sha2` 0.49.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chio-attest-buyer-core` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chio-attest-loopback` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chio-commerce-order` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `chio-core` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `chio-core-types` 0.1.2:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `chio-pheromone` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `chio-weights` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `chisel-extract` 1.0.2:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `chksum` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-cli` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-core` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-core` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-md5` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-sha1` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-sha2` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-sha2-224` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-sha2-256` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-sha2-384` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-hash-sha2-512` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-md5` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-reader` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-sha1` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-sha2` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-sha2-224` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-sha2-256` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-sha2-384` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-sha2-512` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chksum-writer` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-acp` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-ai-protocols` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-blockchain` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-client-core` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-content` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `choreo-daemon` 0.2.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `choreo-im` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-image` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-keystore` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `choreo-markdown` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-mcp` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-power-events` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-proto` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-sanitize` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-sockreg` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-transport` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreo-tui` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `choreographr` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chromium-safestorage-core` 0.1.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `chromium-safestorage-forensic` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chrony-rs-core` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `chunkrs` 0.9.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `churl` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `churust-core` 0.3.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ciftl-rs` 0.1.1:
   category mismatch: consumer of `crc32fast`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `cipher` 0.5.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cipher_password` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cipher-crypt` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ciphera-tessera` 0.0.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ciphern` 0.2.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `circom-scotia` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `circom-witness-rs` 0.5.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `circuit_definitions` 0.153.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cita_trie` 6.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cita-hasher` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `citadeldb` 2.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `citadeldb-crypto` 2.5.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cjson` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ck-meow` 0.1.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ckb-opt-blake2b` 0.1.4:
   HC4: C sources for CKB-VM.
- `ckb-opt-fips202` 0.1.6:
   HC4: C sources for CKB-VM.
- `ckb-opt-sha256` 0.1.0:
   HC4: C sources for CKB-VM.
- `ckb-opt-sha512` 0.1.0:
   HC4: C sources for CKB-VM.
- `claim-ledger` 0.2.1:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `clap-digest` 0.3.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `clap-repl` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `clarity-vm` 2.3.0:
   category mismatch: consumer of `sha2-asm` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `classic-mceliece-rust` 3.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `classified_aes` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `clatter` 2.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `claude-code-sdk-rust` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `claude-manager` 0.16.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `claude-rust-config` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `claude-rust-errors` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `claude-rust-types` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `claudy` 0.9.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `clausura-cli` 1.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `clausura-core` 1.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `claux` 20260909.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `clawspec-core` 0.4.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `clear_on_drop` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cleitonq` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `clia-macaddr` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cljrs-blake3` 0.2.16:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `clmul` 0.8.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `clock-rand` 1.0.3:
   category mismatch: consumer of `aes`, `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `clone-solana-keccak-hasher` 2.2.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `clone-solana-sha256-hasher` 2.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `clonic` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cloud-sdk` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cloud-sdk-hetzner` 1.0.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `clsx` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `clvkr` 0.20.0:
   category mismatch: consumer of `sha1`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `clvmr` 0.20.0:
   category mismatch: consumer of `sha1`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cm-email-webhook-verification` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cmac` 0.8.0:
   screening survivor with `aes`: AES-CMAC, 128-bit native output, streaming `Mac::update`,
   NIST SP 800-38B.
- `cmac_rust` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cmacro` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cmacros` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cmn` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cmn-substrate` 0.4.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cmov` 0.5.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cmpv2` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cms` 0.2.3:
   category mismatch: consumer of `aes`, `sha1`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `cmt` 0.5.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cmtree` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cmz` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cnfy-secp256k1` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cnfy-uint` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cniguru` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cobre-stochastic` 0.15.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cocoon` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cocoon-tpm-crypto` 0.1.4:
   category mismatch: consumer of `aes`, `sha1`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `codeskeleton` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `coding-agent-search` 0.7.0:
   category mismatch: consumer of `blake3`, `crc32fast`, `ring`, `sha2`, `xxhash-rust`,
   `openssl` rather than a hash library of its own; the underlying crate carries the screening
   outcome.
- `cognitum-gate-tilezero` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `coins-bip32` 0.13.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `coins-bip39` 0.13.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `coins-core` 0.13.2:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `coins-ledger` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `colorhash` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `commit_verify` 0.12.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `commit-reveal` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `common-access-token` 0.2.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `common-crypto` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `commoncrypto` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `commoncrypto-sys` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `commucat-ledger` 1.0.103:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `communitas-core` 0.12.4:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `composer-install` 0.2.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `computer-use-linux` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cona` 0.0.26:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `concat-kdf` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `concord` 2.5.21:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `concrete-csprng` 0.4.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `confers` 0.5.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `confidential-script-lib` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `confium-node` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `confium-signatif` 0.10.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `conserve` 24.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `consistenttime` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `const-crypto` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `const-oid` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `const-sha1` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `const-siphasher` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `constant_time_eq` 0.6.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `constduck-procmacro` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `content_scan` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `content_scan_proc_macro` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `content-addressable` 0.1.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cookie-scoop` 0.1.1:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cookie-scoop-cli` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cooklang-sync-client` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `copia` 0.3.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cord` 2.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `corduit` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `core-rpc` 0.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `core-rpc-json` 0.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `coreason-manifest` 0.97.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `coreutils` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `corgi-build` 0.1.75:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `corteq-onepassword` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `corvid_files` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `corvid_hash` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cose_minicbor` 0.1.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cose-rust` 0.1.8:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cose-rust-plus` 0.2.0:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cose2` 0.5.0:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `coset` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cosine-lsh` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cosmian_cover_crypt` 16.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cosmian_crypto_base` 2.1.2:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cosmian_crypto_core` 11.0.0:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `cosmian_kms_csi_provider` 5.27.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cosmian_kms_k8s_operator` 5.27.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cosmian_kms_k8s_plugin` 5.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cosmian_openssl_provider` 1.0.0:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cosmian_rust_curve25519_provider` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cosmos-sdk-proto` 0.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cougr-core` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `courierust` 1.0.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cow-sdk-app-data` 0.1.0-alpha.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cow-sdk-signing` 0.1.0-alpha.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cowprotocol` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cowprotocol-appdata` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cowprotocol-orderbook` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cowprotocol-primitives` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cowprotocol-signing` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cpufeatures` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cpuid-bool` 0.99.99:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cquill` 0.0.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crabapple` 0.4.7:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `crabcrypt` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crabgraph` 0.3.3:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `crack_yearn_md5` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `crankx` 0.2.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cratestack-core` 0.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cratestack-exec` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cratestack-mock-wiremock` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cratestack-redis` 0.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `craton-hsm` 0.10.0:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `crazy-deduper` 0.2.2:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `crc-fast` 1.10.0:
   HC2: CRC-16, CRC-32, and CRC-64 outputs.
- `crc32-v2` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crc64fast` 1.1.0:
   HC2: 64-bit CRC.
- `crc64fast-nvme` 1.2.1:
   HC2: 64-bit CRC.
- `credx` 0.2.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crev-recursive-digest` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crevsum` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crmf` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cros-codecs` 0.0.6:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `cros-codecs-extended` 0.0.5-extended.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `cros-codecs-generic-vaapi` 0.0.6-generic.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `crown` 0.26.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crown-bin` 0.26.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crown-derive` 0.26.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crown-jsasm` 0.26.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crozier` 0.0.47:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crrl` 0.9.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `crtx-context` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crtx-core` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crtx-ledger` 0.1.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crustywad` 0.9.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crustywad-cli` 0.4.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cry_sha256` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypt-io` 1.0.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crypt-sha512` 1.0.1:
   category mismatch: consumer of `aws-lc-sys`, `boring-sys`, `openssl-sys`, `sha2` rather than
   a hash library of its own; the underlying crate carries the screening outcome.
- `crypt3_rs` 0.1.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `cryptan` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptex` 2.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cryptimitives` 0.20.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cryptkit` 0.1.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crypto` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_api` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_api_blake2` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_api_chachapoly` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_bastion` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_box` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_kx` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto_proto` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `crypto_secretbox` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto-async-rs` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto-bigint` 0.7.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto-common` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto-ext` 9.0.0:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crypto-hash` 0.3.4:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crypto-hashes` 0.10.0:
   category mismatch: meta crate over `sha2`, `sha3`, and `md-5`.
- `crypto-keystore-rs` 0.2.2:
   category mismatch: consumer of `aes`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `crypto-mac` 0.11.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto-primes` 0.7.2:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `crypto-rsl` 0.1.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `crypto-seal` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `crypto-tests` 0.5.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crypto-vote` 0.4.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cryptocol` 0.19.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptographic-message-syntax` 0.28.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cryptography-rs` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `cryptohelpers` 2.0.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cryptoki` 0.12.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptoki-sys` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptokit-rs` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptolib` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptonight-hash` 0.1.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cryptos` 0.0.1-alpha.6-patch.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptotools` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cryptox` 1.0.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cryptoxide` 0.6.5:
   screening survivor for SHA-256 (x86 AVX and SSE4.1 message-schedule kernels, aarch64 sha2
   intrinsics, compile-time selection); SHA-512, SHA-1, BLAKE2, and Keccak paths are
   single-architecture (HC1).
- `cryptraits` 0.14.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `crystals-dilithium` 2.0.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `csaf-core` 1.5.22:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `csaf-crud` 1.5.22:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `csaf-models` 1.5.22:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cshake` 0.2.1:
   HC1: same `keccak` backends as `sha3`.
- `csum` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `csv-adapter-core` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ct-codecs` 1.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ct-merkle` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ctf_party` 0.2.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `cthash` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ctr` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cts` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ctutils` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cubby` 0.2.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cubehash` 0.4.1:
   screening survivor: CubeHash rev3 with AVX2, SSE2, and NEON backends selected by `cfg`,
   streaming `update`/`finalize`, MIT.
- `cubical` 0.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cuid2-rs` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cuid2-timeless` 0.1.6:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `cull-gmail` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curuam` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-dalek` 5.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-dalek-arcium-fork` 4.1.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-dalek-fiat` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-dalek-libpep` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `curve25519-dalek-ml` 4.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-dalek-ng` 4.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-dalek-v2` 4.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-elligator2` 0.1.0-alpha.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-entropic` 3.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `curve25519-parser` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `curvy-prover` 0.1.0-rc.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cvmfs` 0.4.2:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cw-merkle-tree` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cw20-atomic-swap` 0.11.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cx448` 0.1.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cxema` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cyanea-core` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `cyber-hemera` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cybergraph` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `cyclist` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `cyfs-sha2` 0.8.4:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `cyphergraphy` 0.3.1:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `daa` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `daa-prime-core` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `daa-prime-trainer` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `daemonic_error` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dahua-camera-http` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dalek-test-curve-docs` 4.0.0-pre.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dano` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `darkbio-crypto` 0.18.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dat` 4.7.0:
   category mismatch: consumer of `aes`, `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `databricks-tui` 0.35.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `databricks-zerobus-ingest-sdk` 2.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `datafusion` 55.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `datary` 0.3.0:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `datasketch-minhash-lsh` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dbl` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dbn-cli` 0.69.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dcbor` 0.25.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dcrypt` 4.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dcrypt-algorithms` 4.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dcrypt-hybrid` 4.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ddiff` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dealcode` 1.0.1:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `debug-derive` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `decaf377` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `decaf377_plus` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `decanter` 0.1.6:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `decanter-crypto` 0.1.6:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `decanter-derive` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dedupefs` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dedups` 0.1.0:
   category mismatch: consumer of `blake3`, `crc32fast`, `gxhash`, `sha1`, `sha2`, `twox-hash`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `degenbot-executor` 0.6.0-alpha.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dekopon-broker-host` 0.17.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dekopon-broker-protocol` 0.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dekopon-capability` 0.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dekopon-policy` 0.17.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `deldup` 1.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `delopay` 0.9.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `deobfuscate` 1.18.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `depit` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `depit-cli` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `deputy-core` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `deputy-crypto` 0.4.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `der` 0.8.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `der_derive` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `derec-cryptography` 0.0.3:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `derec-library` 0.0.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `derec-proto` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `deribit-base` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `derivation-path` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `des` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `descriptor-wallet` 0.10.2:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `device-signer` 0.1.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `devolutions-crypto` 0.10.2:
   category mismatch: consumer of `aes`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `devstuff` 0.0.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dexios` 8.8.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dexios-core` 1.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `df-crypto` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dhcplayer` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dia-hammer` 3.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `diaglite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `diem-crypto` 0.0.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `diesel-libsql` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `differential-engine` 0.11.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dig-coinstore` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dig-dht` 0.16.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dig-identity` 0.7.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dig-keystore` 0.13.2:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dig-merkle` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dig-nat` 0.21.2:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dig-nft` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dig-smt` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dig-social-profile` 0.7.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dig-tips` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dig2browser` 0.4.16:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `digest` 0.11.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digest_auth` 0.3.1:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `digest-access` 0.3.6:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `digest-buffer` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digest-hash` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digest-io` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digest-io-async` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digest-tool` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digest-writer` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digester` 0.1.0:
   category mismatch: consumer of `blake3`, `sha1`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `digestible` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `digestify` 0.4.0:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `dilithium-rs` 0.4.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `dimpl` 0.7.3:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `diqwest` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dircs` 0.3.0:
   category mismatch: consumer of `blake3`, `sha1`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `dirhash_fast` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dirscomp` 0.2.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dirsh` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `distributed` 4.12.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `distronomicon` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `div-int-procmacro` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `djangohashers` 1.8.4:
   category mismatch: consumer of `md-5`, `ring`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `djangors-sessions` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `djpass` 1.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dlccryptlib` 2.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dlms-cosem-rs` 0.1.0:
   category mismatch: consumer of `aes`, `ghash` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dlprotoc` 0.4.12+35.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dmrtd` 0.3.4:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `dns_relay` 1.6.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dns-update` 0.5.8:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `dns-update-lite` 0.5.9:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `do-not-cry` 1.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dock_merlin` 3.0.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `docker-image` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `docker-source-checksum` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `docling-rag` 1.55.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `docrypto` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `doe` 1.1.90:
   category mismatch: consumer of `aes`, `blake3`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `dotenvx-rs` 0.4.33:
   category mismatch: consumer of `sha2`, `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dotlock-bin` 1.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `double-ratchet-2` 0.3.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `doubleentry` 0.7.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dove-core` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `downify` 0.2.1:
   category mismatch: consumer of `sodiumoxide` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `downloader-http-rs` 1.0.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dpapi-core` 0.2.1:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `dpapi-forensic` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dpapi-offline` 0.1.2:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `dpp-calc` 0.20.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dpp-crypto` 0.20.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `drasi-source-grpc` 0.2.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `drasi-source-http` 0.2.11:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-analytics` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-attention` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-engine` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-fabric` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-gpu` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-math` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dreamwell-mesh` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-metaphors` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dreamwell-qttps` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dreamwell-quantum` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dreamwell-universe` 1.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `drillx` 2.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `drs-core` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `drtahash` 0.0.17:
   HC2 and HC3: 64-bit output keyed per map.
- `dryoc` 1.0.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `dsa` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dsc-rs` 0.19.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dset` 0.1.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dsfb-chemical-engineering-cuda` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dstu-core` 0.3.8:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `dt-p3-keccak-air` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dtg-credentials` 0.9.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dtool` 0.17.0:
   category mismatch: consumer of `aes`, `blake2b_simd`, `blake3`, `ring`, `sha2`, `sha3`,
   `twox-hash` rather than a hash library of its own; the underlying crate carries the
   screening outcome.
- `dtt` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dubp` 0.58.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ductus` 0.52.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dudect-bencher` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dufs` 0.46.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dumb-crypto` 3.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dup-crypto` 0.58.0:
   category mismatch: consumer of `blake3`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dupfinder` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `duplicate_destroyer` 0.0.8:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dupsonic` 0.2.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dusk-bls12_381` 0.14.2:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dusk-curves` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dusk-hades` 0.24.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dusk-jubjub` 0.15.2:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dusk-merlin` 4.0.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dusk-pki` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dusk-plonk` 0.22.1:
   category mismatch: consumer of `blake2b_simd`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `dusk-poseidon` 0.41.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dusk-safe` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dusk-schnorr` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dusk-vm` 1.7.0:
   category mismatch: consumer of `blake2b_simd`, `blake3`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `dwldutil` 3.1.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dwmac-my` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dwutil` 0.1.3:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dynamic-config` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `dynamite-orfs` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `dyolo-kya` 2.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `dysk` 3.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `e9571_file_lib` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `e9571_lib1` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eaglesong` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `earthsciio` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `easy_salt` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `easy-hash` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `easy-hasher` 2.2.1:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `EasyCrypto` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eax` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ebman` 0.38.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ec25519` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ecb` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eccoxide` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ecdsa` 0.17.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ecies` 0.2.11:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ecies_25519` 0.2.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ecies-ed25519` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ecies-ed25519-rev` 0.5.4:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ecies-ed25519-silene` 0.1.0:
   category mismatch: consumer of `ring`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ecpay` 0.3.0:
   category mismatch: consumer of `aes`, `md-5`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ecsimple` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ecvrf` 0.4.4:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ed` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed25519` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed25519-compact` 2.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed25519-dalek` 3.0.0:
   category mismatch: consumer of `keccak`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ed25519-dalek-bip32` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ed25519-dalek-blake2-feeless` 1.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ed25519-dalek-blake2b` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed25519-dalek-blake3` 1.0.11:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ed25519-dalek-fiat` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ed25519-dalek-hpke` 0.0.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed25519-dalek-v2` 2.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ed2k` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed448` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed448-goldilocks` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ed448-goldilocks-plus` 0.18.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `edcert` 9.0.1:
   category mismatch: consumer of `sodiumoxide` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `edgesentry-audit` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `edgesentry-rs` 0.1.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `edi-energy` 0.20.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `edirstat` 2.2.0:
   category mismatch: consumer of `ahash`, `crc-fast` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `eero-keel-core` 0.0.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eero-keel-enforce` 0.0.15:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `eero-keel-policy` 0.0.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eero-keel-record` 0.0.15:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `efficient-sm2` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ego-chat` 0.2.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ehl-fdh` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `eidetic-engine` 0.15.2:
   category mismatch: consumer of `blake3`, `ring`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `eip-152` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ekubo_sdk` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `elabs-k256` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `elara-crypto` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `elara-light-client` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `elara-pq-transport` 0.1.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `elara-record` 0.3.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `elara-smt` 0.1.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `elements-miniscript` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `elenchus-compiler` 0.15.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `elf2tab` 0.13.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `elicitation_derive` 0.11.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `elid` 0.4.51:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `elif-auth` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `elliptic-curve` 0.14.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `elliptic-curve-tools` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `embacle` 0.26.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `embacle-mcp` 0.26.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `embacle-server` 0.26.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `embassy-dshot` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `embed_it` 7.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `embed_it_macros` 7.1.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `embed_it_utils` 7.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eme2` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `emixcrypto` 0.6.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `emob-ocpp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `emojihash-rs` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `empyrean-sys` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `enc_file` 0.6.6:
   category mismatch: consumer of `blake3`, `sha2`, `sha3`, `xxhash-rust` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `enchanter` 0.1.4:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `enchantress` 0.1.12:
   category mismatch: consumer of `aes`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `encoding_derive_helpers` 1.7.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `encrust` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `encrypt` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `encryptable-tokio-fs` 0.1.7:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `encryptman` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `encryptman-keyring` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `encrypto_sha256` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `encypher-c2pa` 1.0.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `encypher-c2pa-cbor` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `encypher-c2pa-cli` 1.0.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `encypher-c2pa-core` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `encypher-c2pa-crypto` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `encypher-c2pa-formats` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `encypher-c2pa-trust` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `encypher-c2pa-validate` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `energy-api` 0.20.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `engram-lib` 0.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `enprot` 0.5.79:
   category mismatch: consumer of `sha3`, `botan` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `enr` 0.13.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `entro-hash` 1.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `entropy-api` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `entropy-auth` 2026.9.9:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `entrouter-universal` 0.9.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `envoy-cli` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `eore-api` 3.6.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eore-boost-api` 4.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `epcis-hash` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ephemeral-rollups-sdk` 0.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `equix` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ergotree-ir` 0.28.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `error-forge` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `esi` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `esp-emac` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `esp-hal-procmacros` 0.23.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `esp-p4-eth` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `espflash` 4.6.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `eternaltwin_hammerfest_store` 0.16.4:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `eth` 0.60.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eth-blockies` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eth-valkyoth-evm-core` 0.30.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eth-valkyoth-hash` 0.11.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `eth-valkyoth-verify` 0.27.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethabi-decode` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethdigest` 0.4.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ethdigest-macros` 0.2.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ethereum_hashing` 0.8.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ethereum_serde_utils` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethereum_ssz` 0.10.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethereum_ssz_derive` 0.10.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethereum-triedb` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `etherparse` 0.21.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-addressbook` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-contract` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-contract-abigen` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-contract-derive` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-core` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-derive-eip712` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-etherscan` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-middleware` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-providers` 2.0.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethers-signer-factory` 2.0.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ethers-signers` 2.0.14:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ethers-solc` 2.0.14:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ethnum` 1.5.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ethp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `etoon` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `etwin_hammerfest_store` 0.12.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `eui48` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `everscale-asm` 0.0.9:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `evidence` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `evidence-core` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `evm_ekubo_sdk` 0.6.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `evm-fork-cache` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `evmlite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `evpkdf` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ewf` 0.4.10:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ewf-forensic` 0.7.6:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ewf-image` 0.4.0:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `exception-collector` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `exceptionless` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `exchange-apiws` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `exonum_libsodium-sys` 0.0.24:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `exonum_sodiumoxide` 0.0.24:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `expunge` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `expunge_derive` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `extendhash` 1.0.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `externalities` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ez-hash` 1.1.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `ezcheck` 0.1.7:
   category mismatch: consumer of `md-5`, `ring`, `sha1`, `sha2`, `twox-hash` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `fabric-benchmarking` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fabric-metadata` 12.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fabric-support` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fabric-system` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `faest` 0.3.0:
   category mismatch: consumer of `aes`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `faiss-next` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `faker-rust` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `falcon-rs` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `falcon512_rs` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `falkordb` 0.10.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fallow-api` 3.27.0:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `fallow-cli` 3.27.0:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `fallow-config` 3.27.0:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fallow-core` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-engine` 3.27.0:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `fallow-extract` 3.27.0:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `fallow-graph` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-license` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-mcp` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-output` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-process` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-security` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-types` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fallow-v8-coverage` 3.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fash` 0.1.7:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `fast_rsync` 0.2.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fast-erasure-shake-rng` 0.3.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fast-md5` 1.0.0:
   HC1: its x86_64 and aarch64 cores are scalar unrolled MD5 rounds with no SIMD or dedicated
   instructions (src/aarch64.rs:16-20); MD5 also has no accelerated single-message path on
   either target.
- `fastcrypto` 0.1.11:
   category mismatch: consumer of `aes`, `sha2`, `sha3`, `twox-hash` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `fastcrypto-derive` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fasthash` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fasthash-sys` 0.3.2:
   HC4: C and C++ hash sources.
- `fasthash-sys-fork` 0.4.2:
   HC4: C and C++ hash sources.
- `fastmcp-client` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastmcp-core` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fastmcp-derive` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastmcp-protocol` 0.10.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fastmcp-rust` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastmcp-server` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastmcp-transport` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastmurmur3` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastnum` 0.7.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastpbkdf2` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fastsync` 0.10.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `faucet-auth` 1.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `faucet-common-kafka` 1.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `faucet-common-snowflake` 1.0.11:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `faucet-core` 1.12.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `faucet-kafka-common` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `faucet-sink-kafka` 1.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `faucet-source-gcs` 1.6.1:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `faucet-source-kafka` 1.3.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `faucet-source-s3` 1.8.1:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `faucet-stream` 1.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fci4096` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fclones` 0.35.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3`, `xxhash-rust` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `fcm_receiver_rs` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fcoreutils` 0.22.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fcrypt` 0.3.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fdedup` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fdh` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fdsum` 0.2.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `featherweight-runtime` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fedimint-arti-client` 0.20.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fedimint-hkdf` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `feff10-sys` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferogram-crypto` 0.6.5:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ferrijs-std` 0.5.0:
   category mismatch: consumer of `aws-lc-rs`, `crc32fast`, `keccak`, `md-5` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `ferritls-core` 0.8.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `ferritls-rustls` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferro-blob-store` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferro-maven-layout` 1.0.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ferro-whatsapp` 0.3.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrocrypt` 0.2.5:
   category mismatch: consumer of `openssl`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ferrocrypt-cli` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferroscope-receipt` 0.1.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrotherm` 0.44.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrotorch-hub` 0.6.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrox-sentinel` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrugocc` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrum-engine` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrum-interfaces` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrum-kernels` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrum-kv` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrum-models` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ferrum-quantization` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrum-sampler` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrum-scheduler` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrum-tokenizer` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ferrum-types` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fetch-data` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fetchr-integrity` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fhc` 0.12.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fib-quant` 0.1.0-beta.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fides` 4.3.5:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `fiffy` 0.1.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fig-sys-wasm32` 4.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `filament-cli` 0.8.5:
   category mismatch: consumer of `openssl-sys`, `ring`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `filament-crypto` 0.1.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `file_rw` 0.6.6:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `file-downloader` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `file-downloader-derive` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `file-hasher` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `filebuffer` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `filebyte` 4.5.3:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `filepack` 0.0.10:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `filesystem-mcp-rs` 0.1.25:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `xxhash-rust` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `filevault-core` 0.1.5:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `finance_enums` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `finch` 0.6.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `finch_cli` 0.6.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `finch_lib` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `find_duplicate_files` 0.28.0:
   category mismatch: consumer of `ahash`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `find-identical-files` 0.39.4:
   category mismatch: consumer of `ahash`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `fingerprint-lib` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fingerprint-struct` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `firebase-admin` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `firecrawl-pdfium` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `firedbg-rust-debugger` 1.74.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `firestore` 0.54.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `firmion-std-md5` 0.7.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fishtank` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flacenc-bin` 0.2.7:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `flaron-sdk` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flash-map` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flatbed` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flatbed_build` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flatbed_macros` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flatline-umac` 0.1.0:
   HC4: bindings to the UMAC C implementation.
- `fleek-blake3` 1.5.0:
   duplicate of `blake3` forked for Fleek; screened through `blake3`.
- `flexiber` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flexiber_derive` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flipperbit` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flodl` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flow-record-common` 0.4.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flow-record-derive` 0.4.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flowerpassword` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-arrow` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-audio` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-datafusion` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-derive` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-dns` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-hash` 0.0.6:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `flows-http` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-image` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-io` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-json` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-math` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-mdns` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-pubsub` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-rand` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-text` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flows-video` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flowstats` 0.1.2:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fluence-blake3` 1.5.0:
   duplicate of `blake3` forked for Fluence; screened through `blake3`.
- `fluent-hash` 0.2.4:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fluentbase-codec` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fluentbase-codec-derive` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fluentbase-crypto` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fluentbase-runtime` 1.5.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fluentbase-sdk` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fluentbase-sdk-derive` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fluentbase-sdk-derive-core` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fluentbase-types` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `flyer` 3.1.1:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fn-dsa` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fn-dsa-comm` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fn-dsa-kgen` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fn-dsa-sign` 0.4.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fn-dsa-vrfy` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-dtype` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-io` 0.3.0:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fnp-iter` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-linalg` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fnp-ndarray` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-python` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fnp-random` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-random-core` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-runtime` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnp-ufunc` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fnprint` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnv_rs` 0.4.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fnv64-rs` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `focl` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fog-crypto` 0.5.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fog-pack` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `foldhash` 0.2.0:
   HC1, HC2, and HC3.
- `foldhash-portable` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `foodshare-crypto` 1.3.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `forager-addr` 0.3.0:
   category mismatch: consumer of `blake2b_simd`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `forbidden-strings` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `forensic-hashdb` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `forge_hasher` 0.1.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `forge-guardrails` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `forgeh` 1.0.0-experimental:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `forjar` 1.31.0:
   category mismatch: consumer of `blake3`, `openssl`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `forma-server` 0.2.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `formatjs_cli` 1.4.2:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `formatjs_intl` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `formatorbit-cli` 0.10.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `formatorbit-core` 0.10.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `foundry-block-explorers` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `four-word-networking` 2.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `foxtive` 1.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fp-bench` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fp-columnar` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-conformance` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fp-dot-kernel` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fp-expr` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-frame` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `fp-frankentui` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-groupby` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-index` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-io` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-join` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-python` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fp-runtime` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fp-types` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fpe` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fracsync` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fraiseql-observers` 2.14.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fraiseql-webhooks` 2.14.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `frand` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `franken_whisper` 0.8.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `frankenpandas` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `franklin-crypto` 0.32.11:
   category mismatch: consumer of `blake2s_simd`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `freetsa` 0.1.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frf` 0.1.86:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frost-core` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `frost-core-unofficial` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `frost-ed25519` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frost-ed25519-blake2b` 2.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `frost-p256` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frost-rerandomized` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `frost-ristretto255` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frost-secp256k1` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frost-secp256k1-tr` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `frost-secp256k1-tr-unofficial` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `fruitbasket` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fs-verity` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fsb` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fse_dump` 3.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fselect` 0.10.3:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `fshasher` 0.3.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `fsqlite-mvcc` 0.4.4:
   category mismatch: consumer of `blake3`, `xxhash-rust` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `fstool` 0.4.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fuck-tools-rs` 0.0.5:
   category mismatch: consumer of `aes`, `md-5`, `ring`, `sha1` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `fuel-crypto` 0.66.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fuellite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fugazi` 0.96.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fugue-ustr` 1.1.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `fullbleed` 2.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fundaia` 0.31.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `futu-rs` 0.1.2:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `futures-rustls` 0.26.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fv-mac` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fx-hash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fxhash` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `fynx-proto` 0.1.0-alpha.3:
   category mismatch: consumer of `aes`, `md-5`, `ring`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `g2g-core` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `g2p` 1.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gamlastan-mdq` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gaoya` 0.2.2:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `garbled-circuit` 1.3.1-pre.2:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `garrison` 0.8.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gaussdb` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gaussdb-protocol` 0.1.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `gaussdb-types` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gaze-mcp-core` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gb28181-rs` 0.11.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `gcloud-artifact-registry` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gcrypt` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `geekorm-derive` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gel-jwt` 0.1.4:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `gem-index-filter` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gencrypt` 0.11.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `generic-static-cache` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `genesis-types` 26.7.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `genotp` 0.3.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `genshi-core` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `genshi-evm` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gentoo-cruft` 1.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `get_dir_hash` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `get-hash` 1.1.9:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `getrandom_or_panic` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `getsecure` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `geyserlite` 0.5.27:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gfeh-core` 0.2.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ggen` 26.8.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ggen-cli-lib` 26.7.3:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ggen-config` 26.15.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ggen-graph` 26.15.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ggen-marketplace` 26.15.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ghash` 0.6.0:
   category mismatch: GHASH universal hash over `polyval`.
- `giant-spellbook` 0.4.10:
   category mismatch: consumer of `blake3`, `openssl-sys`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `gifterm` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `git_sshripped_recipient` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `git_sshripped_recipient_models` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `git-cinnabar` 0.7.5:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `git-credential-keepassxc` 0.14.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `git-facade` 1.4.2:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `git-internal` 0.9.0:
   category mismatch: consumer of `ahash`, `blake3`, `crc32fast`, `sha1`, `sha2` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `git-sha1` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `git-stk` 0.12.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `github_submodule_hook` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `github-bot-sdk` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gitmelt` 0.5.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gitoid` 0.9.0:
   category mismatch: consumer of `boring`, `openssl`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `gitveil` 1.3.4:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `gk-authenticator` 0.0.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `glass_pumpkin` 1.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gm-crypto` 0.3.1:
   category mismatch: consumer of `ghash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gm-rs` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gm-sm3` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gm-sm9-rs` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gm-tlcp` 0.6.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gm-tls` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gmac` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gmac_rs` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gmcrypto-c` 1.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gmcrypto-core` 1.13.0:
   HC1 and category: SM3 and SM4 primitives without both-architecture hash acceleration.
- `gmsm` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gmssl-rs` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gmssl-rust` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gnir` 0.16.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gnostr-sha256` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `gnucobol-rs` 0.8.57:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `go-pmacro` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `goish` 0.20.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `goldilocks-crypto` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `golomb-set` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `goosedump` 0.12.64:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `gorn` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gosh-dl` 0.6.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `gossan-core` 0.3.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gost-crypto` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gost94` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `goxoy-hash` 0.0.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `gpgme` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gpgme-sys` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gpt-partition-core` 0.6.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gpt-partition-forensic` 0.6.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gpu-backend` 0.1.0-alpha.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `gpui-updater-core` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gpui-updater-pre` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grain-id-cli` 0.16.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `graphify-cache` 0.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `graviola` 0.4.1:
   screening survivor for SHA-256 (x86 SHA-NI with a portable fallback, aarch64 sha2
   intrinsics); SHA-384 and SHA-512 have an x86_64 AVX2 kernel but no aarch64 kernel (HC1 on
   aarch64).
- `greentic-deployer` 1.1.30:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `greentic-deployer-dev` 1.2.34018717538:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `greentic-runner-host` 1.1.11:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `gridiron` 0.12.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_api` 5.5.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `grin_chain` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_config` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_core` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_keychain` 5.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `grin_p2p` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_pool` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_secp256k1zkp` 0.7.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_servers` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_store` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_util` 5.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_wallet` 5.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grin_wallet_util` 5.4.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `groestl` 0.11.0:
   HC1: the RustCrypto Grøstl implementation is portable, with no AES-NI or NEON path.
- `groestl-aesni` 0.3.1:
   HC1: Grøstl with AES-NI on x86 only.
- `groestlcoin` 0.31.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groestlcoin_hashes` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groestlcoin_slices` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `groestlcoin-internals` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groestlcoin-private` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groestlcoinconsensus` 2.19.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groestlcoincore-rpc` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groestlcoincore-rpc-json` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `groth16-proofs` 4.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `grovedb` 3.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `grundschutz-oscal-viewer` 1.6.59:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `gsearch` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gsigner` 2.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gsocket` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gsocket-proto` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gsocket-tls-srp` 0.1.0:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `gst-plugin-gtk4` 0.15.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `gstp` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `guestkit` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gvrn` 0.27.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gwm-cli` 1.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `gxhash` 3.5.0:
   HC9 (user decision); also HC10, because `GxHasher` output depends on write chunking.
- `gxlang` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `h33-agent-token` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `h33-mcp` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `h33-substrate-verifier` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hail_pdk_procmacro` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hal` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `halftime` 0.1.1:
   category mismatch: almost-universal hash needing key entropy and padded updates.
- `halo2_proofs` 0.3.5:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `halo2curves` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `halo2curves-axiom` 0.7.3:
   category mismatch: consumer of `blake2b_simd`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hamming-bow` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hamming-lsh` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hank` 0.4.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hanzo-rocm-kernels` 0.11.37:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hap-crypto` 1.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hapi-iron-oxide` 0.1.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `happy-cracking` 0.14.0:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `hardpass` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hardware-rust-crypto` 1.0.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `harmonia-store-core` 0.0.0-alpha.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `harmonia-utils-hash` 0.0.0-alpha.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `harn-hostlib` 0.10.138:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hash_token_rust` 0.3.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hash_utils` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2`, `sha3`, `twox-hash` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `hash-data` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hash-gen` 1.1.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hash-id-crack` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hash-lib` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `hash-rs` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hash-wasm-rs` 0.1.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `hash2curve` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hash2field` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hash4lf` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `hashavatar` 1.3.0:
   category mismatch: consumer of `blake3`, `xxhash-rust` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `hashcodecs` 1.4.1:
   HC10 for its XXH3-128 (one-shot, prepared, and batch APIs only) and HC1 for its MurmurHash3,
   whose aarch64 body is scalar.
- `hashcracker` 1.1.3:
   category mismatch: consumer of `aes`, `md-5`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hashcrew` 0.3.0:
   screening survivor: XXH3-128 with NEON, SSE2, and AVX2 kernels for inputs over 240 bytes,
   streaming `Xxh3_128`, Apache-2.0.
- `hashdex` 0.1.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hashdir2` 0.1.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hashdistinct` 0.4.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hashed-type-def-procmacro` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hasher` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashers` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashery` 0.0.4:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `hashes` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashglass` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashgraph-like-consensus` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hashguard` 5.0.2:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hashing` 0.1.8:
   category mismatch: consumer of `blake3`, `md-5`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `hashing-serializer` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashjunkie` 0.6.0:
   category mismatch: consumer of `blake3`, `crc32fast`, `md-5`, `sha1`, `sha2`, `xxhash-rust`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `hashjunkie-cli` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashkit` 0.1.5:
   category mismatch: adapter over other hash crates.
- `hashkitten` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashpump` 0.1.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hashsigs-rs` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashstr` 0.3.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `hashstr-derive` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashstream` 0.1.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `hashsum` 0.1.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hashtree-blossom` 0.2.83:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hashtree-core` 0.2.89:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hashtree-lmdb` 0.2.88:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hashtree-resolver` 0.2.85:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hashtree-rs` 0.2.5:
   HC4: bindings to the hashtree C library.
- `hashver` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hashx` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hassh` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-ci` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-engine` 0.1.0-beta.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hauksbee-extract` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-frontdoor-api` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-ir` 0.1.0-beta.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hauksbee-mcp` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-mcu` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-models` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-server` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hauksbee-solve` 0.1.0-beta.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `haveibeenpwned-downloader` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `havocompare` 0.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hayahash` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `haz-cache` 0.2.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hc_seed_bundle` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hctr2` 0.2.0:
   category mismatch: consumer of `polyval` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hctr2-rs` 0.9.2:
   category mismatch: consumer of `aes`, `polyval`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `hd-cas` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hd-engine` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hdds` 1.1.2:
   category mismatch: consumer of `md-5`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hdds-codegen` 1.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hdk` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hdpki` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hdwallet` 0.4.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `headers-content-md5` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hearth-sdk` 1.6.11:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `heddleco-capability-verifier` 0.19.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hedl-c14n` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hegeltest-c` 0.43.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hekate-crypto` 0.34.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `hekate-keccak` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hematite-cli` 0.14.1:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hephasm` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hermes_rs` 0.1.14:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hermes-tdata` 0.2.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `hermit-rs` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hex_str` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `heyo-sdk` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hf-fetch-model` 0.12.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hfile` 0.5.2:
   category mismatch: consumer of `blake3`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hft-crypto` 0.3.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hibp-bin-fetch` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hibp-verifier` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hiero-streams` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `highhash` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hightower-wireguard` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `highway` 1.3.0:
   screening survivor: HighwayHash-128 with SSE4.1, AVX2, and NEON paths, streaming
   `append`/`finalize128`, MIT.
- `hip4` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hisi-crypto-ws63` 0.1.0-alpha.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hiss` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hkd32` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hkdf` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hmac` 0.13.0:
   category: a keyed construction over a hash that is itself a candidate; with a constant key
   it computes the underlying hash twice over a padded prefix, so it is evaluated through
   `sha2`.
- `hmac-circuit-breaker` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hmac-cli` 0.1.16:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hmac-sha` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hmac-sha1` 0.2.2:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hmac-sha1-compact` 1.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hmac-sha256` 1.1.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hmac-sha512` 1.1.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hmac-sm3` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hofmann-rfc` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `holger` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-cli` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-core` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-maven-znippy-repository` 0.6.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-python-znippy-repository` 0.6.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-ron` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-rust-repository` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `holger-traits` 0.6.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holger-znippy-package-repository` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `holochain_keystore` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `homecore-hap` 0.1.0-alpha.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `honest_aes` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hopf-auth` 0.3.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hopper-runtime` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hotg-rune-wasm3-runtime` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hotmint-crypto` 0.8.11:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hotmint-types` 0.8.11:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hpke` 0.14.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `hpke_pq` 0.11.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hpke-dispatch` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hpke-ng` 0.2.0:
   category mismatch: consumer of `aes`, `ghash`, `polyval`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `hsh` 0.0.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hsh-digest` 0.0.10:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `hswp` 0.1.0:
   category mismatch: consumer of `ring`, `sha2`, `sodiumoxide` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `htauth` 0.1.2:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `htauth-cli` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `htpasswd-verify` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `http-auth` 0.1.10:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `http-ferry` 0.2.1:
   category mismatch: consumer of `md-5`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `http-sig` 0.6.0:
   category mismatch: consumer of `openssl`, `ring`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `http-signature-normalization` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `http-signature-normalization-actix` 0.12.0:
   category mismatch: consumer of `ring`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `http-signature-normalization-actix-extractor` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `http-signature-normalization-http` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `http-signature-normalization-reqwest` 0.14.1:
   category mismatch: consumer of `ring`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `http-signatures` 0.8.0:
   category mismatch: consumer of `openssl`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `httpsig` 0.0.26:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `httpsig-hyper` 0.0.26:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `huber-procmacro` 1.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `huddle` 2.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `huddle-core` 2.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `huddle-gui` 2.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `huddle-protocol` 2.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `huddle-server` 2.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hugincyber` 0.4.40:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `human-bytesize-procmacro` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hurl` 8.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hwaddr` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hx-plugins` 0.9.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hxedit` 0.5.0:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `hya-net` 0.5.1:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `hydracache-redis-compat` 0.70.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hyper-mcp` 0.8.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hyperchad_renderer_vanilla_js_hash` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `hyperuuid` 0.3.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `hyprsaver` 0.4.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `i2cdev-lsm303d` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `i2cdev-lsm303dlhc` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `i2p_client` 0.2.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `i2p_snow` 0.5.1:
   category mismatch: consumer of `hacl-star`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `iax2` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ibc-middleware-module` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ibc-middleware-module-macros` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ibc-middleware-overflow-receive` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ibc-middleware-packet-forward` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ibe` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ic_auth_verifier` 0.10.4:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ic-core-module` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ic-query` 0.43.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ic-query-cli` 0.43.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ic-rig` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ic-sha3` 1.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ic-verifiable-credentials` 1.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `icao-9303` 0.1.0:
   category mismatch: consumer of `aes`, `blake3`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `ice` 0.4.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `idax` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `idea` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `identyhash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `idevice-srp` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ids_service` 2.1.4:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `iicp-client` 0.7.109:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ikigai-browse` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ilink-hub` 0.4.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `illusion` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `image_charts` 6.1.183:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `image_data_hash` 0.1.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `imgfprint` 0.4.6:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `immure` 0.3.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `imohash` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `impcurl` 1.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `impcurl-sys` 1.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `inane-crypto` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `inc-sha1` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `include_assets` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `inco-lightning` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `indextreemap` 0.2.0:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `indodax-cli` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `indy-blssignatures` 0.1.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `inet2_derive` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `inferadb-ledger-types` 0.1.0-dev.20260917:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `infinikey` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `infinitree` 0.11.0:
   category mismatch: consumer of `blake3`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `infinitree-macros` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `injective-cosmwasm` 0.3.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `inkbox` 0.7.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `inklog` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `inline_csharp` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `innospect` 0.1.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `internal-russh-forked-ssh-key` 0.6.18+upstream-0.6.7:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `internet2` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `inventorize` 0.1.1:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `io-imap` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `io-managesieve` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `io-sasl` 0.1.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `io-smtp` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iocutil` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iota-crypto` 0.23.2:
   category mismatch: consumer of `aes`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ipfrs-core` 0.2.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `ipfrs-network` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ipmi-rs` 0.6.0:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `iprr` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iq-crypto` 0.0.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `irgx` 2.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iris-crypto` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `iroh-blake3` 1.4.5:
   duplicate of `blake3` 1.4 forked for iroh; screened through `blake3`.
- `iroh-blobs` 0.103.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iroh-bytes` 0.15.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iroh-dht-experiment` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ironauth-hash-scheme` 0.2.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ironflow-runtime` 2.4.32:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `irontide` 1.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ironvault` 8.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `irox-tools` 0.11.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `is` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `isap-aead` 0.3.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `iscc-lib` 0.6.0:
   category mismatch: consumer of `blake3`, `xxhash-rust` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `isideload-cryptographic-message-syntax` 0.31.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ism330dhcx` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ism330dhcx-rs` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ism330is-rs` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iso8583_rs` 0.1.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `iso9796_rsa` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ix-id` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jacs` 0.13.0:
   category mismatch: consumer of `sha2`, `openssl`, `ring` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `jacs-binding-core` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jamhash` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `janice` 0.3.0:
   category mismatch: consumer of `ahash`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `jaws` 1.0.4:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jax-common` 0.1.11:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jbig2enc-rust` 0.5.4:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `jcl` 1.2.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jcs-canonicalize` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `jh` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jh-rs` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jh-x86_64` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jin` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `jisp_sha3` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jja` 0.9.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `jl-ui-procmacro` 0.1.2026-dev:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jmap-base-client` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `jmap-cid-types` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jmap-client` 0.4.2:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `jmespath_extensions` 0.9.0:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `jose-b64` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jose-jwa` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jose-jwk` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jose-rs` 0.7.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `josekit` 0.10.3:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `json_atomic` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `json-digest` 0.0.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jsonwebtoken` 11.1.0:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `jsonwebtoken-ic` 10.3.0-ic.0:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `jsonwebtokens` 1.2.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `jubjub-schnorr` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jupyter-protocol` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `just_progress` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `jvmrs` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `jw` 2.4.0:
   category mismatch: consumer of `blake3`, `sha2`, `xxhash-rust` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `jwc-rs` 0.1.6:
   category mismatch: consumer of `ghash`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jwt` 0.16.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jwt-next` 0.17.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `jwt-simple` 0.13.1:
   category mismatch: consumer of `blake2b_simd`, `boring` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `k12` 0.5.1:
   screening survivor with a build note: KangarooTwelve over `keccak`, whose accelerated
   backends are the aarch64 SHA-3 extension (run-time detected) and nightly `portable_simd`
   lanes selected by `--cfg keccak_backend="simd128|simd256|simd512"`; without that cfg the x86
   path is scalar (HC1).
- `k12sum` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `k256` 0.14.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `kache` 0.23.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kadcast` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kademlia-dht` 1.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kadena` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kal-sc` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kamu-snap-crypto` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kangarootwelve` 0.1.3:
   HC1: KangarooTwelve over `turboshake` 0.5.0 with no architecture intrinsics on either target.
- `kangarootwelve_xkcp` 0.2.1:
   HC4: FFI wrapper compiling the XKCP C implementation.
- `kaspa-portal` 1.0.1:
   category mismatch: consumer of `blake2b_simd`, `blake3`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `katana-markdown-linter` 0.19.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kbkdf` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kbs2` 0.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kdf` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak` 0.2.2:
   permutation crate behind `k12`, `sha3`, `turboshake`, and `cshake`; screened through them.
- `keccak_cli_tool` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak_prime` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak-asm` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak-batch` 0.1.0:
   HC6: AGPL-3.0-or-later, which cannot be distributed inside an LGPL-3.0-or-later binary; also
   a batch API rather than a single-message hash.
- `keccak-const` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak-hash` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak-hasher` 0.16.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccak-p` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `keccak-rust` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccakf` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccakrs` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keccakrs-wasm` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kem` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kerbcore` 0.2.1:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `kerl` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `kes-summed-ed25519` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kevy` 6.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kevy-hash` 6.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keychain-db` 0.2.6:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `keychain-rs` 0.2.6:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `keygrip` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keygrip-derive` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keyhog` 0.5.86:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `keyhog-profile` 0.5.86:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `keyhog-scanner` 0.5.86:
   category mismatch: consumer of `aes`, `ahash`, `blake3`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `keynesis` 2.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `keyper` 0.6.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `keyrx` 0.4.25:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `khqr` 0.1.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `khqr-api` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `khqr-cli` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `khqr-core` 0.2.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `khqr-ffi` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `khqr-wasm` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kibble` 0.1.0:
   category mismatch: consumer of `aes`, `md-5`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `kindi` 0.2.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kindly-guard-server` 0.11.14:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kira_cdh_compat_cluster` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kira_cdh_compat_lsh` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kira-biodata-manager` 0.3.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kira-cdh` 0.1.1:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kira-mmcif` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `kira-vrs` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kiss_chat` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kitak` 3.2.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kittymemory-rs` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kiwi-rs` 2026.8.26:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kk-crypto` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `klauthed-security` 1.0.0:
   category mismatch: consumer of `ring`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `klbfw` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kleos-lib` 1.10.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `klvmr` 0.19.0:
   category mismatch: consumer of `sha1`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kmac` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `knishio-client` 1.1.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `knixl` 1.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `knolo-agent` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `knolo-agent-core` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `knot-server` 0.6.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `knx-rs-core` 0.9.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `knx-rs-device` 0.9.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `knx-rs-ip` 0.9.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `knx-rs-tp` 0.9.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-aptos` 3.4.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kobe-arweave` 3.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kobe-btc` 3.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kobe-casper` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-cli` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-cosmos` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-evm` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-fil` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-nostr` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-primitives` 3.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kobe-spark` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-sui` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-svm` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobe-ton` 3.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kobe-tron` 3.4.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kobe-xrpl` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kobold-data-shim` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `koibumi-secp256k1` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kona-p2p` 0.1.2:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kotlin-bridge` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kotlin-bridge-build` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kotlin-bridge-cli` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kotlin-bridge-ir` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kotlin-bridge-macro` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kotlinrs` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kotoba-core` 0.1.22:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kotoba-db-core` 0.1.16:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kotoba-jsonnet` 0.1.22:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `kotoba-storage` 0.1.22:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kproc_pmacros` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krabiecdsa` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krafka` 0.24.0:
   category mismatch: consumer of `ahash`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kraken-async-rs` 0.16.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `krata-advmac` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krb5-gss` 0.2.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `kremis-core` 0.21.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `krusty-kms-client` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-arcana` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-arcana-ffi` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-arcana-wasm` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-quantica` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-quantica-ffi` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-tessera` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-tessera-ffi` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `krypteia-tessera-wasm` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kryptering` 0.5.0:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `md-5`, `sha1`, `sha2`, `sha3` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `kunobi-auth` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kupyna` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `kuska-sodiumoxide` 0.2.5-0:
   category mismatch: consumer of `libsodium-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kuznyechik` 0.9.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `kvendra` 0.6.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `kwallet-parser` 0.7.0-alpha:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `kyber-rs` 0.1.0-alpha.9:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `kyumdb` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `l-s` 0.5.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `xxhash-rust` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `l402_middleware` 2.3.4:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `l4d2_addon_parser` 1.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `labrinth` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lair_keystore` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lair_keystore_api` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lakers` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lakers-shared` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lambda-otel-lite` 0.19.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lambdaworks-crypto` 0.13.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `langbox_procmacro` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `laron-wallet` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `late` 0.0.970:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lattice-kyber` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lattice-tune` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `latticearc` 0.12.0:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `layer-crypto` 0.5.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `lazytools` 0.5.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lazytools-core` 0.5.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `leaf-markdown-viewer` 1.28.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `leakguard` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ledger_device_sdk` 1.37.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `leguichet-in` 0.1.6:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `leona` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `leslie_lamport` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lesspass` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lettre` 0.11.23:
   category mismatch: consumer of `boring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `lexariel` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lexe-sha256` 0.1.23:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lexlite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lfchring` 0.1.3:
   category mismatch: consumer of `blake2b_simd`, `blake3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `lhash` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-aead` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-cb-kem` 0.0.11:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lib-q-core` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-duplex-aead` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-fn-dsa` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-fn-dsa-alg` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-fn-dsa-comm` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-fn-dsa-kgen` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-fn-dsa-sign` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-fn-dsa-vrfy` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-hash` 0.0.11:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lib-q-hqc-traits` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-intrinsics` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-k12` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-keccak` 0.0.11:
   HC1 for the consumed surface: Keccak with a `portable_simd` batch path and aarch64 assembly,
   no single-message x86 vector path; also 0.0.x.
- `lib-q-keccak-digest` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-mac` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-mve` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-platform` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-plonky` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-plonky-batch-stark` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-plonky-keccak-air` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-plonky-lookup` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-plonky-multilinear-util` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-plonky-uni-stark` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-poseidon` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-random` 0.0.11:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lib-q-rocca-s` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-romulus` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-saturnin` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-sha3` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-slh-dsa` 0.0.11:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lib-q-stark` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-air` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-challenger` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-commit` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-dft` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-field` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-stark-field-testing` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-fri` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-interpolation` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-matrix` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-mds` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-merkle` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-mersenne31` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-stark-monty31` 0.0.11:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lib-q-stark-rayon` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-sha3-256` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-shake128` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-shake256` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-symmetric` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-stark-util` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-transcript` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-tweak-aead` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-utils` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib-q-zk-encryption-proof` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib3h_crypto_api` 0.0.42:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lib3h_sodium` 0.0.42:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libaes` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libage_auth_handler` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libage_otp` 0.2.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libapt` 1.3.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libb2-sys` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libbitcoin` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libbitcoinpqc` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libbruteforce` 4.1.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libchdman-rs` 0.289.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libcrux` 0.0.5:
   category mismatch: consumer of `libcrux-sha2`, `libcrux-sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `libcrux-blake2` 0.0.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libcrux-digest` 0.0.9:
   category mismatch: consumer of `libcrux-sha2`, `libcrux-sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `libcrux-hacl-rs` 0.0.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libcrux-intrinsics` 0.0.8:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `libcrux-psq` 0.0.10:
   category mismatch: consumer of `libcrux-sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libcrux-sha3` 0.0.10:
   HC1 for the consumed surface: its simd128 and simd256 modules hash several messages in
   parallel (src/lib.rs:260-271); a single message uses the portable permutation.
- `libcrypt-rs` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libes` 0.9.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libflac-rs` 0.143.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libgmssl-sys` 3.1.0-alpha:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libgrite-core` 0.5.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libherokubuildpack` 0.31.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libid-attestations` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libid-crypto` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libjade-sys` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libmacaroon` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libmhash` 0.2.1:
   category mismatch: consumer of `crc32fast`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `libmoshpit` 0.12.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libotp` 0.2.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libp2p-identity` 0.3.0:
   category mismatch: consumer of `sha2`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libpep` 0.13.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libpna` 0.39.0:
   category mismatch: consumer of `aes`, `crc32fast`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `libqabu` 0.2.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libquarkpan` 0.4.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libr2fa` 0.1.3:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libreauth` 0.18.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libro` 0.92.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `librqbit-sha1-wrapper` 9.0.1:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `librqbit-tracker-comms` 9.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `librsync_oxide` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `librtbit-sha1-wrapper` 0.1.1:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libsecp256k1` 0.7.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libsecp256k1-core` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsecp256k1-gen-ecmult` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsecp256k1-gen-genmult` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libside-procmacro` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libslug` 0.13.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `libsm` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsm_stzhang` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsmx` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsodium-rs` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsodium-sys` 0.2.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsodium-sys-stable` 1.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libsui` 0.16.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `libsumatracrypt-rs` 0.6.1:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `libtls` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libvault-core` 0.1.0:
   category mismatch: consumer of `blake2b_simd`, `blake3`, `openssl`, `openssl-sys` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `libvctrl` 2.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libvctrl_core` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libvctrl_handler` 5.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libvctrl_sha512` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `libxml-rs` 0.1.0-alpha.60:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `licenz-core` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `licverify` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `light-hasher` 7.0.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `light-openid` 2.0.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `light-poseidon` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `light-sdk-macros` 0.25.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lightcone` 0.10.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `lightmotif` 0.10.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lightmotif-py` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lighty-core` 26.9.3:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `liminal-ark-pnbr-sponge` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `limiteron` 0.2.10:
   category mismatch: consumer of `ahash`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `limpet` 0.17.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `linear-api` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `linear-cli` 0.3.28:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ling-crypto` 2030.1.12:
   category mismatch: consumer of `blake3`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `link-assistant-router` 1.11.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lioness` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `liquidqc` 1.0.0:
   category mismatch: consumer of `crc32fast`, `md-5` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `listinfo` 0.4.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `litany` 0.0.13:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `litecoin_hashes` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `litelite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `litvc` 1.6.0:
   category mismatch: consumer of `aes`, `blake3`, `crc32fast`, `sha1`, `sha2`, `sha3` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `ll-neighbors` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `llama-rs` 0.22.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `lllv-core` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `lllv-index` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `llmlint` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `llmshim` 0.3.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lms` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lms-signature` 0.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lnpbp` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lockboxer` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lockit` 0.1.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lockstitch` 0.29.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `loco-oauth2` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-core` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-engine` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-host` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-packs` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-protocol` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-provider` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `locode-tools` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `loctree` 0.14.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `logdbd` 0.8.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `logicaffeine-base` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `logicaffeine-system` 0.10.1:
   category mismatch: consumer of `crc32fast`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `logmachine` 2.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `loli_lib_dev` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lonesha256` 1.1.0:
   HC4: bindings to a C library.
- `lonkero` 3.7.3:
   category mismatch: consumer of `ahash`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `lora-packet` 1.1.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `loreyawen` 0.2.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `loupe-cli` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `loupe-core` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `loupe-proto` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `loupe-server` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `loupe-storage` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `loupe-tls` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `loupe-web` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `lowdash` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsh-rs` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsh-rs2` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsm303` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsm303agr` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsm303c` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsm303dlhc` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsm303dlhc-ng` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsm303dlhc-registers` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsmtree` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lsx` 1.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ltk_wad` 0.5.4:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `lucky_commit` 2.2.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `luggage` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `luks-and` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `luks-core` 0.1.8:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `luks-forensic` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `luks2` 0.5.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `lurpax` 0.11.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `lust` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lustro` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lzvn` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `lzy_pbkdf2` 0.1.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `m5stack-core` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `maat_air` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `maat_stdlib` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac_address` 1.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac_address2` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac_conditions` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac_oui` 0.4.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac_proxy` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac-digest` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac-sys-info` 0.1.13:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mac6` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `macaddr` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `macaddr-ouidb` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mach-siegbert-vogt-dxcsa` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `machineid-crystal` 1.2.6:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `machineid-rs` 1.2.4:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `macid` 1.1.3:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `macnuf` 0.1.70+20260914:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `macos-tags` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `macpow` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `macro-toolset` 0.8.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `madeonsol` 0.28.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `magi-rs` 0.19.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `magic-crypt` 5.0.1:
   category mismatch: consumer of `aes`, `md-5`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `magikitten` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `magma` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `magnumdb` 0.4.8:
   category mismatch: consumer of `ahash`, `crc32fast` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `mail-auth` 0.13.2:
   category mismatch: consumer of `aws-lc-rs`, `ring`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `mail-send` 0.6.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `maili-protocol` 0.2.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mailrs-arc` 3.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mailrs-inbound` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mailrs-smtp-proto` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mailrs-webhook-signature` 1.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `makechain-crypto` 0.1.1:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `makegov-tango-webhooks` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `makiko` 0.2.5:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mallard` 0.1.0:
   category mismatch: consumer of `crc32fast`, `md-5`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `malwaredb` 0.3.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `malwaredb-client-py` 0.3.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mandible` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mandible-core` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mandible-extract` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mandible-search` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mandible-tui` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `manuf` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `manzana` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `marg-core` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `marg-storage` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `markdown-org-extract` 0.24.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `markdown-tui-explorer` 1.34.75:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `marsupial` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `marsupial-sys` 0.2.0:
   HC4: FFI crate for the XKCP K12 C implementation.
- `matchcorr` 0.1.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `matchy-wasm` 2.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `matrix-sdk-store-encryption` 0.19.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `matrix256` 1.0.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `matter-cert` 0.3.2:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `matter-commissioning` 0.10.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `maven-haste` 0.2.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mavryk_crypto_rs` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mbedtls` 0.13.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mbedtls-platform-support` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mbedtls-rs-sys` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mc-oblivious-map` 2.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mc-oblivious-ram` 2.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mc-oblivious-traits` 2.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mc-rand` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mc-sgx-tcrypto-sys-types` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mcaptcha_pow_sha256` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mcf` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mcp_daemon` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mcp-gateway` 3.5.1:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mcu-comms` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `md-5` 0.11.0:
   HC1: MD5 with an x86-only assembly option and no aarch64 accelerated path.
- `md-viewer` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md2` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md4` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5-asm` 0.5.2:
   HC4: assembly compiled by `cc`.
- `md5-core` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5-extensions` 0.2.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `md5-img` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5-many` 0.1.0-alpha.4:
   category mismatch: multi-buffer batch MD5.
- `md5-rs` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5crypt` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5hash` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5mix` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `md5namer` 0.1.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mdf` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mdg` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mdictlib` 0.2.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mdoc-rs` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mdtablefix` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `meld-config-manager` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `melda` 0.6.8:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `melodies-blake2` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `memflow-registry` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mentedb` 0.37.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mentedb-cognitive` 0.37.3:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-consolidation` 0.37.3:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-context` 0.37.3:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-core` 0.37.3:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-embedding` 0.37.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-extraction` 0.37.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-graph` 0.37.3:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mentedb-index` 0.37.3:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `mentedb-query` 0.37.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mentedb-server` 0.37.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mentedb-storage` 0.37.3:
   category mismatch: consumer of `ahash`, `crc32fast` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `mentisdb` 0.10.8:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `meowhash` 0.3.0:
   HC1 and HC3: aarch64 support disabled since 0.2; MeowHash 0.5 output not declared final.
- `meowpow` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mercs2_engine` 1.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mercs2_formats` 3.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mercs2_game` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mercs2_smuggler` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mercs2_workshop` 3.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mere-proofs` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `merkeltreerust` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkle_hash` 3.9.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `merkle_light` 0.4.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `merkle-forest` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `merkle-heapless` 0.0.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkle-helix-rs` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `merkle-sha3` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkle-tree-accumulator` 0.3.0:
   category mismatch: consumer of `blake3`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `merkle-tree-db` 0.0.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `merkle-tree-stream` 0.12.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkle-variants` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkleforge-hash` 0.4.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `merkletree-mintlayer` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkletreerust` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `merkql` 0.2.0:
   category mismatch: consumer of `crc32fast`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `merlin` 3.0.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `merlin-rna` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mesalink` 1.1.0-cratesio:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mesh-core` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `metal-json` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `metamorphic-crypto` 0.11.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `metamorphic-log` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `metaunmask` 0.1.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `metaunmask-cli` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `metrohash` 1.0.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `micro-xoodyak` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `microBioRust` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `microBioRust-microSeqIO` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `micromail` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `microsalt` 0.2.21:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mid-signin` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `midas_fetcher` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-ace-codegen` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-air` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-assembly` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-core-lib` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-crypto` 0.33.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `miden-crypto-derive` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-field` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-lifted-air` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-precompiles` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-precompiles-prover` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-protocol` 0.16.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-prover` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-serde-utils` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-stark-transcript` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-stateful-hasher` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-stdlib` 0.19.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miden-vm` 0.33.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `midnight-zk-stdlib` 2.3.5:
   category mismatch: consumer of `blake2b_simd`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `minhash-rs` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mini-app-core` 0.19.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mini-film` 23.2.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `mini-functions` 0.0.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `minip2p-noise` 0.5.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `miniscript` 13.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miniserve` 0.35.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `minisign` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `minisign-verify` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `minlz` 1.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `minotp` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mirador` 1.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miscreant` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `miscreant-aes-v05` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `misp-rs` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mithril-stm` 0.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mizaru2` 0.2.20:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mkit-core` 0.4.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `MKT_KSA_Geolocation_Security` 2.0.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ml-dsa` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ml-kem` 0.3.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mlkem-rs` 0.12.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mlkem-selkie` 0.0.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `mls-rs` 0.56.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mlx-native` 0.15.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mmap-chunker-core` 0.2.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mnemo-compliance` 0.5.29:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mnemo-core` 0.5.29:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mnemo-db` 0.5.29:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mnemo-mcp` 0.5.29:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mnemossh` 0.1.13:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `moadim` 3.2.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-amqp` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-analytics` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-chaos` 0.3.222:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-cli` 0.3.222:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-contracts` 0.3.222:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-foundation` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-import` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-intelligence` 0.3.222:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-observability` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-openapi` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-plugin-cli` 0.3.222:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-plugin-core` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-plugin-loader` 0.3.222:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mockforge-plugin-registry` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-proxy` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-recorder` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-route-chaos` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-scenarios` 0.3.222:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-sdk` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-template-expansion` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-tracing` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-tui` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-tunnel` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-ui` 0.3.222:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mockforge-vbr` 0.3.222:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mockforge-workspace` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mockforge-ws` 0.3.222:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `models-cat` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `modern-crypto-toolkit` 1.1.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `modlite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `modrinth-wrapped` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `modular-agent-core` 0.30.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `module-lattice` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `moirai-crypto` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `molt-argparse-procmacro` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan` 5.2.12:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `monerochan-build` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-core-executor` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-core-machine` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-cuda` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-curves` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-derive` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-primitives` 5.2.12:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `monerochan-prover` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-recursion-circuit` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-recursion-compiler` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-recursion-core` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-recursion-derive` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monerochan-recursion-gnark-ffi` 5.2.12:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `monerochan-stark` 5.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mongo-migrate` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mongocrypt` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mongocrypt-sys` 0.1.6+1.18.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `monotree` 0.4.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `moongraphql_builder` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `moonshine-tag` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `moq-relay` 0.14.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `morocco` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `moshpit` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `moshpit-agent` 0.12.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `moshpit-keygen` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `moshpits` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `motosan-ai` 0.27.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mpfs-hal-procmacros` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mpl-protocol` 0.1.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `mpt-crypto` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mpt-crypto-sys` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mq-db` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mqtt-procmacro` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mrgfile` 0.1.14:
   category mismatch: consumer of `aws-lc-rs`, `aws-lc-sys`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `ms-icpr` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ms-offcrypto-writer` 1.0.7:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ms-pkca` 0.1.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `msoffice-crypto` 0.1.0-rc.2:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `msqlx` 0.9.0-msqlx.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `multi-hash` 1.1.2:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `multi-key` 2.0.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `multi-sig` 1.4.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `multihash` 0.19.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `multihash-codetable` 0.2.2:
   category mismatch: multihash code table over other crates.
- `multiset-hash` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `murmur3` 0.5.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `murmur3_32` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `museair` 0.6.0:
   HC1: scalar multiply design, measured as the non-cryptographic scalar control.
- `mutree` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `mwa_giant_squid` 2.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mx3` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `my-password` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `my-password-cli` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `myca` 0.2.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mykey` 1.0.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `myna-card` 3.3.3:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `mysten-mldsa-native-rs` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `mythic-c2` 0.2.2:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `n7n-trace` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nahui` 0.3.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `nail-pow` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nam-reddsa` 0.5.2-nam.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `nam-redjubjub` 0.7.1-nam.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_account` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_controller` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_core` 0.251.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `namada_ethereum_bridge` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_events` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_gas` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_governance` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_ibc` 0.251.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `namada_io` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_macros` 0.251.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `namada_merkle_tree` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_parameters` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_proof_of_stake` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_replay_protection` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_sdk` 0.251.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `namada_shielded_token` 0.251.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `namada_state` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_storage` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_systems` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_token` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_trans_token` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_tx` 0.251.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `namada_tx_env` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_vm` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_vote_ext` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_vp` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_vp_env` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `namada_wallet` 0.251.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `names` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nanobook` 0.18.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nargo-cache` 0.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nargo-lock` 0.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `native-tls` 0.2.18:
   category mismatch: consumer of `openssl`, `openssl-sys` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `nautilus-cryptography` 0.64.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `nautilus-polymarket` 0.64.0:
   category mismatch: consumer of `ahash`, `aws-lc-rs` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `nb-cli` 0.0.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nd300` 4.0.1:
   category mismatch: consumer of `sha2`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ndaal-csaf-cli` 1.5.22:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `near-kit` 0.18.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `near-verify-rs` 0.3.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `neco-sha1` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `neco-sha2` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nectar-postage-usage` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nemo-relay-worker` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nemo-relay-worker-proto` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nenjo-packages` 0.37.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `neo-crypto` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `neo-devpack` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `neo-devpack-solidity` 0.22.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `neo3` 3.3.1:
   category mismatch: consumer of `aes`, `ring`, `sha2`, `sha3` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `neptune-cash` 0.17.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `net-mesh` 0.36.0:
   category mismatch: consumer of `blake3`, `ring`, `xxhash-rust` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `netfetcher` 0.1.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `netforge` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nethsm-cli` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nettext` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nettle` 7.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nettle-sys` 2.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `neuedu-cryptos` 0.6.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `neuedu-cryptos-wasm` 0.6.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `neuron-encrypt` 2.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `newton-aggregator` 0.5.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `newtua-common` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `newtua-core` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `newtype_array` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nexus-graph-sdk` 2.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nexus-shield` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ngit-grasp` 3.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nils-common` 1.25.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nivalis` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nix-base32` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nix-narinfo` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nkct` 2.2.1:
   category mismatch: consumer of `openssl`, `openssl-sys`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `nkeys` 0.4.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nkscan` 0.11.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `no-std-svm-merkle-tree` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `no-way-jose-aes-cbc-hs` 0.1.0-rc.2:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `nod` 1.4.4:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `noethers-turnstile-core` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `noetl` 5.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `noise-framework` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `noise-rust-crypto` 0.6.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nonce-auth` 0.6.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `noncrypto-digests` 0.4.0:
   category mismatch: `Digest` adapters over other hash crates.
- `normalize-code-similarity` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nornir-build-thing` 0.1.18:
   category mismatch: consumer of `crc32fast`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `nornir-guard` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nornir-hash` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nostr-ots` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nostringer` 0.1.8:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nostro2-nips` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nothings` 0.0.9:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `nova-scotia` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nova-snark` 0.76.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `npm-utils` 0.6.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `npvdkgrs` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nrf-modem` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ntag424` 0.1.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ntlm-hash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ntlmssp` 0.1.1:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ntpsec-rs` 0.3.50:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ntpsec-rs-core` 0.3.50:
   category mismatch: consumer of `aes`, `md-5`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ntpsec-rs-d` 0.3.50:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ntpsec-rs-keygen` 0.3.50:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ntpsec-rs-query` 0.3.50:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nu_plugin_hashes` 0.1.9:
   category mismatch: consumer of `blake3`, `sha1`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `nulid` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `num-prime` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `numr` 0.7.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `numra-fit` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nunc-time` 0.0.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nuts-container` 0.7.9:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `nuxodecs` 0.1.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `nx-request-handler` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `nxv` 0.7.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `nydus-utils` 0.5.1:
   category mismatch: consumer of `blake3`, `openssl`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `nyxpass` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oakvcs-core` 0.102.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oan-crypto` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oanda-cli` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oas-crypto` 1.1.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oauth-toolkit` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `objecthash` 0.4.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `objr_procmacro` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `objrs` 0.0.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `oboron` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oboron-py` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `ocb3` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oci-api` 0.9.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oci-spec` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oci-tar-builder` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ociman` 0.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ockam` 0.150.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_api` 0.93.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ockam_app_lib` 0.150.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_channel` 0.72.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_command` 0.150.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_core` 0.125.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_ebpf` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_entity` 0.35.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ockam_executor` 0.94.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_identity` 0.134.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ockam_key_exchange_core` 0.65.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_key_exchange_x3dh` 0.71.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_key_exchange_xx` 0.79.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_macros` 0.38.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_multiaddr` 0.70.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_node` 0.139.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_node_attribute` 0.27.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_transport_ble` 0.101.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_transport_tcp` 0.137.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_transport_udp` 0.81.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_transport_uds` 0.66.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_transport_websocket` 0.128.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_vault` 0.132.0:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ockam_vault_aws` 0.58.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ockam_vault_core` 0.35.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_vault_sync_core` 0.42.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_vault_test_attribute` 0.29.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam_vault_test_suite` 0.37.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ockam-ffi` 0.78.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ocsp` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `octofhir-canonical-manager` 0.2.3:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `odf-crypto` 0.1.0-rc.3:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `odin-prompt-toolkit` 0.9.7:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `odl` 3.2.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `of_execution_core` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ofb` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `office-crypto` 0.4.0:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `offsetscan` 0.4.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `oid` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oid-registry` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `okf-attestor` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `okf-validator` 0.2.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `okid` 0.26.0:
   category mismatch: consumer of `blake3`, `sha1`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `ola-lang` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `olm-rs` 2.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `omamori` 1.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `omni-dev` 0.43.0:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `omnia-runtime-macro` 0.34.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `omnisor` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `onc-rpc` 0.3.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `one-step-kdf` 0.1.0-rc.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oneagentgraph` 0.4.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `onefilerepo` 0.34.164:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oneharness` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oneio` 0.26.1:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `onemoney-protocol` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `onepipeline` 0.34.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ootle-wasm` 0.41.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `op-revm` 20.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `opaque-borink` 0.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `opaque-ke` 4.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `opaque-ke-hybrid` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `opaquebind` 0.3.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ope-crypto` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `open-pincery` 1.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `openai-oxide` 0.17.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `openairplay1` 0.3.0:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `openairplay1-dashboard` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openairplay1-dashboard-protocol` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openairplay1-receiver` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openbao` 2.1.8:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `opencrabs` 0.5.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `openfiat-crypto` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `opengm_crypto` 0.1.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `opengm_rts` 0.2.4:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `openmls` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openmls_libcrux_crypto` 0.4.0:
   category mismatch: consumer of `aes`, `libcrux-sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `openmls_traits` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openpgp-cert-d` 0.3.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `openqasm3` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openrouter-rs` 0.15.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `opensaml` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openssl` 0.10.81:
   HC4: links the OpenSSL C library.
- `openssl-kdf` 0.4.2:
   category mismatch: consumer of `openssl`, `openssl-sys` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `openssl-ktls` 0.2.4:
   category mismatch: consumer of `openssl`, `openssl-sys` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `openssl-sys` 0.9.117:
   category mismatch: consumer of `aws-lc-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `opensymphony` 2.11.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `opentdf` 0.15.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `opentdf-crypto` 0.15.0:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `mbedtls`, `sha1`, `sha2` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `opentdf-kas` 0.15.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `opentdf-wasm` 0.15.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `opentimestamps` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openvet-crypto` 0.6.0:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `openvet-lockfile` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openvet-log` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `openvm-sha2-circuit` 2.0.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `openworkers-v8` 152.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ophelia-hasher-blake2b` 0.2.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `opi-ai` 0.8.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oracle_procmacro` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orca_wavebreak` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orca_whirlpools` 8.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orca_whirlpools_client` 8.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orca_whirlpools_core` 2.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orca_whirlpools_macros` 1.0.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ord-bitcoincore-rpc` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ord-bitcoincore-rpc-json` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orderly-connector-rs` 0.4.15:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ordsum` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ordvec` 0.5.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `ordvec-manifest` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ore-boost-api` 4.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ore-hq-client` 4.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ore-pool-api` 1.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ore-pool-types` 1.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ore-program` 1.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ore-rs` 0.8.3:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ore-utils` 2.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `origin-crypto-sdk` 0.6.6:
   category mismatch: consumer of `blake3`, `ring`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `orion` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `orodruin` 0.2.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `osdp` 0.3.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `osdp-embedded` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osmpbf-file-downloader-derive` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_alloc` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_arc` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_arrays` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_btree` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_entropy` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_hash_tables` 0.1.33:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `osom_lib_hashes` 0.1.33:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `osom_lib_macros` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_numbers` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_primitives` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_prng` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_reprc` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_strings` 0.1.35:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_test_helpers` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osom_lib_try_clone` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `osshkeys` 0.7.0:
   category mismatch: consumer of `aes`, `md-5`, `openssl`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `ostrya` 0.2.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ostrya-composefs` 0.2.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ostrya-core` 0.2.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ostrya-gvariant` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ostrya-rt` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ostrya-sys` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `otel-arrow-contrib-data-engine-recordset` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `otp-auth` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `otp-std` 0.2.3:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oui` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oui-data` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `outdatty` 0.4.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `over-there-auth` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `oversync` 0.6.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ovr-evm-precompile-blake2` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `owl-crypto` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxcache` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxiarc-jpeg` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxiarc-lzma` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxiaudio` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxiaudio-encode` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypt-cli` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypt-cmac` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypt-drbg` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypt-keccak-accel` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypt-sha` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypt-sha-accel` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypto` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypto-adapter-aws-lc` 0.3.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oxicrypto-adapter-pkcs11` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypto-aead` 0.3.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oxicrypto-bench` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypto-core` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicrypto-hash` 0.3.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `oxicrypto-kdf` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxicrypto-mac` 0.3.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `oxicrypto-sig` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxicuda-ann` 0.5.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxicuda-sketch` 0.5.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxiddd` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxideav-aacs` 0.1.3:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oxideav-ape` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxideav-flac` 0.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxideav-h266` 0.0.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxideav-webp` 0.2.3:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `oxidize-pdf` 5.1.2:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oxigdal-kafka` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxigeo-kafka` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `oxihuman-core` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oximedia-archive` 0.2.1:
   category mismatch: consumer of `blake3`, `crc32fast`, `md-5`, `sha1`, `sha2`, `xxhash-rust`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `oximedia-dedup` 0.2.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oximedia-watermark` 0.2.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `oximemo-core` 0.17.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oximg` 0.11.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `oxios` 2.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxirs-physics` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxirush-security` 0.1.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oxistore-blob` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxitls-adapter-rustls-rustcrypto` 0.3.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `oxitls-rcgen` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxitls-rustcrypto-provider` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxitls-webpki-roots` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxixml-support` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `oxpulse-turn` 0.2.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `oxys` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p192` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p224` 0.13.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p256` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p256k1` 7.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p2panda-blobs` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p2panda-core` 0.7.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `p2panda-discovery` 0.7.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `p2panda-encryption` 0.7.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p3-air` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-baby-bear` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-blake3` 0.7.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `p3-bn254` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-bn254-fr` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-challenger` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-commit` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-dft` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-field` 0.7.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `p3-fri` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-goldilocks` 0.7.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `p3-interpolation` 0.5.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-keccak` 0.7.0:
   HC1 for the consumed surface: `Keccak256Hash` hashes one message through scalar
   `tiny-keccak` (src/lib.rs:102-118); its AVX2, AVX-512, SSE2, and NEON modules are batched
   permutations for the proof system.
- `p3-keccak-air` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-koala-bear` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-matrix` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-maybe-rayon` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-mds` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-merkle-tree` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-miden-lifted-air` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-miden-lifted-fri` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-miden-lifted-stark` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-miden-lmcs` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-miden-stateful-hasher` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-miden-transcript` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-monty-31` 0.7.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `p3-poseidon` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-poseidon1` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-poseidon1-air` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-poseidon2` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-poseidon2-air` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-sha256` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p3-symmetric` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-uni-stark` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p3-util` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `p384` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p384_rs` 0.1.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p3q-sha3` 0.0.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `p521` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pacha` 0.4.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `package-family-name` 3.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pact-plugin-cli` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pagedb` 0.1.0-beta.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pake-cpace` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pakery-core` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pakery-crypto` 0.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `palette-arcade` 1.4.5:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `palisade-errors` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallas-crypto` 1.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallet-content` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallet-encointer-offline-payment` 24.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallet-evm-precompile-blake2` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallet-insecure-randomness-collective-flip` 37.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallet-psm` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pallet-transaction-storage` 47.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pamoja-lorawan` 0.1.18:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pamoja-ros2` 0.1.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `panic-handler` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `panproto-vcs` 0.74.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `papera` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paperboy` 0.5.5:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `papermake` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paq` 1.5.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `paqus` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `par2-rs` 0.10.3:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `paranoid` 0.0.0-pre.10:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `paranoid-hash` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paraoxidizer` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `parity-bip39` 2.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `parse-changelog` 0.6.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `parse-dockerfile` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `parselite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paserk` 0.4.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pasetors` 0.8.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `passgenz` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `passkey` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `passki` 0.3.1:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `passlane` 4.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `password-auth` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `password-hash` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `patent-hash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pathfinder-crypto` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paysec-crypto` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paysec-crypto-rustcrypto` 0.2.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `paysec-crypto-soft-aes` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `paysec-keyblock` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pbi` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pbkdf2` 0.13.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pbkdf2-identifier` 0.0.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pbkdf2-identifier-cli` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pchain-types` 0.4.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pdfbull` 0.15.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pe-sign` 0.1.10:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `peeroxide-dht` 1.7.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pem` 4.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pem-rfc7468` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `penchant-ledger` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `penis` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `perfgate-sha256` 0.15.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `perfgate-types` 0.18.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `permutation-keccak` 0.1.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `persistentcache_procmacro` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `perspt` 0.6.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pesto-poster` 0.10.3:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `pezpallet-insecure-randomness-collective-flip` 16.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pezpallet-transaction-storage` 27.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pezsc-consensus-babe` 0.34.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pezsc-consensus-beefy` 13.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pg_lens_core` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pg_lens_tui` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pg_lens_web` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pg_ripple_http` 0.136.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pg_walstream` 0.8.1:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pg-proto` 0.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pgb-iam` 0.2.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pgmacro` 0.1.13:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pgmold` 0.34.17:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pgp` 0.20.0:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha1-asm`, `sha2`, `sha3` rather than
   a hash library of its own; the underlying crate carries the screening outcome.
- `phc` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `philharmonic-connector-common` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `philharmonic-connector-service` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `philharmonic-types` 0.3.7:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `phonelib` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `phonerr` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `phoxal-bundle` 0.65.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `physis` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `physis-core` 0.1.25:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pi_agent_rust` 0.5.1:
   category mismatch: consumer of `md-5`, `ring`, `sha1`, `sha2`, `xxhash-rust` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `pilgrimage` 0.16.4:
   category mismatch: consumer of `crc32fast`, `ring`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `pincers` 0.2.0:
   category mismatch: consumer of `md-5`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `pingkeeper` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pixpack` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pkcs1` 0.7.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pkcs12` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pkcs5` 0.8.1:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `pkcs7` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pkcs8` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pkg2zip` 0.2.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pkgsrc` 0.14.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `pkloong-kcapi` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pkloong-kcapi-sys` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pktkit` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `plant-transaction-storage` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pleme-linker` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `plonky2` 1.1.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `plonky2_field` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `plonky2_maybe_rayon` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pmac` 0.8.0:
   screening survivor with `aes`: AES-PMAC, 128-bit native output, streaming, parallel
   block-cipher calls through the `aes` backend.
- `pmacct-prometheus` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pmacro_ruly` 2.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pmcp-code-mode` 0.5.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pmcp-package` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pmtree` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pna` 0.39.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pocx_hashlib` 1.0.5:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `pokemon-synthesizer` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polar-bear-hft-crypto` 0.3.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polkadot-p2p-connect` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `poly-kv` 0.1.0-alpha.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `poly-mcp` 0.5.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `poly-tools` 0.5.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `poly1305` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `poly1305-nostd` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polydat-core` 0.3.2:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `polydat-nodes` 0.3.2:
   category mismatch: consumer of `md-5`, `sha2`, `xxhash-rust` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `polyester-sdk` 0.1.0-alpha.48:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polyforge-cli` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polyforge-core` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polygraphia` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polyhash` 0.3.1:
   category mismatch: POLYVAL and GHASH universal hashes.
- `polymarket-rs-sdk` 0.1.14:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polymarket-sdk` 0.0.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polynode` 0.17.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polyoxide-clob` 0.32.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polyoxide-core` 0.32.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `polyoxide-relay` 0.32.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polyval` 0.7.3:
   category mismatch: POLYVAL universal hash over GF(2^128), a MAC building block needing a key
   and a padding scheme chosen by the consumer.
- `polyvalid` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polyvalue` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polyvers` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `polyvers-macros` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pond-db` 0.18.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pontemesh-sdk-core` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `poprf-ristretto` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `poprf-ristretto-ffi` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `portable-network-archive` 0.39.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `portage-metadata` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `postgres-protocol-sm3` 0.6.6-1:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `postgresql_archive` 0.21.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `posvault_handler` 2.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pow_account` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pow_sha256` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pow-buster` 0.2.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `PoWerRuSt` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pptx` 0.1.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pptx-rs2` 0.5.0:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ppv-lite86` 0.2.21:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pq-mceliece` 0.3.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `pq-modern-rust-tls` 0.1.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pq-transport-gateway` 2.0.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pq-xmss` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pqc_sphincsplus` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pqc-kem` 0.2.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pqc-sig` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqc-wordlist` 1.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pqcrypto` 0.18.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-dilithium` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-falcon` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-hqc` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-kyber` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-mldsa` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-mlkem` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-sphincsplus` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqcrypto-sphincsplus-wasi` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `pqcrypto-traits` 0.3.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqfile` 4.3.4:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `pqgd` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqhybridsign` 0.0.0-rc9:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pqrascv-bitcoin-anchor` 1.0.0-rc.8:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pqrascv-cli` 1.0.0-rc.8:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pqrascv-core` 0.2.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pqrascv-hardware` 1.0.0-rc.8:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pqrascv-sigstore-client` 1.0.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqrascv-verifier` 1.0.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pqxdh-zoa` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prefixed-api-key` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `preimage` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `premise_web3` 0.1.5:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pretty-sha2` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `primadb` 0.1.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `primefield` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `primeorder` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-atomic` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-clock` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-concurrent` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-config` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-core` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-function` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-retry` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-rust-atomic` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prism3-value` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `priv_osom_lib_reprc_proc_macros` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `priv_osom_lib_try_clone_proc_macros` 0.1.33:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `privacypass` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `probabilistic_data_structures` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `probability-rs` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `probminhash` 0.1.12:
   category mismatch: consumer of `sha2`, `twox-hash` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `processkit-cli` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `procmac` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `procmachines` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `procmacro2` 99.9.99:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `product-os-random` 0.0.32:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `product-os-security` 0.0.67:
   category mismatch: consumer of `openssl`, `ring`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `profiling-procmacros` 1.0.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `progpow_verifier` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `prolly-map` 0.7.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `prolog8` 26.7.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `prometheus-endpoint` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `promocrypt-core` 1.1.0:
   category mismatch: consumer of `crc32fast`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `proof_system` 0.34.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `proof-fair` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `prooflite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `proto-blue-crypto` 0.3.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `protoflow-core` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `protoflow-derive` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `proton-drive-rs` 0.6.4:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `proton-sdk` 0.6.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `prov-fixity` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `provenance-mark` 0.24.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `provii-crypto-public-inputs` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pruefung` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `psa-crypto` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `psa-crypto-sys` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pso-protocol` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `psph` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pubky-noise` 0.1.0-rc8:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pubnub` 0.8.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pumas` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `purecrypto` 0.9.0:
   screening survivor for SHA-256 and SHA-1 (x86 SHA-NI and aarch64 sha2,
   `src/hash/sha_hw.rs`); its BLAKE3 SIMD is x86_64 only and its SHA-512 hardware path is
   aarch64 only (HC1).
- `puressh` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `purl` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `purrdf-shapes` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `puu-installer` 1.0.50:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pw` 1.1.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `pw_hash` 0.1.3:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `pwgen2` 0.8.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pwhash` 1.0.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pwtool` 0.13.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `pyo3` 0.29.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `pyth-lazer-client` 26.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-backends` 0.1.5:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `q-periapt-cli` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-core` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-ffi` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-kem` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-mlkem-native-sys` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-policy` 0.1.5:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `q-periapt-rustls` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-sig` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `q-periapt-wasm` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qae-kernel` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qcicada` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qcs-api` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qcs-api-client-openapi` 0.24.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qex` 0.30.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qfe` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qhermes-kernel` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `qiniu-cdn-manager` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qiniu-upload-util` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qlean` 0.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qncke` 0.1.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qobuz-api` 2.0.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qos_crypto` 0.14.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qp-human-checkphrase` 2.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qp-poseidon` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qrc-opensource-rs` 0.3.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qrptonote` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qssl` 0.2.0:
   category mismatch: consumer of `ring`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `qt-artifacts` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qtum_hashes` 0.12.0-qtum:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quai-abi` 0.1.0-alpha.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quai-crypto` 0.1.0-alpha.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quai-keystore` 0.1.0-alpha.3:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quai-wallet` 0.1.0-alpha.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quantacore-sdk` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quantica` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `quantum-shield` 0.3.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quantum-sign` 0.1.7:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `quantus-cli` 2.2.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `quantz-sha3` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quarithmetic` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `qudag-crypto` 0.5.1:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `qudag-vault-core` 0.5.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `queue-runtime` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quic-parser` 0.1.5:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `ring` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `quichash` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quichash-core` 0.1.2:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3`, `xxhash-rust`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `quickcrypt` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quickdash` 0.6.1:
   category mismatch: consumer of `blake3`, `crc32fast`, `md-5`, `sha2`, `sha3`, `xxhash-rust`
   rather than a hash library of its own; the underlying crate carries the screening outcome.
- `quicktransform` 1.0.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `quickxorhash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quilt-rs` 0.39.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `quincy` 3.0.3:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `quincy-client` 3.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quincy-gui` 3.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quincy-server` 3.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `quipu` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `quipu-voprf` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `quorum-crypto-core` 0.7.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `qux-pqc` 1.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `qv-core` 4.3.10:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `r1cs` 0.4.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `r255b3` 0.3.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `r2kit` 0.2.1:
   category mismatch: consumer of `crc32fast`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `ra2a` 0.10.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rabbitmq_http_client` 0.92.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rabbitmqadmin` 2.35.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rabe` 0.4.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rabe-bn` 0.4.23:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `radicle-artifact-core` 0.18.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `radius-tokio` 0.1.0:
   category mismatch: consumer of `aws-lc-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rails-cookie-parser` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rainbowterm` 0.2.26:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rama-boring` 0.6.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rand_hash` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rand_seeder` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `randevu` 2.0.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rapidhash` 4.5.1:
   HC1 and HC2: scalar, 64-bit.
- `rapina` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `raptor-cli` 0.1.91:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rash` 0.6.0:
   category mismatch: consumer of `md-5`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rasypt-lite-cli` 1.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rasypt-lite-lib` 1.1.1:
   category mismatch: consumer of `aes`, `ghash`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ratify` 2.7.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `ratify-protocol` 1.0.0-alpha.20:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rattler_build_source_cache` 0.1.12:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rav1d-safe` 0.6.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rayfish` 0.1.5:
   category mismatch: consumer of `ahash`, `blake3`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `raylib` 6.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `razermacos` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rblake2sum` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rblake3sum` 0.4.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rc2` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rc4` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rcrypto` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rcypher` 0.4.1:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rcypher-cli` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rd-rds` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rdar` 0.7.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rdf-canon` 0.15.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rdfless` 0.4.34:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `readable-hash` 0.10.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `readany` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reallyme-codec` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reallyme-cose` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reallyme-crypto` 0.3.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reallyme-crypto-dispatch` 0.3.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reallyme-crypto-hpke` 0.3.9:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `reallyme-crypto-proto` 0.3.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reallyme-crypto-sha3` 0.3.9:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reallyme-crypto-sha3-256` 0.3.9:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `recoco` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `recoco-core` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `recrypt` 0.16.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reddsa` 0.5.2:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `redgold-schema` 0.1.48:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `redicat` 0.4.2:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `redismodule_cmd_procmacros` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redjubjub` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-hkdf` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-hkdf-arm` 0.1.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-hkdf-core` 0.1.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-hkdf-rust` 0.1.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-hkdf-wycheproof` 0.1.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-hkdf-x86` 0.1.0-rc.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `redoubt-rand` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reed-solomon` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ref-solver` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reflow_assets` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `refset` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reinhardt-auth` 0.3.18:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reinhardt-core` 0.3.18:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reinhardt-deeplink` 0.3.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reinhardt-middleware` 0.3.18:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `reishi-handshake` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `relational_types_procmacro` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `relaunch` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `relayly` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `remnant-core` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `remoc` 0.20.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `renderreport` 0.5.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `renegade-sdk` 0.1.16:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `renet` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `renet2` 0.16.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `renet2_netcode` 0.16.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `renetcode` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `repl-lsp` 1.20.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `repl-proto` 1.20.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reqsign-aliyun-oss` 3.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reqtls` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `reqwless` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rescue_poseidon` 0.32.11:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `resender` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `residiuum-format` 0.2.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `residiuum-snapshot-format` 0.1.0-alpha.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `resource_proof` 1.0.39:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `resurrect-core` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `resurrect-ethereum` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `resurrect-libp2p` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `resurrect-node` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `retail-mac` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `retrospector` 1.1.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `revenant-sign-tls` 4.0.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `reverse_resonance_id` 0.1.0:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `revm-inspectors` 0.43.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rex` 4.0.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rfc2289-otp` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rfc6979` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rfc7693` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rgp` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rialo-s-keccak-hasher` 0.18.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rialo-s-sha256-hasher` 0.18.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `riemann_client` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rigel` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `rightsize` 0.7.9:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rightsize-docker` 0.7.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rightsize-modules` 0.7.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rightsize-msb` 0.7.9:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ring` 0.17.14:
   HC4: compiles C and perlasm-generated assembly through `cc`.
- `ring-compat` 0.8.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ring-digest` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `ringpcx` 0.17.14:
   HC4: a fork of `ring`, which compiles C and assembly through `cc`.
- `rings-core` 0.20.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ripemd` 0.2.0:
   HC1: portable.
- `ripemd128` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `risc0-circuit-keccak` 4.0.6:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `risc0-circuit-keccak-sys` 4.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `risc0-crypto` 0.1.0-rc.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `risc0-crypto-evm` 0.1.0-rc.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `risc0-zeroio` 0.14.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `risc0-zkp-core` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ritehash` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `river-core` 0.1.21:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `riverctl` 0.2.18:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rkik` 2.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rkik-nts` 1.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rlg-report` 0.0.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rlibphonenumber` 2.2.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rlnc-cat-rs` 0.2.4:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rndc` 0.1.5:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rneter` 0.5.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rnp-rs` 0.1.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `robocomp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `robocomp_avian3d` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `robocomp_rapier3d` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `robust_downloader` 0.0.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rockchip-pm` 0.4.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rocket-slogger` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rok-hash` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `roka-totp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rolling-token-auth` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rolodex-dns` 0.6.3:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rom-weaver-checksum` 0.15.1:
   category mismatch: consumer of `blake3`, `crc32fast`, `md-5`, `sha1`, `sha2` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `rona` 2.35.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rong_jscore_sys` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ronkathon` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rooster` 2.14.1:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rootchain-crypto` 1.0.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rootle` 0.12.2:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ros2-types` 0.5.7:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rosalind-receipt` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `roughenough` 1.1.8:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rp-pio-serial` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rp-postgrest-error` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rpecli` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rpm` 0.28.0:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rpmrepo_metadata` 0.7.0:
   category mismatch: consumer of `ahash`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `rs_blake2` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_hasher_ctx` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_keccak_nbits` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_md5` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_poke` 0.2.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha1` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha256` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha3_224` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha3_256` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha3_384` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha3_512` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha384` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha512` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha512_224` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sha512_256` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_shake128` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_shake256` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_shield` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_sm3` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs_ssl` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `rs-builder-signing-sdk` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rs-io` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs-suno` 0.42.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rs-x11-hash` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsa` 0.9.10:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsa-der` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsa-fdh` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsacracker` 0.9.1:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsasl` 2.3.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rscrypto` 0.9.0:
   screening survivor: XXH3-128 (AVX-512, AVX2, NEON, run-time dispatch), BLAKE3 (AVX2 and
   AVX-512 `global_asm!`, NEON intrinsics, optional rayon tree mode), SHA-256 (SHA-NI, aarch64
   sha2), SHA-512 (AVX2, AVX-512VL, aarch64 sha3); zero runtime dependencies, MIT OR Apache-2.0.
- `rsearch-common` 0.8.1:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsearch-index` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsearch-ingest` 0.8.1:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsearch-metastore` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsearch-search` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsearch-server` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsearch-storage` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsemu` 0.0.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rshash` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `rsigma-eval` 0.22.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rsip` 0.4.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsipclient` 2.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rskit-util` 0.2.0-alpha.6:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsleigh` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsmanuf` 2026.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsmd5` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsomics-liftover` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsos` 0.4.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rsteria2` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rstix` 0.22.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rsupd` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rsurl` 0.1.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rtc` 0.20.5:
   category mismatch: consumer of `aws-lc-rs`, `ring`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `rtime-nts` 0.15.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rtk-cli` 0.1.2:
   category mismatch: consumer of `openssl`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `rtp-engine` 0.1.0:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rubrail` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-ast` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-compiler` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-diagnostic` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-hir` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-lexer` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-lir` 0.10.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rue-options` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-parser` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rue-types` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rufield-provenance` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ruipmi` 0.6.2:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `rulake` 2.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rullst` 12.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rullst-capital` 12.0.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rullst-security` 12.0.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ruma-signatures` 0.22.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rumqttc-v5-next` 0.34.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rune-axum-signature` 0.1.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rune-blake3` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rune-md5` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rune-sha1` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rune-sha256` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rune-sha512` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `runner-run` 0.26.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ruoqa` 0.3.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ruscrypt` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusotp` 0.5.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rusqlcipher` 0.14.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `russh` 0.63.3:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `ghash`, `keccak`, `polyval`, `ring`,
   `sha1`, `sha2`, `sha3` rather than a hash library of its own; the underlying crate carries
   the screening outcome.
- `russh-extra` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust_cascade` 1.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rust_ctf_party` 0.2.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rust_kits` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust_md5` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust_md5_updated` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust_sm` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust_sodium` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust_sodium-sys` 0.10.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-2fa-hotp-totp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-aes-keywrap` 0.1.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rust-argon2` 3.0.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rust-bigint` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-bip39` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-bls-bn254` 0.2.1:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rust-clacc` 3.6.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rust-crypto` 0.2.36:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rust-crypto-hatter-fork` 0.2.36:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rust-crypto-wasm` 0.3.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rust-cryptoauthlib` 0.4.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-geo-prep` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-hashcash` 0.3.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rust-libteec` 0.6.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-merkle` 0.1.7:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rust-otp` 3.0.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rust-par2` 0.1.3:
   category mismatch: consumer of `crc32fast`, `md-5` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rust-rapidsnark` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rust-sanitize` 0.15.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rust-sign` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rust-witness` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusta-cli` 1.3.18:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustbinary` 0.1.8:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rustbinary-derive` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustbus` 0.19.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustc-hash` 2.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustdom-x-argon2` 2.1.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rustdrivesync` 1.1.1:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustfs-utils` 1.0.0:
   category mismatch: consumer of `crc-fast`, `highway`, `md-5`, `sha1`, `sha2` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `rustfsm_procmacro` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustgenhash` 0.16.0:
   category mismatch: consumer of `aes`, `blake3`, `md-5`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `rustgenpass` 0.6.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rustkey` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustledger-ffi-wasi` 0.24.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustls` 0.23.45:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rustls-ccm` 0.2.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustls-cert-gen` 0.2.0:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rustls-connector` 0.23.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustls-fork-shadow-tls` 0.20.8:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustls-mbedcrypto-provider` 0.1.1:
   category mismatch: consumer of `mbedtls` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rustls-mbedpki-provider` 0.2.1:
   category mismatch: consumer of `mbedtls` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rustls-mbedtls-provider-utils` 0.2.1:
   category mismatch: consumer of `mbedtls` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rustls-native-certs` 0.8.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustls-native-ossl` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustls-pemfile` 2.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustls-pki-types` 1.15.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustls-rustcrypto` 0.0.2-alpha:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustls-symcrypt` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustls-webpki` 0.103.15:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rustnetconf` 0.17.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustpatcher` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustpatcher-macros` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustretro-procmacro` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rustssh2` 9.0.0:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `ring`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `rusty_av1e` 0.8.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `rusty_crypto` 0.1.24:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `Rusty_CryptoAuthLib` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusty_flac` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusty_h265` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusty_time-nts` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusty_vault` 0.2.1:
   category mismatch: consumer of `blake2b_simd`, `openssl`, `openssl-sys` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `rusty-money` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rusty-sidekiq` 0.15.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rustyhash` 0.1.1:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `rustywallet-mnemonic` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ruvector-graph-transformer` 2.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ruvector-rulake` 2.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ruvix-types` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rvf-crypto` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rvn` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rvoip-rtp-core` 0.3.10:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `rvoip-sip-core` 0.3.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rvoip-users-core` 0.3.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rvoip-vcon` 0.3.10:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rvp9-io` 0.2.0-alpha.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rvpk` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `rw-builder` 0.3.0:
   category mismatch: consumer of `aes`, `crc32fast`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `rxls` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ryg-rans-rs-casefile` 0.5.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ryg-rans-rs-cli` 0.5.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ryg-rans-rs-oracle` 0.5.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `rzbackup` 3.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `s2protocol` 3.5.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `s3-endpoint` 0.1.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `s3etag` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `s3ls-rs` 1.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `s3sync` 1.62.1:
   category mismatch: consumer of `crc-fast`, `crc32fast`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `s3util-rs` 1.10.2:
   category mismatch: consumer of `crc-fast`, `crc32fast`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `s3v4` 0.3.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `s7cmd` 1.8.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sabry_procmacro` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sabry_procmacro_impl` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sacp-cbor` 0.18.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sad-rsa` 0.10.2:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `saferet` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `said` 0.5.2:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `salsa20` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `salty` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `saml` 0.0.1-alpha.2:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `saml-rs` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `samlify` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sanitization-crypto-interop` 2.1.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `saorsa-core` 0.27.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `saorsa-fec` 0.4.14:
   category mismatch: consumer of `blake3`, `crc32fast`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `saorsa-mls` 0.3.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `saorsa-pqc` 0.5.1:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `saorsa-rsps` 0.2.3:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `saorsa-transport` 0.36.4:
   category mismatch: consumer of `aws-lc-rs`, `blake3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `sapio-bitcoin` 0.28.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sapio-secp256k1` 0.28.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sapphire-framework-blob` 0.14.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sapphire-hash` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sasl-scram` 0.0.1-rc.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `satsnet` 0.32.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `satsnet_hashes` 0.14.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `save_my_code` 2.0.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `sb-aes-gcm-siv` 0.10.3:
   category mismatch: consumer of `aes`, `polyval` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sb-curve25519-dalek` 3.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sbf` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sbf-blake3` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sbo3l-core` 1.2.2:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sbo3l-policy` 1.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sbo3l-storage` 1.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sbom-auditor` 1.6.24:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `sbom-tools` 0.2.0:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `sc_drbg` 0.1.0-alpha.2:
   category mismatch: consumer of `aes`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sc-consensus-babe` 0.59.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sc-consensus-beefy` 38.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sc-hop` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sc-sha` 1.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scarb-stable-hash` 1.0.0:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `scema-anchor` 1.24.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sceptre` 0.7.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `schemapin` 1.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `schnorr_fun` 0.13.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `schnorrkel` 0.11.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scirs2-datasets` 0.6.5:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `scirs2-io` 0.6.5:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `scll-backend-rustcrypto` 1.0.1:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scoped-threadpool-std` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scorpiofs` 0.4.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scour-secrets` 0.20.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scout-cli` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scram` 0.6.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scram-2` 0.7.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scram-rs` 0.24.0:
   category mismatch: consumer of `aws-lc-rs`, `md-5`, `openssl`, `ring`, `sha1`, `sha2`,
   `sha3` rather than a hash library of its own; the underlying crate carries the screening
   outcome.
- `screenshot-rs` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scry-sai-provenance` 3.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scryer-plugin-pdk` 0.5.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scryer-prolog` 0.10.0:
   category mismatch: consumer of `ring`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `scrypt` 0.12.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scryptenc` 0.10.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `scsynth-wasm32-libc` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scsynth-wasm32-libcxx` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scsys-crypto` 0.3.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sct` 0.7.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `scx_pandemonium` 5.18.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `scytale` 0.5.0:
   screening survivor for SHA-256 (x86_64 SHA-NI, aarch64 Armv8 sha2, run-time probe); SHA-512
   has no x86_64 accelerated path (HC1), SHA-3 is aarch64-only (HC1).
- `sdk-4mica` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `seal-crypto` 0.1.5:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `seam-engine` 0.5.38:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sec1` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secapi` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secapi-sys` 0.4.0:
   category mismatch: consumer of `openssl-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `secp` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secp256k1` 0.33.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secp256k1-plus` 0.5.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secp256k1-zkp` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secp256kfun_k256_backend` 2.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secrecy` 0.10.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secret-toolkit-crypto` 0.10.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `secret-vault` 2.1.0:
   category mismatch: consumer of `ahash`, `ring` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `secret-vault-value` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secrets` 1.3.0:
   category mismatch: consumer of `libsodium-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `secrets-app` 0.16.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secrets-vault` 2.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `secretstash-cli` 2.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secure-enclave-rs` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `secure-gate` 0.8.0-rc.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `securerand-rs` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `securestore` 0.100.3:
   category mismatch: consumer of `aes`, `sha1`, `openssl` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `security-framework` 3.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `security-framework-sys` 2.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `security-translocate` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `security-translocate-sys` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `seedlock` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `seekable-stream-cipher` 0.2.4:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `seiza-cabi` 0.18.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `self_update` 1.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `self_upgrade` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `semaphore-rs-ark-zkey` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `semaphore-rs-keccak` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `semdup` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `semtree-rag` 0.5.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sentinel-cli` 2.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sentinel-crypto` 2.1.1:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sentinel-dbms` 2.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sentinel-wal` 2.1.1:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `seqproc` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sequoia-cert-store` 0.7.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sequoia-chameleon-gnupg` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sequoia-net` 0.30.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sequoia-octopus-librnp` 1.11.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sequoia-openpgp` 2.4.1:
   category mismatch: consumer of `aes`, `botan`, `md-5`, `nettle`, `openssl-sys`, `sha2`,
   `sha3`, `xxhash-rust` rather than a hash library of its own; the underlying crate carries
   the screening outcome.
- `sequoia-policy-config` 0.8.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sequoia-sq` 1.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sequoia-wot` 0.15.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `serbero` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `serde_hash` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `serde_json_canonicalizer` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `serdect` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `serializer` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `serpent` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `servicenow-cli` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `setsum` 0.9.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sgp4` 2.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha_256` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha_256_scratch` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha_file_hashing` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha-1` 0.10.1:
   deprecated wrapper for `sha1`.
- `sha-crypt` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha-rs` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha1` 0.11.0:
   screening survivor: SHA-1 through x86 SHA-NI and the aarch64 sha2 extension with run-time
   detection.
- `sha1_cracker` 0.1.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha1_hab` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha1_smol` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha1-asm` 0.5.3:
   HC4: compression function in assembly compiled by `cc`.
- `sha1-checked` 0.10.0:
   category: SHA-1 with collision detection over `sha1`, which more than doubles its cost;
   evaluated through `sha1`.
- `sha1-checksumdir` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha1-extensions` 0.2.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha1-hasher` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `sha1-hasher-faster` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `sha1-macros` 1.0.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha1collisiondetection` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha2` 0.11.0:
   screening survivor: SHA-256 through x86 SHA-NI and aarch64 sha2, SHA-512 through x86 AVX2
   and the aarch64 sha3 extension, run-time detection with `cpufeatures`.
- `sha2_ce` 0.10.6:
   HC4: SHA-2 through `sha2-asm`.
- `sha2_hasher` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha2-asm` 0.6.4:
   HC4: compression functions in assembly compiled by `cc`.
- `sha2-compress` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha2-const` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha2-const-stable` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha2-fv` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha256` 1.6.0:
   category: wrapper over `sha2` or OpenSSL; evaluated through `sha2`.
- `sha256-rs` 1.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha256-viz` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha256fast` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha256sum_from_scratch` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha2ni` 0.8.5:
   HC1 and HC4: x86-only SHA-NI through `sha2-asm`.
- `sha2raw` 14.1.0:
   HC4: SHA-2 through `sha2-asm`; also an unpadded-block API.
- `sha3` 0.12.0:
   HC1 for the consumed surface: SHA-3 and SHAKE over `keccak`, whose default x86 backend is
   scalar; the aarch64 sha3 path alone does not satisfy both architectures.
- `sha3_ce` 0.10.6:
   HC1: same `keccak` backends as `sha3`.
- `sha3-asm` 0.1.8:
   HC4: assembly bindings compiled by `cc`.
- `sha3-circuit` 0.2.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sha3-const` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha3-extensions` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha3-kernel-hasher` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha3-kmac` 0.3.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha3-literal` 0.1.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sha3-plus` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `sha3-rust` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha3-selkie` 0.0.0:
   HC7: published only as version 0.0.0, which carries no usable release.
- `sha3-utils` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sha3sum` 1.3.3:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `shabal` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shadertoy` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shadertoy-browser` 0.6.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `shadowrocks` 0.1.0:
   category mismatch: consumer of `openssl`, `ring`, `sodiumoxide` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `shadowsocks-crypto` 0.8.0:
   category mismatch: consumer of `aes`, `aws-lc-rs`, `blake3`, `ghash`, `md-5`, `sha1` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `shadowvpn` 0.6.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `shaha` 0.2.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `shairplay` 0.10.0:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `shake` 0.1.0:
   HC1: same `keccak` backends as `sha3`.
- `shaman` 0.1.0:
   HC1: SHA-2 through the 2015-era `simd` abstraction, no aarch64 intrinsics.
- `SHARAG256` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shardline-auth` 1.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shared-aes-enc` 0.3.8:
   category mismatch: consumer of `aes`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sharks` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shash` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shashasha` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shasum` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sheesy-cli` 4.0.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shell-tunnel` 0.25.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `shepherd-render` 6.7.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sherpa-onnx-models` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shield-core` 2.2.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `shielded` 0.1.2:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `shiguredo_cmake` 4.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `shiguredo_s3` 2026.1.0-canary.7:
   category mismatch: consumer of `aws-lc-rs`, `crc-fast`, `md-5`, `sha1`, `sha2` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `shin` 0.17.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ship` 0.3.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `shiplog-ids` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `short-crypt` 1.0.29:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `si-crypto-hashes` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sigh` 1.0.3:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sigma_fun` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sigmate` 1.0.0:
   category mismatch: consumer of `openssl`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `signatory` 0.27.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signatory_kit` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signature` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signature_bbs_plus` 0.37.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signature_bls` 0.35.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signature_core` 0.37.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signature_derive` 2.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signature_ps` 0.35.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-aptos` 3.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-arweave` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-btc` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-casper` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-cli` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-cosmos` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-evm` 3.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-fil` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-nostr` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-primitives` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-spark` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signer-sui` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-svm` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-ton` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `signer-tron` 3.2.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `signer-xrpl` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signet-core` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `signinwithethereum` 0.8.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sigstore-crypto` 0.11.0:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `sigstore-rekor` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sigstore-tsa` 0.11.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `silero-vad-rust` 6.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `silicon-iam-client` 1.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `silk-graph` 0.5.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `simdutf8-cli` 1.3.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `simhash` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `simple_crypt` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `simple_download_utility` 0.1.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `simple_hasher` 1.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `simple-file-encrypt` 0.2.0:
   category mismatch: consumer of `aes`, `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `simple-http-server` 0.8.1:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `simple-srp` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `simple-web` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `simplehash` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sinowealth-kb-tool` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sinowisp` 2.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sinsemilla` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sip-client` 2.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `siphasher` 1.0.3:
   HC1: scalar SipHash.
- `siphon-sip` 1.9.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `siprs-registration` 0.1.2:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sirraya-crypto` 0.1.3:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `size_lru` 0.1.38:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sk_dkim` 0.1.8:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sk-pqc` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `skeg-platform` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `skeg-resp3` 0.2.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `skeg-simd` 0.1.6:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `skeg-vector` 0.1.8:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `skein` 0.2.0:
   HC1: portable Threefish.
- `skein-ffi` 0.5.0:
   HC4: FFI binding to the Skein C implementation.
- `skein-hash` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `skesa-rs` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sketchir` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `skippydb` 0.2.2:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `sklears-neighbors` 0.2.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `sl-dkls23` 1.0.0-beta:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `slahasher` 0.5.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `slancha-wire` 0.17.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `slate-kv-crypto` 0.6.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `slh-dsa` 0.1.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `slip10_ed25519` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `slip132` 0.10.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `slip21` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sloc-git` 1.6.20:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sloc-guard` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `slop-keccak-air` 6.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `slop-poseidon2` 6.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `slop-symmetric` 6.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sloth256-189` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sm-crypto` 0.1.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sm2` 0.13.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sm2cl` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sm3` 0.5.0:
   HC1: portable.
- `sm4` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `small-collections` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `smallrand` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `smart-account-auth` 0.27.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `smb2` 0.22.1:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `smb2-client` 0.2.5:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `smcrypto` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `smtp_recv` 0.1.69:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `smugglr-core` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sn_registers` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sn0int` 0.26.1:
   category mismatch: consumer of `md-5`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `snapcast-client` 0.17.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `snapcast-proto` 0.17.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snapcast-server` 0.17.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `snapfire_typecheck` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `snarkvm` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-algorithms` 4.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `snarkvm-circuit` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-curves` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-derives` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-dpc` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-fields` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-gadgets` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-authority` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-block` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-coinbase` 0.16.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-committee` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal-batch-certificate` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal-batch-header` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal-data` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal-subdag` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal-transmission` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-narwhal-transmission-id` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-puzzle` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-puzzle-epoch` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-query` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-ledger-store` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-marlin` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-parameters` 4.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `snarkvm-polycommit` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-profiler` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-r1cs` 0.12.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-synthesizer` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-synthesizer-process` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-synthesizer-program` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-synthesizer-snark` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-utilities` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-utilities-derives` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snarkvm-wasm` 4.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snerd-rust` 0.2.5:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `snerdmq` 0.2.10:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `snmp2` 0.5.2:
   category mismatch: consumer of `aes`, `md-5`, `openssl`, `sha1`, `sha2` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `snow` 0.10.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `snowbridge-amcl` 1.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snowbridge-milagro-bls` 1.5.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `snowflake-jwt` 0.3.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sntrup` 0.4.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `sntrup761` 0.4.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `socket-patch-cli` 4.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sodalite` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sodiumbox` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sodiumoxide` 0.2.7:
   category mismatch: consumer of `libsodium-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sodoken` 0.1.0:
   category mismatch: consumer of `libsodium-sys` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `soft-aes` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `softaes` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solabi` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana` 0.17.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana_ed25519_verify` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-awesome` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-ecvrf` 0.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-ed25519` 0.2.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-ed25519-adaptor` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-ed25519-sha512` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-falcon512` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-hmac-drbg` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-hmac-sha256` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-keccak-hasher` 3.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-keychain` 1.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-nostd-keccak` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-nostd-secp256k1-recover` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-nostd-sha256` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-ratchet-anchor` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-ratchet-core` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-ratchet-quasar` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-rfc6979` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-secp256k1-ecdsa` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-secp256k1-schnorr` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-sha256-hasher` 3.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-sha512-hasher` 1.0.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `solana-shake256` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solana-winternitz` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-ast` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-compiler` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-config` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-data-structures` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-interface` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-macros` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-parse` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solar-sema` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `solid-pod-rs` 0.5.0-alpha.9:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sora-cli` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-codegen` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-config-format` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-core` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-data` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-diagnostics` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-excel` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-execution` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-export` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-input` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-input-csv` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-input-schema` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-input-structured` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-input-toml` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-input-xlsx` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-ir` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-mcp` 0.13.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sora-schema` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-studio` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-templates` 0.13.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sora-workspace` 0.13.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `soroban-fork` 0.9.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `soroban-poseidon` 27.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sorted-iter` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp-ark-bls12-377` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp-ark-bls12-381` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp-ark-bw6-761` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp-ark-ed-on-bls12-377` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp-ark-ed-on-bls12-381-bandersnatch` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp-ark-models` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sp1-solana` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sp800-185` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spacedb` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `spacedls` 0.8.0:
   category mismatch: consumer of `aes`, `ghash`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `spake2` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `spark-token-primitives` 0.1.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sparkid` 2.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spartan` 0.9.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `spartan2` 0.9.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sparx` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spectral-fingerprint` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spectreq` 0.1.1:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `spectrex` 0.3.19:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `spellbook` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spg-audit` 8.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spg-crypto` 8.0.4:
   category mismatch: application crypto bundle over BLAKE3 and CRC32.
- `sphinx-rs` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sphinxcrypto` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `spice-coordinates` 1.2.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `spideroak-crypto` 0.8.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `spki` 0.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `spodes-rs` 0.7.1:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `sponge-cursor` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sponge-hash-aes256` 1.10.7:
   HC3 not established: a bespoke AES-256 sponge with no published specification, standard, or
   output-stability statement in its README or documentation.
- `sppark` 0.1.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `spring-web` 0.4.17:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sql-splitter` 1.21.0:
   category mismatch: consumer of `ahash`, `crc32fast`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `sqlite_ext_ntqq_db` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sqlite-compressions` 0.3.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sqlite-hashes` 0.10.10:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `sqlx` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sqlx-cli` 0.9.0:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sqry-cli` 31.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `squads-rustfsm-procmacro` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `squeez` 1.48.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `srisum` 5.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `srp` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `srp6` 1.0.0-beta.2:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `srx-rs` 1.0.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ssb-crypto` 0.2.3:
   category mismatch: consumer of `libsodium-sys`, `sha2`, `sodiumoxide` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `ssh-agent-lib` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ssh-cipher` 0.3.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ssh-cipher-fork-arti` 0.2.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ssh-encoding` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ssh-encoding-fork-arti` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ssh-key` 0.6.7:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ssh-key-fork-arti` 0.6.7:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ssh-keydump` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ssh-rs` 0.5.0:
   category mismatch: consumer of `aes`, `ring`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `ssign-core` 0.1.5:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sskr` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ssri` 9.2.0:
   category mismatch: consumer of `sha2`, `xxhash-rust` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `ssri2` 0.2.0:
   category mismatch: consumer of `sha1`, `sha2`, `xxhash-rust` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `stack-auth` 0.42.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stacks-common` 0.0.3:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `starknet-crypto` 0.8.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `starknet-types-core` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `starkom-pcs` 6.0.3:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `starling` 4.0.0:
   category mismatch: consumer of `md-5`, `openssl`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `starrocks-stream-load` 1.0.13:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stateset-a2a` 1.35.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `stateset-crypto` 1.35.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `stdrandom` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stdtx` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `steam-crypto-rs` 0.1.2:
   category mismatch: consumer of `aes`, `crc32fast`, `sha1` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `steam-vdf-parser` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stellar-accounts` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stellar-contract-utils` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stellar-rust-client` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stellar-zk-core` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stemma-artifacts` 0.5.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sthash` 0.2.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stinger-mqtt-trait` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stix-rs` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stochash` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `storelib_rs` 0.1.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `storj` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stratlite` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stream-cipher` 0.99.99:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `streamforge` 1.0.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `streamget-rs` 0.2.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `streamsha` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `streebog` 0.11.0:
   HC1: portable.
- `strict_encoding` 2.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `strict_encoding_test` 2.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `stringzilla` 5.1.2:
   HC2 and HC4: 64-bit hashes through C and C++ sources.
- `strobe-rs` 0.13.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `stronghold_engine` 2.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `structdiff` 0.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `strx` 0.1.0:
   category mismatch: consumer of `aes`, `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `strykelang` 0.17.52:
   category mismatch: consumer of `aes`, `blake3`, `crc32fast`, `md-5`, `sha1`, `sha2`, `sha3`,
   `xxhash-rust` rather than a hash library of its own; the underlying crate carries the
   screening outcome.
- `stylus-registry` 0.1.0-beta.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `styrene-identity` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `subgrind` 0.1.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `substrate-benchmark-machine` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `substrate-bn` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `substrate-bn-succinct` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `substrate-bn-succinct-rs` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `substreams-abis` 1.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `subtle` 2.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `subtle-encoding` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `subtle-ng` 2.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sudp` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sui-cache-eval` 0.1.219:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sui-graph-store` 0.1.219:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sumatradigest` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sumchain-crypto` 0.4.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `sumchain-wire` 0.4.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `suminuri-wire` 0.1.27:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `summer-cos` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `summer-web` 0.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `suno-core` 0.42.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sunscreen_curve25519` 0.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sunset` 0.6.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `super-speedy-syslog-searcher_ere_automator_procmacro` 0.10.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `SuperBit` 0.1.1:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `superbit_lsh` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `superboring` 0.1.14:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `superposition_libaccounts` 0.10.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `supply-chain-trust-example-crate-000047` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `supply-chain-trust-example-crate-000069` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `supply-chain-trust-example-crate-000075` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `sure25` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sutegi-crypto` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `sv` 0.2.2:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `svgmacro` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swarmforge-sha1-wrapper` 0.1.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `swc-plugin-formatjs` 10.5.423:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `swh-digestmap` 0.3.5:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `swh-mosaic` 0.5.0:
   category mismatch: consumer of `crc32fast`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `swh-osv` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swift-bridge` 0.1.59:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swift-bridge-build` 0.1.59:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swift-bridge-cli` 0.1.59:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swift-bridge-ir` 0.1.59:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swift-bridge-macro` 0.1.59:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swiftness` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swiss-table` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `swt3-ai` 0.7.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `syd` 3.59.0:
   category mismatch: consumer of `ahash`, `blake3`, `md-5`, `sha1`, `sha2`, `sha3` rather than
   a hash library of its own; the underlying crate carries the screening outcome.
- `symbi` 1.20.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `symcrypt` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `syn-solidity` 1.7.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `synadb` 1.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `synap-sdk` 1.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `synrepo` 0.2.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `synta-certificate` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `synta-krb5` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `synta-mtc` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `synta-python` 0.3.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `synta-x509-verification` 0.3.4:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `syntheca` 0.3.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `systemprompt-models` 0.54.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `systemprompt-security` 0.54.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `systemprompt-sync` 0.22.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `sz-orm-crypto` 7.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `t1ha` 0.1.2:
   HC1 (x86 intrinsics only) and HC3.
- `tabbyssl` 0.10.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `taceo-oprf-types` 0.16.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tacet-cli` 0.1.29:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tameshi` 0.1.5:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tamga` 0.3.5:
   category mismatch: consumer of `aws-lc-rs`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tamper-audit` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tape-sha256` 0.2.1:
   HC10 and category: batch (`hash_many`) and hash-chain APIs only, no single-message streaming
   state.
- `tapwarden` 0.2.4:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tar-sha256` 0.1.0-a2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tari_bulletproofs` 4.4.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tari_merlin` 4.0.0:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tari-curve25519-dalek` 4.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tari-tiny-keccak` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tarsum` 0.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tarzan` 0.5.0:
   category mismatch: consumer of `sha2`, `twox-hash` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tatara-lisp-script` 0.3.58:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `tauri-plugin-auth-session` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tauri-plugin-clerk` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tauri-plugin-deep-link` 2.4.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tauri-plugin-google-auth` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tauri-plugin-hwinfo` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tc_runtime` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tc-consensus-babe` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tdln-ast` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tdln-proof` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tds-protocol` 0.20.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tectonic-bedrock` 0.5.3:
   category mismatch: consumer of `aes`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tellaro-query-language` 2.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `temari` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tencentcloud-sign-sdk` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tendermint` 0.40.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tendermint-proto` 0.40.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tenhou-shuffle` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tenset_merkletree` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tensor-id` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tenthash` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tenzro-crypto` 0.2.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tenzro-sdk` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tenzro-storage` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tenzro-types` 0.2.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tephra-client` 0.5.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tequel-rs` 2.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ter-music-rust` 2.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ternary-hash` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `terseid` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `test_kms_server` 5.27.0:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `test-curve25519-dalek` 4.0.0-rc.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-dalek-docs` 4.0.0-pre.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-ed25519-dalek` 2.0.0-pre.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `test-foo-bar` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-foo-bar-blake2` 0.0.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-foo-bar-digest` 0.0.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-foo-bar-hacl-rs` 0.0.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-foo-bar-intrinsics` 0.0.8:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `test-foo-bar-macros` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-foo-bar-platform` 0.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `test-foo-bar-sha3` 0.0.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tet-application-crypto` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tet-core` 2.1.2:
   category mismatch: consumer of `sha2`, `twox-hash` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tet-io` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetcore-database` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetcore-std` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetcore-storage` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetcore-tracing` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetcore-utils` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetcore-wasm-interface` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetsy-keccak-hash` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tetsy-keccak-hasher` 0.15.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tezos_crypto_rs` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tf-types` 0.1.8:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `tfhe` 1.8.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `tfhe-ark-ec` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tfhe-csprng` 0.10.0:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `thalovant` 0.10.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `thash` 0.2.5:
   category mismatch: consumer of `blake3`, `k12`, `md-5`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `theway-contract` 0.1.27:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `thirdkind` 3.13.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `thread` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `threatflux-hashing` 1.7.0:
   category mismatch: consumer of `blake3`, `md-5`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `threatflux-string-analysis` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `threatflux-vertex-rust-sdk` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `threefish` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `threencr` 1.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tiberius-raw-bulk` 0.12.3-raw-bulk.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tibet-cortex-core` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tibet-zip-airlock` 2.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tibet-zip-core` 2.4.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tibet-zip-jis` 2.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tibet-zip-mirror` 2.4.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tidecoin` 0.33.0-beta:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tidecoin_hashes` 0.20.0:
   duplicate fork of `bitcoin_hashes`; screened through it.
- `tidepool-core` 0.4.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tiger` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tiger-digest` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tightbeam-rs` 0.14.0:
   category mismatch: consumer of `aes`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tii-procmacro` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tilecoding` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-aead` 0.3.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tink-core` 0.3.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tink-daead` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-ffi` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-hybrid` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-mac` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-prf` 0.3.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tink-proto` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-signature` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tink-streaming-aead` 0.3.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tiny_ed448_goldilocks` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tiny-crypto` 0.1.3:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tiny-keccak` 2.0.2:
   HC1: scalar Keccak, no architecture intrinsics.
- `tiny-merkle` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tiny-multihash` 0.5.0:
   category mismatch: consumer of `blake2b_simd`, `blake2s_simd`, `blake3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `tinycache` 0.1.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tirami-core` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tirith` 0.4.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tirith-core` 0.4.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tiss-hash` 0.1.0:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tlpsign` 1.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tlrc` 1.13.1:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tlsferret` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tlsh2` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tmp-curve25519-dalek-h2c-do-not-use` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `tmp-ed25519` 1.0.0-pre.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tmpltool` 1.5.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tnt-bls` 0.1.8:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `toad-hash` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `toast` 0.48.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `toast-api` 0.1.9:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `todoist-api` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `todozi` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tohu` 0.0.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tokenomics-simulator` 0.5.12:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tokio-gaussdb` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tokio-postgres-rustls-improved` 0.16.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tokio-rustls` 0.26.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tokio-websockets` 0.13.3:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `tokmd-content` 1.9.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tokmd-redact` 1.9.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tomacto` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tomesole` 0.1.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ton` 0.4.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ton_core` 0.3.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ton_lib` 0.0.38:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ton_macros` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tonic-mock` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `toolpath-claude` 0.12.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `toolu-orm-connection` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `toolu-orm-core` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `toolu-orm-macros` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `toolu-orm-query` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tor-cell` 0.46.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tor-hsclient` 0.46.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tor-hscrypto` 0.46.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tor-hsservice` 0.46.0:
   category mismatch: consumer of `k12` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tor-key-forge` 0.46.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tor-llcrypto` 0.46.0:
   category mismatch: consumer of `aes`, `openssl`, `sha1`, `sha2`, `sha3` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `tor-relay-crypto` 0.46.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `torchbear` 0.11.5:
   category mismatch: consumer of `openssl`, `sodiumoxide` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `torm` 0.4.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `torm-derive` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tors-core` 0.7.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2`, `twox-hash` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `torsh-hub` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `totp_rfc6238` 0.7.0:
   category mismatch: consumer of `ring`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `totp-lite` 2.0.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `totp-qr` 0.2.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `totp-rs` 6.0.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `totp-sm-rs` 0.1.5:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `totper` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `totpyx` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `toucHNews` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-api` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-consensus` 0.8.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-inherents` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-keystore` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-runtime` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-runtime-interface` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-runtime-interface-proc-macro` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-state-machine` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-trie` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tp-version` 2.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tpcdsgen` 0.1.0-alpha.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tpfs_log_event_procmacro` 0.1.19:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tpt-cv-feature` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tpt-telos-sdk` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tr300` 4.3.12:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tracehash-rs` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tracehash-rs-derive` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `transit_model_procmacro` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `traversev` 0.1.0-alpha.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tree_hash` 0.12.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tree_hash_derive` 0.12.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tree-sitter-irules` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tree-type` 0.4.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `treetop-bundle` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `trevo-keyless` 0.8.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `trevokeyless` 2.0.0:
   category mismatch: consumer of `blake2b_simd` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `trezoa-keccak-hasher` 3.2.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `trezoa-sha256-hasher` 3.2.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `trezor-noise-rust-crypto` 0.6.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `triblespace` 0.46.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `trident-lang` 0.2.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `trimwire` 0.6.0:
   category mismatch: consumer of `ahash`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `triplets` 0.27.1-alpha:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tropic01-driver` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `trouve` 1.1.0:
   category mismatch: consumer of `ahash`, `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `trouve-search` 4.11.0:
   category mismatch: consumer of `ahash`, `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `trustbeat` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `trustee` 0.19.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `trusty-embedderd` 0.3.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ts-procmacro` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tsafe-attest` 4.1.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tsafe-core` 4.1.0:
   category mismatch: consumer of `blake3`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `tsclient-rs` 0.1.3:
   category mismatch: consumer of `aes`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tsp-ltv` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tss-esapi` 7.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tss-esapi-sys` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tsslib` 0.2.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ttpkit-auth` 0.2.1:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `tun-tap-mac` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tuple-hash` 0.6.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `turbomcp` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turbomcp-core` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turboquant-rs` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turboshake` 0.7.1:
   HC1: same `keccak` backends as `sha3`.
- `turingmachine` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turingmachine-rs` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turn-server-sdk` 0.3.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `turnkey_client` 0.15.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turnkey_enclave_encrypt` 0.15.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `turnkey_proofs` 0.15.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tvmasm` 0.2.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `twc-rs` 4.0.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `twenty-first` 2.0.2:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `twig-sys-wasm32` 3.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `twine_protocol` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `twitch_oauth_token` 4.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `twofish` 0.8.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `txc` 0.5.2:
   category mismatch: consumer of `blake3`, `crc32fast`, `md-5`, `sha1`, `sha2`, `sha3` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `txtfp` 0.3.2:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `tycho-asm` 0.2.5:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `tydence` 0.2.0:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `tydence-cli` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `type-bridge-typedb-driver-b7` 3.8.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `type-bridge-typedb-driver-b8` 3.11.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `type-bridge-typedb-driver-b9` 3.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `typed-qb-procmacro` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `typedb-client` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `typedb-driver` 3.12.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `typeduck-codex-utils-rustls-provider` 0.17.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `tzukuj` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `u-siem-sonicwall` 0.0.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ubl-codec` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ubl-crypto` 0.1.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ubl-ledger` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ubl-types` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ubl-wasm` 0.1.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ucfp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `udb` 0.5.22:
   category mismatch: consumer of `crc32fast`, `openssl`, `openssl-sys`, `sha1`, `sha2` rather
   than a hash library of its own; the underlying crate carries the screening outcome.
- `udigest` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ufs-core` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ufs-forensic` 0.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ultrafast-mcp` 202506018.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `umac` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `underskrift` 0.1.4:
   category mismatch: consumer of `sha1`, `sha2`, `sha3` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `unegg` 0.2.1:
   category mismatch: consumer of `aes`, `crc32fast`, `sha1` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `unigateway-session` 2.15.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uniqopy` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `universal_notifications` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `universal-hash` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `unpdf` 0.19.0:
   category mismatch: consumer of `aes`, `crc32fast`, `md-5` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `unpdf-cli` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `unrar-rs` 0.10.7:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `uor-addr` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uor-addr-1` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uor-foundation` 0.5.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uor-prism` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uor-prism-crypto` 0.4.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `uor-prism-fhe` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uor-prism-numerics` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uor-prism-tensor` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `up-transport-zenoh` 0.9.1:
   category mismatch: consumer of `keccak` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `upgrade_verify` 0.1.4:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `upyun-sdk` 0.1.5:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ureld` 0.1.43:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uri-register` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `ursa` 0.3.7:
   category mismatch: consumer of `aes`, `openssl`, `sha2`, `sha3` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `usc-abi-encoding` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `use_css_procmacros` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `use-git-oid` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `use-mac` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uselesskey` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uselesskey-core-hash` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uselesskey-core-id` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uselesskey-core-seed` 0.7.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uselesskey-hmac` 0.10.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `userspace-rng` 1.0.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ustr` 1.1.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ustr-fxhash` 1.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `utxo_detective_cryptography` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uu_md5sum` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uu_sha1sum` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uu_sha256sum` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uu_sha512sum` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `uuidv5` 0.1.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `uxn-tal-defined` 0.4.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `v8` 152.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vach` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vacp2p_pmtree` 3.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vade-evan` 0.3.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vaid-mint` 0.7.0:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `valargroup-zakura-assets` 0.3457371.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `valknut-rs` 1.5.1:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `valve_pak` 0.1.0:
   category mismatch: consumer of `crc32fast` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vanity` 0.9.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `variant-ssl` 0.17.37:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vastlint-cli` 0.13.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vastlint-core` 0.13.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `veil-guard` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vellaveto-audit` 6.1.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `velocity_mcp` 3.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `velocity-mcp-core` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `velocity-mcp-edge` 3.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `verdet` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `veridict` 0.19.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `verifysign` 0.2.5:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `ves-stark-primitives` 0.3.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `veyron-wire` 0.2.7:
   category mismatch: consumer of `crc32fast`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `vgi-rpc` 0.25.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `viadkim` 0.2.0:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `viator` 0.1.19:
   category mismatch: consumer of `crc32fast`, `sha1`, `sha2`, `xxhash-rust` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `vipune` 0.12.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `visual-hashing` 0.1.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `visualsign` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vitaminc-aead` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vitaminc-async-traits` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vitaminc-encrypt` 0.4.0:
   category mismatch: consumer of `aws-lc-rs` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vitaminc-protected` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vld` 0.4.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vls-core` 0.14.0:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vmac` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vmdk` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `vmdk-cli` 0.8.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vmdk-core` 0.8.4:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `voa` 0.7.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vodozemac` 0.11.0:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `voided-core` 0.2.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vp9dec` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vr-identity` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vrd` 0.0.12:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `vrf` 0.2.5:
   category mismatch: consumer of `openssl` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vrf_fun` 0.12.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vrf-r255` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vrf-rfc9381` 0.0.7:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vsf` 0.9.3:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vsss-rs` 6.0.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vtcode-a2a` 0.163.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vtcode-webmcp` 0.163.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vuke` 0.9.0:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `vvva_crypto` 2.8.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `vyre-primitives` 0.7.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `vyre-reference` 0.7.2:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `w3f-bls` 0.2.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `w3f-pcs` 0.0.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `w3f-plonk-common` 0.0.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `w3f-ring-proof` 0.0.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wa-agent` 0.1.7:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `wa-rs` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wae-tools` 0.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters-1` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters-2` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters-3` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters-4` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters-5` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wagyu-zcash-parameters-6` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `walletkit-db` 0.23.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `warmplane` 0.30.0:
   category mismatch: consumer of `openssl`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `warpin-integrity` 0.2.6:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wascap` 0.16.2:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wasi-pg-client` 0.2.1:
   category mismatch: consumer of `md-5`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `wasm_web_crypto` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm-builder` 3.0.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm-embedded-rt-wasm3` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm-pvm` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm-pvm-cli` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm-smtp` 0.18.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wasm-smtp-cloudflare` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm-smtp-tokio` 0.18.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm3` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm3-provider` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm3-sys` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm32-unknown-unknown-openbsd-libc` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm32-unknown-unknown-openbsd-libc-wctypes-fix` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm3x` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm3x-sys` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wasm4pm-compat` 26.8.7:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `wasmtime-wasi-crypto` 12.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `waxpkg` 0.20.28:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wayfind` 1.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wd_tools` 0.15.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `weap` 0.1.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `weavatrix` 1.16.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `weavatrix-edit` 0.1.7:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `weavatrix-scan` 0.5.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `web-bot-auth` 0.7.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `web3-hash-utils` 1.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `webauthn_rp` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `webbuf_aescbc` 0.15.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `webbuf_aesgcm` 0.15.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `webex-headless-messenger` 0.1.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `webhook-verify` 0.1.0:
   category mismatch: consumer of `crc32fast`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `webpki` 0.22.4:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `webrisk_hash` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `webrtc` 0.20.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `webylib` 0.3.20:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wechat_work_crypto` 0.1.0:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `wedb_embed` 0.1.13:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wedpr_l_crypto_hash_blake2b` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wedpr_l_crypto_hash_ripemd160` 1.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wedpr_l_crypto_hash_sha3` 1.0.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wedpr_l_crypto_hash_sm3` 1.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `weftos-rvf-crypto` 0.3.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wgpu-sha1` 0.3.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `whasher` 0.1.8:
   category mismatch: consumer of `gxhash` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `whatsapp-rust` 0.7.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `whatsyoursign` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wheel-rs` 1.11.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `whirlpool` 0.11.0:
   HC1: portable.
- `whirlpool-asm` 0.6.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `whitaker-installer` 0.2.8:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `whitebit-sdk` 1.0.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wickra-exchange` 0.1.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wickra-exchange-core` 0.1.5:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wifi-densepose-bfld` 0.3.1:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `win-crypto-ng` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winarm-cpufeatures` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `windows-cmacros` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winter-air` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winter-crypto` 0.13.1:
   category mismatch: consumer of `blake3`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `winter-fri` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winter-math` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winter-prover` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winter-verifier` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winterfell` 0.13.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `winx-code-agent` 0.2.350:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wirefeather` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `wit-deps` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wit-deps-cli` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `with_tempdir_procmacro` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `with-watch` 0.1.5:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `witnet-bn` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wnaf` 0.14.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wol` 0.5.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolf-crypto` 0.1.0-alpha.15:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolf-crypto-sys` 0.1.0-alpha.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolfcrypt` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolfcrypt-conformance` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolfcrypt-sys` 0.2.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolfcrypt-wrapper` 1.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolfhsm` 0.2.1:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wolfhsm-sys` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wolfssl-wolfcrypt` 2.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wookiee` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `worktrunk` 0.78.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wormhole-vaas` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wrapped_mono` 0.4.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `write-hasher` 0.1.2:
   category mismatch: consumer of `crc32fast`, `sha1`, `sha2` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `ws-kit` 0.4.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wsc` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wsc-attestation` 0.11.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wsc-verify-core` 0.11.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `wsts` 14.0.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `wx-cli` 0.1.1:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `wyhash` 0.6.0:
   HC1 and HC2: scalar, 64-bit.
- `wyrand` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x-wing` 0.1.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `x0x` 0.45.0:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `x25519-dalek` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x25519-dalek-fiat` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x25519-dalek-ng` 1.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x25519-parser` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `x448` 0.6.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x509-cert` 0.3.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `x509-certificate` 0.25.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `x509-ocsp` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x509-parser` 0.18.1:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `x509-signature` 0.5.0:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `x509-tsp` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `x509-verify` 0.4.8:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `xaeroflux-macros` 0.8.1-m5:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `xai-grpc-client` 0.4.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xark-blake2s` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xark-keccak` 0.2.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xcbc-fdh` 0.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xchecker-receipt` 1.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xdvdfs-cli` 0.8.3:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xecrypt` 0.1.0:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `xfs-core` 0.1.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xgx_intern` 0.6.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xhash` 0.1.38:
   category mismatch: consumer of `xxhash-rust` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `xhs-electronic-print` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xinachtli` 0.2.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xkcp-rs` 0.0.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xkcp-sys` 0.0.5:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xlb` 0.8.37:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `xlsxwriter` 0.6.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xlsynth-driver` 0.70.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xml-sec` 0.1.16:
   category mismatch: consumer of `aes`, `md-5`, `sha1`, `sha2` rather than a hash library of
   its own; the underlying crate carries the screening outcome.
- `xoodoo` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xoodyak` 0.8.4:
   screening survivor: Xoodoo permutation with x86_64 SSE2 and aarch64 NEON implementations,
   streaming absorb and squeeze, MIT.
- `xorstring-procmacro` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xotp` 0.4.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xous-api-names` 0.9.71:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xpc-connection` 0.2.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xpc-connection-sys` 0.1.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xphone` 0.6.0:
   category mismatch: consumer of `aes`, `sha1` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `xposedornot` 1.0.1:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xs_curve25519-dalek` 4.1.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xs_ed25519-dalek` 2.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xs_x25519-dalek` 2.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xsalsa20poly1305` 0.9.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xtask` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xtask-kit` 0.1.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `xvc` 0.7.0:
   category mismatch: consumer of `blake3`, `sha2`, `sha3` rather than a hash library of its
   own; the underlying crate carries the screening outcome.
- `xx` 2.6.1:
   category mismatch: consumer of `blake3`, `md-5`, `sha1`, `sha2` rather than a hash library
   of its own; the underlying crate carries the screening outcome.
- `xxtea-nostd` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `xz4rust` 0.2.3:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `yadf` 1.4.1:
   category mismatch: consumer of `ahash`, `blake3`, `highway`, `twox-hash` rather than a hash
   library of its own; the underlying crate carries the screening outcome.
- `yauuid` 0.2.1:
   category mismatch: consumer of `md-5` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `yayo` 0.2.7:
   category mismatch: consumer of `ring` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `yescrypt` 0.1.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `yet-another-md5` 2.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `yogcrypt` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `yore-cli` 0.8.1:
   category mismatch: consumer of `ahash` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `ypass` 0.1.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `yubico_manager` 0.9.0:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `yubikey-hmac-otp` 0.10.2:
   category mismatch: consumer of `aes` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `yui-kh` 0.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `yunli` 0.1.20:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zagens-secrets` 0.8.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zakura-assets` 0.3481487.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zano-verify` 0.2.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zanolib` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zarchive` 0.2.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zawk` 0.5.25:
   category mismatch: consumer of `aes`, `blake3`, `sha2`, `xxhash-rust`, `openssl` rather than
   a hash library of its own; the underlying crate carries the screening outcome.
- `zbox` 0.9.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zca-rs` 0.1.0:
   category mismatch: consumer of `aes`, `md-5` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zcash_voting` 5.0.2:
   category mismatch: consumer of `blake2b_simd`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `zenith-core` 0.0.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zenith-layout` 0.0.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zenith-scene` 0.0.8:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zeno-fcis-crypto` 1.1.0:
   category mismatch: consumer of `libcrux-sha2`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `zenrust` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zenrust-auth` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zenrust-encryption` 0.0.3:
   category mismatch: consumer of `aes`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zenrust-hashing` 0.0.3:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zensim-regress` 0.3.1:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zentinel-modsec` 0.4.0:
   category mismatch: consumer of `md-5`, `sha1`, `sha2` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `zeph-context` 0.22.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zeph-core` 0.22.4:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zeph-plugins` 0.22.4:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zera-sc` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zernio` 0.0.970:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zeroclawlabs` 0.6.9:
   category mismatch: consumer of `ring`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zerocrypt` 0.1.7:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zerodds-foundation` 1.0.0-rc.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zerodds-security` 1.0.0-rc.6:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zerodds-security-crypto` 1.0.0-rc.6:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `zerodds-security-keyexchange` 1.0.0-rc.6:
   category mismatch: consumer of `aws-lc-rs`, `ring` rather than a hash library of its own;
   the underlying crate carries the screening outcome.
- `zeroize` 1.9.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zeroize_derive` 1.5.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zerokit_utils` 3.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zeropool-bn` 0.5.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zeus-bip32` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zewif` 0.0.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zf-zebrachain` 0.0.16:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zff` 3.0.0:
   category mismatch: consumer of `aes`, `blake3`, `sha2`, `sha3`, `twox-hash` rather than a
   hash library of its own; the underlying crate carries the screening outcome.
- `zfs-forensic-core` 0.1.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zic-rs-vendor-oracle-lab` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zisk-precomp-blake2` 1.2.0-alpha:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zisk-precomp-keccakf` 1.2.0-alpha:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zisk-precomp-sha256f` 1.2.0-alpha:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zisk-rom-setup` 1.2.0-alpha:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zkboo-bip32` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `zkboo-keccak` 0.0.0:
   no downloadable stable version (no stable release, or the download failed); name and
   description show no byte-string hash library.
- `zkevm-hashes` 0.3.0:
   category mismatch: consumer of `sha3` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zkryptium` 0.7.0:
   category mismatch: consumer of `sha2`, `sha3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zksync_merkle_tree` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zksync_mini_merkle_tree` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zkv` 0.3.0:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `znippy` 0.9.9:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `znippy-cli` 0.9.10:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `znippy-common` 0.9.15:
   category mismatch: consumer of `blake3`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `znippy-compress` 0.9.13:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `znippy-decompress` 0.9.11:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zorph-crypto` 0.1.0:
   category mismatch: consumer of `blake3` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zpaq_rs` 1.0.4:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zrail` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zrail-core` 0.0.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zrail-rust` 0.0.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zsign-rs` 0.1.2:
   category mismatch: consumer of `sha1`, `sha2` rather than a hash library of its own; the
   underlying crate carries the screening outcome.
- `zssh` 0.4.2:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zstandard` 0.1.8:
   category mismatch: not a byte-string hash library (manual review of description and source
   layout).
- `zsync-rs` 0.2.1:
   category mismatch: consumer of `sha1` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
- `zvec` 0.1.0:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zvvnmod-utn57` 0.3.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zwohash` 0.1.2:
   HC1: source scan found intrinsics for at most one of x86_64 and aarch64, no SIMD
   abstraction, and no accelerated dependency.
- `zync-core` 0.6.0:
   category mismatch: consumer of `sha2` rather than a hash library of its own; the underlying
   crate carries the screening outcome.
