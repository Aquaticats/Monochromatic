# Rust resources

## Knowledge

- [The Rust Programming Language: Experimental Edition](https://rust-book.cs.brown.edu/)
  Primary reading sequence for every companion lesson.
  Use for:
   lesson order,
   terminology,
   quizzes,
   and the research-informed ownership explanations.
- [Experimental edition source](https://github.com/cognitive-engineering-lab/rust-book)
  Source repository for the rendered book and its interactive extensions.
  Use for:
   checking exact page content,
   source changes,
   and reported issues.
- [The Rust Reference](https://doc.rust-lang.org/reference/)
  Primary language reference,
   which assumes prior Rust familiarity rather than teaching the language sequentially.
  Use for:
   resolving precise questions about syntax,
   semantics,
   types,
   memory,
   and unsafety.
- [Rust standard library documentation](https://doc.rust-lang.org/std/)
  API documentation for primitives,
   the prelude,
   macros,
   and `std` modules,
   with links to implementation source.
  Use for:
   distinguishing language behavior from library behavior and understanding APIs used by the monorepo.
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
  Official guide and reference for Cargo packages,
   manifests,
   workspaces,
   commands,
   and dependency resolution.
  Use for:
   connecting the book's Cargo model to each Rust package in this monorepo.
- [Rust compiler error index](https://doc.rust-lang.org/error_codes/error-index.html)
  Official index of error-code explanations emitted by `rustc`.
  Use for:
   following compiler diagnostics to their language rule and a corrected example.
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
  Official advanced companion for unsafe Rust,
   data layout,
   aliasing,
   concurrency,
   and FFI.
  Use only after the relevant book foundations;
   it is incomplete and directs disagreements to The Rust Reference.
- [Monorepo Rust policy](../../../AGENTS.md)
  Repository-specific Rust documentation,
   lint,
   build,
   and verification requirements.
  Use for:
   separating general Rust rules from local engineering decisions.
- [Monorepo Rust modules](../../rust-module/)
  Reusable Rust libraries,
   benchmarks,
   and fuzz targets maintained in this repository.
  Use for:
   concrete examples once the book has introduced each required concept.
- [Monorepo Rust linter](../../linter/monochromatic-lint/)
  Rust-based linting infrastructure and its tests.
  Use for:
   examples involving traits,
   syntax trees,
   diagnostics,
   and tooling.
- [Monorepo Rust applications](../../desktop-app/)
  Desktop applications containing first-party Rust crates.
  Use for:
   examples involving application boundaries,
   platform APIs,
   and UI integration.
- [Monorepo Rust music player](../../music-player/)
  Rust audio core,
   benchmarks,
   desktop integration,
   and Android bridge.
  Use for:
   examples involving concurrency,
   performance,
   FFI,
   and cross-platform design.

## Wisdom (communities)

- [Rust users forum](https://users.rust-lang.org/)
  Official community forum for learning and practical Rust questions.
  Use for:
   testing an explanation against experienced practitioners or seeking design feedback with a minimal example.
- [Rust Zulip](https://forge.rust-lang.org/platforms/zulip.html)
  Rust project chat used by compiler,
   language,
   library,
   and tooling teams.
  Use for:
   focused questions about active project work after identifying the relevant stream and reading its guidance.

## Gaps

- A concept-to-package source map will be built incrementally as the book introduces each concept.
- The learner's Rust baseline has not yet been demonstrated;
   only a TypeScript background is established.
