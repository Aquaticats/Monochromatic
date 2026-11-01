# Fearless SIMD 1.0: a narrow AVX-512VBMI guard cannot prove its Ice Lake token

## Symptom

This is a **source-established integration constraint**, not a reproduced compiler error or a discovered upstream bug.
`package/forbidden-strings/core/src/dfa/sheng.rs` and `sheng2.rs` gate their fast paths on
AVX-512F, AVX-512BW, and AVX-512VBMI.
Replacing that guard with `fearless_simd::Level::new().as_avx512()` would require a larger CPU feature set
and therefore cannot preserve fast-path eligibility on every CPU admitted by the current guard.
The same token is required by `fearless_simd::kernel!` for AVX-512 intrinsics.
No specific machine has yet been shown to lose this path, and no slowdown has been measured.

## Root cause

Source examined at the `fearless_simd-v1.0.0` tag,
commit `dcbd824a897a098ec0124c088f971ac6d890d0cd`, on 2026-09-22.

1.  The consuming Sheng entry checks only the three features
    (`package/forbidden-strings/core/src/dfa/sheng.rs:226-240`):

    ```rust
    if is_x86_feature_detected!("avx512vbmi")
        && is_x86_feature_detected!("avx512bw")
        && is_x86_feature_detected!("avx512f")
    {
        unsafe { sheng_all_avx512(&tables, lines, out) };
    }
    ```

    The two-byte entry makes the same check in `src/dfa/sheng2.rs:251-266`.

2.  Fearless SIMD checks a larger feature set before producing `Level::Avx512`
    (`fearless_simd/src/lib.rs:286-330` in the pinned upstream tag):

    ```rust
    fn x86_detects_icelake_avx512() -> bool {
        std::arch::is_x86_feature_detected!("adx")
            && std::arch::is_x86_feature_detected!("aes")
            && std::arch::is_x86_feature_detected!("avx512bitalg")
            && std::arch::is_x86_feature_detected!("avx512bw")
            // Further requirements include VBMI, VBMI2, VAES and SHA.
    }
    ```

    `fearless_simd/src/generated/avx512.rs:35-51` assigns the same full set
    to `Avx512::assume_supported` and documents its safety contract.
    Constructing an `Avx512` token after checking only the consumer's three features
    would violate that contract.

3.  `fearless_simd::kernel!` maps its fixed `Avx512` token to that full feature set
    (`fearless_simd/src/kernel_macros.rs:152-202,340-383`):

    ```rust
    (Avx512, $item:item) => {
        #[target_feature(
            enable = "fxsr,adx,aes,avx512bitalg,avx512bw,avx512cd,avx512dq,avx512f,avx512ifma,avx512vbmi,avx512vbmi2,avx512vl,avx512vnni,avx512vpopcntdq,bmi1,bmi2,cmpxchg16b,fma,gfni,lzcnt,movbe,pclmulqdq,popcnt,rdrand,rdseed,sha,vaes,vpclmulqdq,xsave,xsavec,xsaveopt,xsaves"
        )]
        $item
    };
    ```

    The macro accepts the named tokens, not an arbitrary list of target features
    (`fearless_simd/src/kernel_macros.rs:4-20,68-99`).
    Its safe wrapper does not discharge independent pointer validity requirements
    (`fearless_simd/src/kernel_macros.rs:4-21,340-383`).

## Verification

- **Version and provenance:** pinned tag and commit named under Root cause.
  The published 1.0 crates.io archive checksum was not verified in this source-only assessment.
- **Source catalog, works:** the incumbent's narrow three-feature guard selects its existing
  `#[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]` function
  (`package/forbidden-strings/core/src/dfa/sheng.rs:281-302`).
  Upstream `fearless_simd/src/generated/simd_types.rs:7990-8001` provides
  `u8x64<S>` construction for any valid token,
  while `src/generated/avx512.rs:17687-17700` converts it to `__m512i` for any `S: Simd`.
  This is a source-supported load-only design, not a compiled consumer experiment.
- **Source catalog, does not preserve equivalent dispatch:** replacing the narrow gate
  with `Level::new().as_avx512()` cannot select `Avx512` on a CPU missing any feature
  listed at `fearless_simd/src/lib.rs:286-330`.
  Calling `Avx512::assume_supported()` after only the three-feature check is not a safe workaround.
- **Minimal compile-time harness, not executed:** with `fearless_simd = "=1.0.0"` in a
  disposable Cargo package, compile the following function under Rust 1.89 or later:

  ```rust
  // src/lib.rs
  use fearless_simd::Avx512;

  #[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
  fn narrow_guard() {
      let _proof = Avx512::assume_supported();
  }
  ```

  The source contract predicts that a safe call cannot prove the extra target features.
  The harness is a future check, **not** evidence of a particular compiler diagnostic.
- **Execution status:** no crate was installed, compiled, or run.
  An error code, actual affected CPU, and a changed machine-code or timing result are not claimed.
  A future disposable consumer test must exercise a feature-subset CPU before describing
  this as an observed regression.

## Verified workarounds

No runtime workaround was verified in this source-only evaluation.
The existing narrow guarded kernel remains the working source path.
Two **source-supported approaches, pending compilation and correctness tests**, are:

- Retain the old x86 kernel on feature-subset CPUs.
  Tradeoff: mixed implementation and dependency cost remain; the local target-feature call
  still needs a proof for the old narrow gate.
- Use the core crate's safe `u8x64::load_array_ref` with a valid lower-tier token,
  then its native conversion to `__m512i` inside the current narrowly gated function.
  Upstream `fearless_simd/src/generated/simd_trait.rs:9652-9654` and
  `src/generated/avx512.rs:17687-17700` expose these pieces;
  the consumer's table elements are `[u8; 64]` in
  `package/forbidden-strings/core/src/dfa/sheng.rs:99-117`.
  Tradeoff: only raw-pointer load obligations move into the dependency;
  the target-feature call remains and emitted loads may differ.

## What does not work

- Treat `Level::new().as_avx512()` as equivalent to the current three-feature dispatch:
  the source predicate requires more features.
- Fabricate `Avx512` from the narrow check:
  that would violate the token's documented safety preconditions.
- Assume `kernel!` makes pointer loads safe:
  its upstream documentation explicitly excludes non-feature safety requirements.
- Infer a throughput regression from the stricter predicate:
  the affected CPU set is conditional and benchmark performance was not measured.

## Upstream filing decision

`.out-of-scope/` was checked on 2026-09-22;
`cargo-workspace.md` is about a root workspace and does not exempt this topic.
`gh search issues` and `gh search prs` within `linebender/fearless_simd`
for `AVX512 VBMI token` and `Ice Lake dispatch` returned no matches on 2026-09-22.

1.  **Upstream fault? No.** The token describes an intentional Ice Lake feature tier,
    and this report does not establish a bug or a violated API promise.
2.  **Could upstream change the API? Yes in principle.** A separate narrow token would
    require upstream design and auditing; the feasibility has not been assessed.
3.  **Is this exact narrow tier supported? No evidence.**
    The documented token names Ice Lake instead.
4.  **Are contributions welcome? Yes in general.** `README.md:87-89` welcomes pull requests.
    `fearless_simd/SECURITY.md:17-20` bans entirely AI-generated security reports;
    this is not a security report and no such filing is proposed.
5.  **Would they likely change it? Unknown.**
    No maintainer request or decision on this exact tier was found in the searched terms.
6.  **Minimal fix prototyped? No.** The first and third constraints fail,
    so the auto-prototype prerequisite does not fire for an upstream issue.

Nothing to file or comment upstream on the present evidence.
The actionable consumer comparison is tracked in
[`../audit/tech-fearless-simd-adoption-vet-2026-09-22.md`](../audit/tech-fearless-simd-adoption-vet-2026-09-22.md).
