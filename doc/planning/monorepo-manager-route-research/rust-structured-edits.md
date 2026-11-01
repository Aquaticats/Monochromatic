# Comment-preserving JSONC, TOML, and XML edits in Rust

Merged into "Managed file editing" in `doc/planning/monorepo-manager-from-scratch-design.md` on 2026-09-17,
 where the user later dropped the untouched-bytes and CRLF rules defined here.
Later troubleshooting docs corrected these readings,
listed under "Research notes" in that design section:
`jsonc-parser` defaults are a loose set rather than JSON5,
the `1.0` rewrite came from this research's `serde_json` equality guard,
this research's all-`f64` number equality skips changes between integers above 2^53,
and the Biome JSON crates need three exact pins rather than seven.

## Status and scope

- Research date:
   2026-09-16 into 2026-09-17.
- Read-only design research for the all-Rust monorepo manager (`meow`)
   described in `doc/planning/monorepo-manager-from-scratch-design.md`,
   sections "Managed file edits",
   "Byte-identical output",
   and "Rewrite scope".
- Scope addition relayed by the coordinator:
   "XML comments must also be properly preserved."
- Nothing under `/var/home/user/Monochromatic` was modified,
   installed,
   or committed.
- Every option was designed until disqualifying problems surfaced (rule `YKZ`);
   documentation problems are recorded and never cull.

## Conventions

- **Verified** means a file and line read in this session,
   a page or API response fetched in this session with the quoted text found,
   or a probe listed in "Probes run" with its output.
- **Unverified** means inference,
   recall,
   or a claim not exercised by a probe.
- Path abbreviations:
  - `SE/` is
     `/tmp/claude-1000/-var-home-user-Monochromatic/e28ad59c-f3f5-46e9-8c8c-59a61c331610/scratchpad/structured-edits/`.
  - `PR/` is `SE/probe/`.
  - `TGT/` is `~/temp/agent/structured-edits-target-2026-09-16/`.
  - `REG/` is `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.
  - `TE/` is `package/toml-edit/core/src/`.
  - `JE/` is `package/jsonc-edit/core-ts/src/`.
  - `FE/` is `package/file-enforcer/core/src/`.
- Case identifiers such as `t03` or `j07` name cases in `PR/cases.ts`.

## Probes run

### Environment and deviations

- Toolchain:
   `rustc 1.100.0-nightly (0fc141305 2026-09-11)`,
   `cargo 1.100.0-nightly`,
   targets `x86_64-unknown-linux-gnu`,
   `x86_64-unknown-linux-musl`,
   and `aarch64-unknown-linux-musl` already installed (`rustup target list --installed`).
- Network worked this time:
   `curl --head https://index.crates.io/config.json` returned `HTTP/2 200` with a generic user agent.
  Crates were fetched by `cargo fetch` (Cargo's own `cargo/<version>` user agent),
   crate metadata by `SE/crates-search.ts` and `SE/crates-meta.ts`
   with user agent `structured-edit-research/0.1 (generic)`.
- Disclosure:
   `gh api` calls (repository metadata,
   IntelliJ source files,
   `tombi` releases)
   and `gh repo clone` used the machine's existing `gh` authentication,
   as the repository's `CLN` rule and the caller's clone instruction prescribe;
   no name or email address was sent in any request made by this research.
- Deviation:
   the first workspace build failed with "Disk quota exceeded (os error 122)" on the `/tmp` tmpfs (`usrquota`),
   so Cargo target directories moved to `TGT/` through `.cargo/config.toml` files in `PR/rust/`,
   `PR/rust-tombi/`,
   and `SE/size/`.
  Sources,
   cases,
   outputs,
   and scores stay under `SE/`.
- Clones (shallow,
   `gh repo clone ... -- --depth 1`):
   `~/temp/agent/tombi-2026-09-16` (commit `12c3b5b`);
   `~/temp/agent/taplo-2026-09-16` and `~/temp/agent/jsonc-parser-2026-09-16` already existed from an earlier agent
   (commits `08f343b` and `e6e3837 0.33.2`).

### Differential edit harness

- `node PR/cases.ts` writes the shared cases:
   21 TOML,
   16 JSONC,
   and 9 XML cases,
   each naming the incumbent test or file-enforcer operation it represents.
  Each case lists comments that must survive,
   untouched byte spans that must stay identical,
   comments of removed nodes that must be gone,
   an order of substrings,
   value checks,
   and whether the output must equal the input or the input is malformed.
- `node PR/incumbent.ts` runs the repository editors read-only:
   `module-toml-edit` with `applyCargoPlan` semantics (guard with `tomlHas`,
   write only when `tomlGetValue` differs),
   `module-jsonc-edit`,
   and file-enforcer `pipeline/xml.ts`.
- `PR/rust/` is a Cargo workspace of probes,
   each compiled under `#![forbid(unsafe_code)]`
   (the attribute covers probe code,
   not dependencies).
  `harness` applies every case twice and records both outputs.
- All candidates implement the same op semantics:
   `set` writes only when the value differs (numbers compared by value in wrappers),
   `ensureElement` appends only when absent,
   `removeElement` removes by value,
   `rename` and `delete` are no-ops when the path is absent,
   so a second identical run must be byte-identical.
- `node PR/compare.ts` scores outputs,
   reading values back with the repository parsers,
   into `PR/scores.md`.
- `node PR/run-all.ts` regenerates cases,
   reruns the incumbent,
   rebuilds with `cargo build --offline --jobs 4`,
   reruns every probe,
   and scores.
- Positive control for the harness (rule `QPC`):
   the same checks report failures for `toml_edit` used as its README shows
   and for the TypeScript JSONC editor,
   so a passing candidate is not an artifact of a blind check.
- Limits of the harness:
   value checks read XML attributes with `@lezer/xml`,
   which does not apply attribute-value normalization,
   so raw tab or newline characters written into attributes are caught only by manual inspection ("XML").

### Corpus round trips

- `TGT/probe/debug/toml-edit-wrapper corpus`,
   `taplo-probe corpus`,
   and `TGT/tombi/debug/tombi-probe` run every fixture in `package/test-fixture/toml-edit/src/`
   (91 valid,
   108 invalid,
   the counts `TE/fixtures.unit.test.ts:41` and `:44` assert):
   valid files must round-trip byte for byte,
   invalid files must fail.
- `jsonc-parser-asis corpus`,
   `biome-json-probe corpus`,
   and `json-five-probe corpus` run the `module-jsonc-edit.conformance` corpus
   (10 valid,
   11 invalid,
   from `package/jsonc-edit/core-ts.conformance/src/jsonc.conformance.unit.test.ts`).

### Sizes

- `SE/size/` holds one binary per candidate,
   each parsing and editing once,
   built with `lto = true`,
   `codegen-units = 1`,
   `strip = true`,
   `cargo build --release --offline --jobs 4 --target <triple>`.
- `aarch64-unknown-linux-musl` needed `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld`,
   because the host `cc` linked through `/usr/bin/ld.bfd`,
   which failed with "unrecognized option '--fix-cortex-a53-843419'".

## What "proper" means

### Operations file-enforcer performs today

- TOML (Verified):
  - `applyCargoPlan` (`FE/cargo/apply-plan.ts`) over `package/*/*/Cargo.toml` and `package/*/*/*/Cargo.toml`
     (`file-enforcer.config.ts:1772-1777`):
     guarded sets of scalars and inline tables holding arrays
     (edition,
     license,
     publish,
     lint keys,
     shared dependencies such as `clap = { version = "4", features = ["derive"] }`,
     repository,
     readme,
     homepage,
     profile keys),
     writing only when the value differs (`setIfDiffers`,
     `apply-plan.ts:46`),
     plus appending `[lints.clippy]` and `[workspace]` blocks with comment lines when absent (`apply-plan.ts:179-181`).
  - `getTomlProperty` reads `package.license` (`file-enforcer.config.ts:381`).
  - `editTomlKey` and `overwriteTomlKey` are exported but not called by the configuration
     (`rg` found only their definitions,
     examples,
     and `index.ts` exports).
- JSON and JSONC (Verified):
  - file-enforcer uses strict `JSON.parse` and `JSON.stringify(value, null, 2)` (`FE/pipeline/json.ts:118`,
     `:139`)
     for LSP4IJ embedded JSON and whole generated files.
  - `module-jsonc-edit` has no production consumer:
     `rg` for `module-jsonc-edit` outside `node_modules` and `dist` found only its sidecars,
     documentation,
     `pnpm-lock.yaml`,
     and `package/config/pnpr/config.yaml`.
  - `node SE/json-census.ts`:
     414 tracked `.json` files,
     179 rejected by strict `JSON.parse` (mostly `tsconfig.json` with trailing commas,
     plus `package/config/dprint/index.json` with comments);
     `git ls-files '*.jsonc' '*.json5'` printed nothing.
- XML (Verified):
   LSP4IJ settings through `replaceOrInsertXmlEntry` (`FE/pipeline/xml.ts:421`),
   called for the base and scoped servers in two files (`FE/jetbrains/lsp4ij-apply.ts:324`,
   `:339`,
   `:389`),
   and option reads through `getXmlOptionValue` (`FE/pipeline/xml.ts:362`).

### Incumbent guarantees

TOML,
 from `module-toml-edit` and file-enforcer tests (Verified):

- Unedited documents round-trip byte for byte (`TE/fixtures.unit.test.ts:75` over 91 fixtures),
   except that CRLF becomes LF (`package/toml-edit/core/README.md:39-46`).
- An equal-value set is byte-identical,
   keeping literal quotes and hex spelling
   (`TE/toml-set.unit.test.ts:31`,
   `:71`,
   `:84`;
   `FE/cargo/apply-plan.unit.test.ts:148`).
- Setting a value keeps its trailing inline comment and comment lines on other keys (`TE/toml-set.unit.test.ts:41`,
   `:56`).
- Creation lands inside the existing table (`TE/toml-set.unit.test.ts:149`;
   `apply-plan.unit.test.ts:53`),
   top-level keys land before table headers (`:579`),
   deep paths become dotted keys (`:514`),
   inline tables extend in place (`:527`),
   and a guard stops creation in crates that never declared the key (`apply-plan.unit.test.ts:121`).
- Deleting a key removes its line and same-line comment
   and keeps unrelated comment lines (`TE/toml-delete.unit.test.ts:39`,
   `:51`);
   deleting an array element keeps a comment after the array (`:196`).
- New nodes are formatted canonically:
   arrays inline as `[ 10, 30, ]` up to 4 elements and 80 columns,
   otherwise multi-line with 2-space indentation
   (`TE/types.ts:166-169`,
   `TE/toml-delete.unit.test.ts:172`),
   inline tables as `{ y = 1, }` (`TE/toml-set.unit.test.ts:476`).
- Re-applying the same edit is idempotent (`FE/pipeline/toml.property.unit.test.ts:146`).
- Comment attachment model:
   consecutive `#` lines directly above a key attach to it,
   a blank line breaks attachment (`TE/comments.unit.test.ts:29`,
   `:41`).

JSONC,
 from `module-jsonc-edit` (Verified):

- The write model is canonical,
   not splice:
   "This is not byte-identical splice;
   whitespace between tokens is normalized" (`package/jsonc-edit/core-ts/README.md:103-104`),
   "No source ranges are threaded through the parse" (`doc/decision/jsonc-edit-parser-foundation.md:64`).
- Raw scalar tokens survive when unedited (`JE/stringify.unit.test.ts:64`);
   records emit with 2-space indentation and trailing commas (`:100`);
   stacked `//` comments merge (`:112`).
- Setting a value keeps that value's comment (`JE/edit-set.unit.test.ts:140`);
   creating keys,
   appending at the length index,
   and deleting are supported (`:105`,
   `:59`,
   `:176`).
- Idempotency is "a second round-trip equals the first" (`JE/stringify.unit.test.ts:34`),
   but the comment-safety property only requires a fixpoint from the second emission:
   "First emission may normalize placement"
   (`package/jsonc-edit/core-ts.fuzz/src/comment-safety.property.unit.test.ts:44`).
- JSONC means VS Code semantics,
   JSON5 syntax and scalar roots rejected
   (`jsonc.conformance.unit.test.ts:2`,
   `:57`).

XML,
 from file-enforcer (Verified):

- Entries and options are found by decoded attribute values regardless of quote style,
   spacing,
   and attribute order
   (`FE/pipeline/xml.unit.test.ts:171`,
   `:198`).
- Replacement splices from the entry line's indentation so repeated runs do not accrete whitespace,
   and is idempotent (`FE/pipeline/xml.ts:443`,
   `xml.unit.test.ts:224`);
   insertion goes before the last `</map>` indentation (`:244`);
   no map throws (`xml.ts:465`,
   `xml.unit.test.ts:259`).
- Attribute text escapes `&`,
   `"`,
   `<`,
   `>`,
   newline,
   carriage return,
   and tab (`FE/pipeline/xml-coding.ts:177`,
   `xml.unit.test.ts:74`).
- No test asserts anything about XML comments
   (`rg --ignore-case 'comment|<!--'` over the XML and LSP4IJ tests printed nothing).
- User decision:
   a malformed managed XML file fails with a diagnostic.

### Definition

Proposed from the incumbent guarantees,
the user's requirements,
and the conflicts between incumbents that the probes exposed.
Items marked "open" need a user answer ("Open questions").

- D1 round trip:
   parse and serialize without edits is byte-identical for well-formed input.
  TOML and XML incumbents meet it;
   the JSONC incumbent does not by design (open).
- D2 equal values:
   an edit whose target already has the value changes no byte,
   comparing numbers by value (`1.0` equals `1`) and strings by decoded text.
- D3 edited node:
   only the value's bytes change;
   its leading comments,
   same-line trailing comment,
   key spelling,
   and position stay.
- D4 untouched nodes:
   byte-identical,
   including comments,
   blank lines,
   indentation,
   quote style,
   and line endings.
- D5 created nodes:
   placed after the last member of the target container and before comments or blank lines
   that belong to the next section;
   indentation,
   trailing-comma use,
   and line ending copied from siblings;
   TOML deep paths as dotted keys;
   guards decide whether a node may be created.
- D6 deleted nodes:
   removed with their same-line trailing comment and their directly attached leading comments (no blank line between);
   blank-line-separated comments stay;
   no comment is moved onto a different node (open,
   incumbents disagree).
- D7 arrays:
   appending or removing elements keeps the other elements' comments and a multi-line layout.
- D8 rename:
   only key text changes;
   comments and position stay.
- D9 idempotency:
   the first application reaches the fixpoint;
   a second identical run changes no byte.
- D10 strictness:
   malformed input fails with a diagnostic that names a position;
   every output re-parses with the same parser.
- D11 encoding:
   inserted text is encoded for its destination grammar (repository rules `SYB`,
   `STB`),
   including `&#9;` and `&#10;` for tab and newline in XML attributes.
- D12 XML comments:
   comments anywhere survive edits that do not replace the node containing them;
   whole-entry replacement drops comments inside that entry because the new block text replaces it (open).
- D13 formatting of new composite TOML values:
   incumbent style `[ "a", ]` and `{ k = v, }`,
   or crate style `["a"]` and `{ k = v }` (open).

### Where incumbents fall short

Verified by the harness (`PR/scores.md`) unless marked:

- `module-toml-edit` (15 of 21):
  - `t06`:
     a new top-level key lands below the comment that belonged to `[s]`,
     output `a = 1\n\n# about s\nb = 2\n[s]`.
  - `t13`,
     `t14`:
     appending to or removing from a multi-line array re-emits it inline and deletes element comments,
     output `arr = [ "a", "b", "c", ]  # tail`.
  - `t10`:
     deleting `foo` keeps `# about foo` as an orphan.
  - `t15`:
     no rename API.
  - `t21`:
     CRLF input comes back LF.
- `module-jsonc-edit` (7 of 16):
  - `j07`:
     deleting `a` also deletes `// section`,
     a comment separated from `a` by a blank line (data loss).
  - `j05`:
     the commented-out line `// "disabled",` above `"b"` becomes a trailing comment of `"b"`.
  - `j04`,
     `j12`:
     a comment after the closing brace moves above the opening brace.
  - `j01`,
     `j03`,
     `j11`:
     untouched whitespace and raw numbers are rewritten (`1.0` equal-value set rewrote the file).
  - `j16`:
     CRLF input comes back with mixed endings (`// crlf\r\n` next to LF lines) and no final newline.
  - `j10`:
     no rename API.
- file-enforcer XML (4 of 9):
  - `x07`,
     `x08`:
     an unknown entity and a missing close tag are spliced anyway,
     contrary to the user decision.
  - `x04` to `x06`:
     no attribute-level edit,
     so every managed change replaces a whole entry and drops comments inside it (`x03`).

## Probe results by candidate

### TOML

- `toml_edit` 0.25.15 used as its README shows (`doc["a"]["b"] = value(...)`,
   `Array::push`,
   `Array::remove`,
   `TableLike::remove`):
   12 of 21.
  - `t01`,
     `t03`,
     `t07`,
     `t16`,
     `t18`:
     index assignment replaces the value decor,
     deleting the same-line comment;
     on the file-enforcer Cargo plan itself (`t18`) `# clap trailing` is lost.
  - `t14`:
     removing `"b"` deletes `# first` and moves `# second` onto `"a"`,
     output `"a", # second` (a comment attached to the wrong element).
  - `t10`:
     deleting `foo` also deletes `# file header`,
     which a blank line separated from `foo`,
     because the whole leading trivia is the key's decor prefix.
  - `t15`:
     remove plus insert moves the key to the end and deletes `# above`.
  - `t21`:
     CRLF becomes LF.
  - Also Verified in source:
     `Table::insert` on an occupied key calls `entry.key_mut().fmt()`
     (`REG/toml_edit-0.25.15+spec-1.1.0/src/table.rs:429-443`),
     the reset that deleted a comment in the prior research,
     still present in 0.25.15.
- `toml_edit` 0.25.15 behind a repository wrapper (`PR/rust/toml-edit-wrapper/src/main.rs`,
   226 lines,
   plus `common.rs`,
   69 lines):
   21 of 21.
  - The wrapper replaces values in place copying the old decor,
     never calls `insert` on an occupied key,
     creates deep paths as dotted inline tables (`b.c = 42`),
     moves the closing whitespace of an inline table to the appended member (`{ x = 1, y = 2 }`),
     copies the last element's line indentation when appending to a multi-line array,
     hands a removed element's prefix to the next element,
     keeps the blank-line-separated part of a deleted key's prefix,
     renames by rebuilding the table with the old key decor,
     and restores CRLF when every source line ended in CRLF.
  - Wrinkles:
     deleting the first table leaves a leading blank line (`t11` output starts `\n# about bar`);
     `Item::get_mut` with a string inserts an `Item::None` placeholder for a missing key
     (`REG/toml_edit-0.25.15+spec-1.1.0/src/index.rs:52`),
     so presence must be checked first.
  - Corpus:
     88 of 91 valid fixtures round-trip;
     `date01-leading-zero` and `number-exponent01-with-sign` gain a final newline,
     and `keys-sample11-out-of-order` regroups out-of-order dotted keys
     (both limitations listed in the crate README,
     `REG/toml_edit-0.25.15+spec-1.1.0/README.md`,
     "Limitations").
    106 of 108 invalid fixtures fail;
     `invalid22-key-newline-value-in-inline-table` and `invalid23-...` parse.
- `taplo` 0.14.0 with value splices by DOM text range and `Rewrite::rename_keys`:
   6 of 21;
   every create,
   delete,
   and array op needs a repository editor.
  - `t20`:
     rejects `clap = { version = "3", features = [ "derive", ], }` with "expected value,
     trailing comma is not allowed".
  - Corpus:
     76 of 91 valid;
     fails the TOML 1.1 fixtures (`inline-table-sample01-1.1`,
     `string-sample01-1.1-basic-strings`,
     five `toml10-invalid-toml11-valid-*`)
     and several date and exponent fixtures;
     107 of 108 invalid rejected.
- `tombi-parser` at commit `12c3b5b` (path dependency on the clone):
   parser only,
   no edit probe.
  - Corpus:
     91 of 91 valid fixtures round-trip with no errors;
     `t20` parses with no errors and round-trips.
  - 78 of 108 invalid fixtures produce parser errors;
     duplicate keys,
     table redefinitions,
     escapes,
     and dates pass the parser because semantic checks live in other `tombi` crates.
- Cargo accepts the incumbent's TOML 1.1 output:
   `cargo metadata --offline --no-deps` on a manifest with `cfg-if = { version = "1", features = [ ], }`
   succeeded on nightly 1.100,
   `1.98.1`,
   and `stable` (`cargo 1.98.0`),
   in `SE/toml11-cargo/`.
  Other consumers of managed TOML (for example Mise) were not probed (Unverified).

### JSONC

- `jsonc-parser` 0.33.2 CST used as documented,
   with strict options (comments and trailing commas only):
   14 of 16.
  - `j11`:
     setting `a` to `1` where the file says `1.0` rewrote the token,
     because `serde_json` numbers `1.0` and `1` compare unequal.
  - `j07`:
     deleting `a` keeps `// about a` above the next member.
    Cause,
     Verified in source:
     `remove_comma_separated` removes only previous trivia up to the first newline
     (`~/temp/agent/jsonc-parser-2026-09-16/src/cst/mod.rs:3285-3291`).
  - Everything else passed,
     including trailing-comma style copied into new members (`j03`),
     appends after commented-out lines (`j05`,
     `j13`),
     rename by `set_raw_value` keeping `/* k */` and `/* v */` (`j10`),
     and CRLF preserved for inserted lines (`j16`).
  - Corpus:
     10 of 10 valid round-trip;
     9 of 11 invalid rejected,
     scalar roots `42` and `"bare string"` accepted.
  - Real file:
     appending `"a": true` to `package/config/dprint/index.json` placed it after the last property and
     before the trailing commented-out `/* "typescript": ... */` block
     (`diff` against the original showed only that insertion,
     plus a final newline the probe's `println!` added).
- `jsonc-parser` 0.33.2 behind a wrapper (`PR/rust/jsonc-parser-wrapper/src/main.rs`,
   133 lines including a corpus mode,
   plus the shared `PR/rust/jsonc-parser-asis/src/common.rs`,
   112 lines):
   16 of 16.
  - Adds numeric-aware equality,
     removes directly attached leading comments before `remove`,
     and rejects non-container roots.
  - Corpus through the wrapper entry point (`TGT/probe/debug/jsonc-parser-wrapper corpus`):
     "valid:
     10 of 10",
     "invalid:
     11 of 11 rejected".
  - Wrinkle:
     after deleting `a`,
     the blank line between `// section` and `// about b` is gone (`j07` output).
- Biome JSON crates from crates.io (`biome_json_parser` 0.5.7 and friends),
   value replacement by trimmed text range only:
   5 of 16;
   create,
   delete,
   rename,
   and array ops need a repository editor.
  - Build:
     caret requirements resolved `biome_rowan` and `biome_parser` to 0.5.8 and `biome_unicode_table` to 0.5.9;
     `biome_json_syntax` 0.5.7 then failed with "not all trait items implemented,
     missing:
     `is_trivia`",
     and after pinning `biome_parser` and `biome_rowan`,
     `biome_json_parser` 0.5.7 failed with
     "non-exhaustive patterns:
     `biome_unicode_table::Dispatch::DOL` not covered".
    It compiled only after `cargo update ... --precise 0.5.7` for `biome_parser`,
     `biome_rowan`,
     `biome_console`,
     `biome_diagnostics`,
     `biome_text_edit`,
     `biome_text_size`,
     and `biome_unicode_table`.
  - Corpus:
     10 of 10 valid round-trip;
     9 of 11 invalid rejected,
     scalar roots accepted.
- `json-five` 0.3.1 round-trip model (`json_five::rt`),
   replacing existing member values:
   4 of 16.
  - `j08`:
     output `/* strictness *` without the closing slash,
     which no parser accepts
     ("unterminated block comment (at offset 29)").
  - Corpus:
     9 of 10 valid round-trip (`{ /* block */ "a": 1 }` fails);
     6 of 11 invalid rejected,
     accepting `{ 'a': 1 }`,
     `{ a: 1 }`,
     `{ "a": 0x1F }`,
     and scalar roots.

### XML

- Splice editor shared by two indexers (`PR/rust/xml-quick/src/splice.rs`,
   105 lines):
   a port of `replaceOrInsertXmlEntry` plus `setOption`,
   which replaces only an option's `value` attribute text,
   escaping for the attribute's own quote character,
   and re-indexing after every op and on the final output.
- `quick-xml` 0.42.0 as a strict index (`PR/rust/xml-quick/src/main.rs`,
   139 lines):
   9 of 9.
  - Repository code had to add well-formedness checks:
     unclosed elements at end of input,
     exactly one root,
     text outside the root,
     unknown entity references in text (`Event::GeneralRef`),
     entity resolution in attributes through `normalized_value_with`,
     and a scan of `<!ENTITY>` declarations in the DOCTYPE.
  - Attribute byte ranges come from pointer differences between the borrowed value and the source string (safe code).
  - Diagnostics:
     `x07` "attribute value:
     at 4..8:
     unrecognized entity `nbsp`" (position relative to the attribute value),
     `x08` "ill-formed document:
     expected `</entry>`,
     but `</map>` was found at byte 756".
- `roxmltree` 0.21.1 as a strict index with `allow_dtd: true` (`PR/rust/xml-rox/src/main.rs`,
   54 lines):
   9 of 9.
  - Byte ranges come from `Node::range` and `Attribute::range_value`;
     well-formedness and DTD internal entities are the parser's job.
  - Diagnostics:
     `x07` "unknown entity reference 'nbsp' at 11:52",
     `x08` "expected 'entry' tag,
     not 'map' at 20:7".
- `xot` 0.31.2 parse,
   edit nodes,
   `to_string`:
   2 of 9 (only the malformed cases pass).
  - Keeps comments and whitespace text,
     but rewrites every element in the document:
     `" />"` becomes `"/>"`,
     `key='beta'` becomes `key="beta"`.
  - Writes a raw tab into `value="new &quot;cmd&quot; &amp; &lt;x>\ttab"` and raw newlines into `configurationContent`.
    XML 1.0 attribute-value normalization turns them into spaces on the next parse:
     "For a white space character (#x20,
     #xD,
     #xA,
     #x9),
     append a space character (#x20) to the normalized value"
     (fetched `https://www.w3.org/TR/xml/`),
     and `quick-xml` documents the same translation for `\n` and `\t`
     (`REG/quick-xml-0.42.0/src/events/attributes.rs:95-101`).
  - `x09`:
     "DTD is not supported".
- `xmltree` 0.12.0 (`Element::parse_all`,
   `write_with_config` without indentation):
   2 of 9.
  - Writes the whole document on one line (all whitespace text dropped),
     reorders attributes (`value` before `name`),
     and is not idempotent;
     which cases fail idempotency changed between two runs,
     so attribute order is not deterministic.
  - Writes a raw tab into the attribute.
- `xml` 1.4.0 (xml-rs) event reader copied to the event writer,
   comments and whitespace kept:
   2 of 9.
  - Adds `<?xml version="1.0" encoding="UTF-8"?>`,
     writes `<option ...></option>` for `<option ... />` with `normalize_empty_elements(false)`,
     normalizes quotes,
     writes a raw tab into the attribute,
     and emits newlines as `&#xA;`.
- Context,
   Verified:
   IntelliJ's JDOM builder ignores comments,
   "DTD,
   COMMENT and PROCESSING_INSTRUCTION are ignored"
   (`platform/util/src/com/intellij/openapi/util/SafeStAXStreamBuilder.kt:21`,
   fetched with `gh api`),
   and `JDOMUtil.load` calls that builder (`platform/util/src/com/intellij/openapi/util/JDOMUtil.java:328`,
   `:348`).
  That the settings store loads LSP4IJ files through this path and rewrites them without comments on save is Unverified.

### Static musl builds and sizes

All four leading crates are pure Rust and built statically without a C compiler (Verified):

- `file` reports `x86_64-unknown-linux-musl` binaries as "static-pie linked"
   and the `aarch64-unknown-linux-musl` binary as "ARM aarch64 ... statically linked".
- The x86_64 musl binaries ran natively on the fixtures;
   the aarch64 musl binaries ran under `qemu-aarch64-static`
   (`with-toml-edit` printed `edition = "2024"`,
   `with-roxmltree` printed `64`,
   `with-jsonc-parser` produced one `"a": true`).
- Dependency trees (`cargo tree --edges normal`):
   `toml_edit` pulls `indexmap`,
   `hashbrown`,
   `equivalent`,
   `toml_parser`,
   `toml_writer`,
   `toml_datetime`,
   `winnow`;
   `jsonc-parser` with `cst` pulls nothing;
   `quick-xml` and `roxmltree` pull `memchr`;
   `tombi-parser` plus `tombi-ast-syntax` pull 109 unique packages.
- `unsafe` in sources:
   `rg --count-matches '\bunsafe\b'` printed no match for `toml_edit` 0.25.15 or `jsonc-parser` 0.33.2 sources
   while printing matches for `toml_parser`,
   `quick-xml`,
   and `taplo`,
   so the search ran;
   `toml_parser` forbids unsafe unless its `unsafe` feature is on (`REG/toml_parser-1.1.3+spec-1.1.0/src/lib.rs:18`);
   `quick-xml` and `roxmltree` declare `#![forbid(unsafe_code)]` (`src/lib.rs:47` and `src/lib.rs:17`).

Stripped release sizes in bytes,
 and the increase over a baseline that reads a file and prints it (Verified by `ls -l` under `TGT/size/`):

- `x86_64-unknown-linux-musl`:
   baseline 300,920;
   `toml_edit` 537,608 (+236,688);
   `jsonc-parser` 435,456 (+134,536);
   `quick-xml` 343,800 (+42,880);
   `roxmltree` 377,992 (+77,072);
   all four (both XML parsers included) 779,320 (+478,400);
   `tombi-parser` 2,205,560 (+1,904,640).
- `x86_64-unknown-linux-gnu`:
   baseline 393,848;
   `toml_edit` 631,416 (+237,568);
   `jsonc-parser` 529,016 (+135,168);
   `quick-xml` 434,840 (+40,992);
   `roxmltree` 471,704 (+77,856);
   all four 869,016 (+475,168).
- `aarch64-unknown-linux-musl`:
   baseline 355,152;
   `toml_edit` 547,312 (+192,160);
   `jsonc-parser` 468,744 (+113,592);
   `quick-xml` 394,968 (+39,816);
   `roxmltree` 421,168 (+66,016);
   all four 759,208 (+404,056);
   `tombi-parser` 1,813,440 (+1,458,288).
- The size binaries exercise one call path each;
   a full wrapper would add code (Unverified how much).

## Maintenance data

Fetched 2026-09-16 by `SE/crates-meta.ts` from `https://crates.io/api/v1/crates/<name>`
and `gh api repos/<owner>/<repo>`
(results in `SE/crates-meta.json`);
"past year" counts non-yanked versions published since 2025-09-16.
Verified unless marked.

- `toml_edit` 0.25.15,
   published 2026-09-11,
   22 releases in the past year,
   198,995,334 recent downloads,
   MIT OR Apache-2.0;
   `toml-rs/toml` 1,074 stars,
   71 open issues,
   pushed 2026-09-15.
- `taplo` 0.14.0,
   2025-05-22,
   0 releases,
   716,818 recent downloads,
   MIT;
   `tamasfe/taplo` 2,391 stars,
   240 open issues.
- `tombi`:
   crates.io holds only placeholders (`tombi` 0.0.1 "Reserved package for tombi");
   the workspace version is `0.0.0-dev` with the comment "We use git tags for versioning"
   (`~/temp/agent/tombi-2026-09-16/Cargo.toml:24`);
   `tombi-toml/tombi` 1,121 stars,
   6 open issues,
   MIT,
   157 GitHub releases since 2025-09-16,
   latest `v1.5.5` on 2026-09-12.
- `oxc-toml` 0.14.6,
   7 releases,
   59,352 recent downloads,
   8 stars,
   described as "A TOML formatter library".
- `toml_dom` 0.6.0,
   7 releases,
   655 recent downloads,
   0 stars.
- `jsonc-parser` 0.33.2,
   2026-09-12,
   14 releases,
   3,078,299 recent downloads,
   MIT;
   `dprint/jsonc-parser` 64 stars,
   1 open issue.
- Biome JSON crates:
   `biome_json_parser` 0.5.7 published 2024-03-12,
   `biome_rowan` 0.5.8 on 2024-12-18,
   0 releases in the past year;
   `biomejs/biome` 25,853 stars,
   391 open issues,
   latest release `@biomejs/biome@2.5.14` on 2026-09-16;
   the repository's `crates/biome_json_parser/Cargo.toml` still says `version = "0.5.7"` and `publish = true`.
- `json-five` 0.3.1,
   2026-01-08,
   1 release,
   485,887 recent downloads,
   MIT,
   20 stars;
   open issue 7 "Provide better methods for reading and editing whitespace/comments".
- `edikt-jsonc` 0.4.0,
   5 releases,
   6,036 recent downloads,
   1 star,
   described as "lossless rowan+logos CST".
- `quick-xml` 0.42.0,
   2026-08-22,
   10 releases,
   109,742,400 recent downloads,
   MIT;
   `tafia/quick-xml` 1,567 stars,
   91 open issues.
- `roxmltree` 0.21.1,
   2025-10-12,
   1 release,
   21,871,024 recent downloads,
   MIT OR Apache-2.0;
   536 stars,
   8 open issues,
   pushed 2026-05-23.
- `xot` 0.31.2,
   2025-04-09,
   0 releases,
   75,888 recent downloads,
   43 stars,
   18 open issues.
- `xmltree` 0.12.0,
   2025-11-10,
   1 release,
   4,986,529 recent downloads,
   47 stars.
- `xml` 1.4.0 (the renamed `xml-rs`),
   2026-08-06,
   7 releases,
   3,902,736 recent downloads,
   118 stars.
- Screened XML trees:
   `uppsala` 0.10.1 (16 releases,
   171,212 recent downloads,
   22 stars),
   `xmloxide` 0.5.0 (13 releases,
   17,671,
   85 stars),
   `twig-doc` 3.4.0 ("losslessly round-trip ... XML documents",
   26 releases,
   1,975,
   1 star).
- Multi-format editors:
   `fig` 4.0.0 (19 releases,
   1,870 recent downloads,
   11 stars),
   `patchloom` 0.34.0 (44 releases,
   1,233,
   17 stars).

Repository Rust incumbents (Verified by `rg` over tracked `Cargo.toml` and `Cargo.lock`):

- No crate depends directly on `toml_edit`,
   `jsonc-parser`,
   `quick-xml`,
   `roxmltree`,
   `rowan`,
   or `taplo`.
- `toml` 1 is a direct dependency of `package/rust-module/rust-linter-core/Cargo.toml:37`;
   `rowan` 0.15.18 arrives through `ra_ap_syntax` in `package/linter/rust/Cargo.lock`;
   `toml_edit` 0.25.12 through `proc-macro-crate` in `package/music-player/desktop-app/Cargo.lock`,
   which also locks `quick-xml`,
   `roxmltree`,
   and `xml-rs` transitively.

## TOML options

### T1: `toml_edit` behind a repository wrapper

- Design:
   `DocumentMut` is the lossless tree;
   a repository module exposes only set,
   create,
   delete,
   rename,
   ensure-element,
   remove-element,
   and append-block,
   implemented with the decor-preserving calls the probe used;
   equality guard and CRLF restoration in the module;
   the probe case set and the `toml-test` fixtures become its regression tests;
   parse compares `doc.to_string()` with the source and reports files that `toml_edit` cannot round-trip.
- Pros:
  - 21 of 21 cases,
     idempotent,
     on a maintained crate "primarily tailored for cargo-edit needs" (crate README) with 22 releases
     and 199 million recent downloads.
  - TOML 1.1 input and output;
     new composite values default to TOML 1.0-valid `["a"]` and `{ k = v }`.
  - The passing wrapper is 295 probe lines (226 in `main.rs` including a corpus mode,
     69 in `common.rs`).
  - Pure Rust,
     static musl on both architectures,
     +236,688 bytes on x86_64.
- Cons:
  - The wrapper depends on where `toml_edit` stores trivia
     (key leaf decor prefix,
     value decor suffix,
     array element prefix,
     array trailing,
     table decor);
     an upstream change there silently changes comment behavior,
     so the regression corpus is mandatory.
  - Out-of-order dotted keys are regrouped and a missing final newline is added (upstream "Limitations"),
     so those files change beyond the edit.
  - Accepts two `toml-test` invalid fixtures.
  - `Item::get_mut` creates placeholders for missing keys.
- Disqualifying problems:
   none found.

### T2: `tombi` lossless parser plus a repository splice editor

- Design:
   `tombi-parser` and `tombi-ast-syntax` as git dependencies;
   a repository editor computes byte ranges for values,
   key-value lines,
   tables,
   and array elements from the syntax tree,
   formats inserted text itself,
   and validates with `tombi` document-tree crates or a repository checker.
- Pros:
  - 91 of 91 valid fixtures round-trip exactly,
     including the three `toml_edit` misses and TOML 1.1.
  - Splicing makes untouched bytes identical by construction.
- Cons:
  - No published crates;
     a git dependency on a repository with 157 releases in a year.
  - The parser alone lets 30 invalid fixtures through;
     validation needs more crates or repository code.
  - Every placement,
     formatting,
     and comment rule is repository code,
     unprobed.
  - +1,904,640 bytes on x86_64 musl with 109 packages,
     for the parser alone.
- Disqualifying problems:
   none found;
   the editor is unbuilt,
   so its behavior is Unverified.

### T3: Rust port of `module-toml-edit`

- Design:
   port the splice and canonical emitters (`TE/`,
   7,954 production lines measured with `wc --lines` over non-test files)
   onto a Rust lossless parser (`toml_parser` events or `tombi`),
   since `toml-eslint-parser` has no Rust counterpart.
- Pros:
  - Behavior already specified by the repository's tests and fuzz suites.
  - Keeps today's canonical formatting.
- Cons:
  - Ports verified defects:
     `t06` comment detachment,
     `t13` and `t14` array comment loss and layout collapse,
     orphaned comments,
     no rename,
     LF-only output.
  - Largest amount of ported code,
     plus a parser layer.
- Disqualifying problems:
   none,
   provided the port fixes the listed defects;
   unfixed,
   `t13` and `t14` violate comment preservation.

### T4: `toml_edit` as-is

- Design:
   call sites use index assignment,
   `insert`,
   `push`,
   and `remove` directly.
- Pros:
   no wrapper code.
- Cons and disqualifying problems:
  - Loses the same-line comment on the file-enforcer Cargo plan itself (`t18`),
     on scalar sets,
     and on dotted keys.
  - Moves a removed array element's neighbor comment onto the wrong element (`t14`).
  - Deletes blank-line-separated comments with a deleted key (`t10`).
  - Disqualified:
     comment loss on operations file-enforcer performs today.

### T5: `taplo` plus a repository editor

- Design:
   like T2 on `taplo`'s `rowan` tree and DOM text ranges.
- Pros:
   lossless tree for TOML 1.0;
   built-in key rename.
- Cons:
   0 releases in the past year,
   240 open issues;
   everything except value splice and rename is repository code.
- Disqualifying problems:
   rejects TOML 1.1 input,
   including the `{ ..., }` form the incumbent already writes into Cargo manifests and Cargo 1.98 accepts.

### Screened, not probed

- `toml_dom` 0.6.0:
   "TOML 1.1 / ATML library:
   read,
   modify,
   write";
   behavior Unverified;
   655 recent downloads,
   0 stars.
- `fig` 4.0.0 and `patchloom` 0.34.0:
   multi-format comment-preserving editors;
   behavior Unverified;
   under 2,000 recent downloads each.
- `oxc-toml` 0.14.6:
   described as a formatter library;
   whether it can edit without reformatting is Unverified.

## JSONC options

### J1: `jsonc-parser` CST behind a repository wrapper

- Design:
   `CstRootNode::parse` with strict `ParseOptions` (comments and trailing commas only);
   wrapper adds numeric-aware equality,
   attached-comment removal on delete,
   container-root check,
   and rename through `set_raw_value`;
   inserted members use the crate's style detection (indentation,
   trailing commas,
   newline kind).
- Pros:
  - 16 of 16 cases,
     idempotent,
     CRLF kept,
     conformance corpus 10 of 10 with 11 of 11 invalid rejected after the root check.
  - Zero dependencies,
     no `unsafe` in sources,
     +134,536 bytes on x86_64 musl.
  - Maintained by dprint (14 releases in the past year,
     3 million recent downloads);
     `jsonc-morph`,
     which `doc/decision/jsonc-edit-parser-foundation.md` called "the best byte-faithful round-trip in the field",
     wraps this crate:
     its `rs_lib/Cargo.toml` declares `jsonc-parser = { version = "0.33.2", features = ["cst", ...] }`
     (fetched with `gh api repos/dsherret/jsonc-morph/contents/rs_lib/Cargo.toml`).
- Cons:
  - Pre-1.0 API.
  - `ParseOptions::default()` enables loose names,
     missing commas,
     single quotes,
     hexadecimal,
     and unary plus
     (`~/temp/agent/jsonc-parser-2026-09-16/src/parse_to_ast.rs:68-80`),
     so strict options must never be forgotten.
  - The CST uses `Rc` and `RefCell` (`src/cst/mod.rs:36`,
     `:42`),
     so a tree cannot cross threads;
     per-file work in a child process is unaffected (Unverified for other designs).
  - Comments are positional trivia,
     not the attached comment-as-data model `module-jsonc-edit` offers.
  - The probe wrapper drops a blank line when removing attached comments.
- Disqualifying problems:
   none found.

### J2: `jsonc-parser` CST as-is

- Design:
   call sites use `set_value`,
   `append`,
   `remove`,
   and `replace_with` directly.
- Pros:
   14 of 16 with no wrapper.
- Cons:
   rewrites `1.0` on an equal-value set (`j11`);
   leaves a deleted member's leading comment behind (`j07`);
   default options accept JSON5.
- Disqualifying problems:
   none;
   `j11` violates D2 and `j07` depends on the D6 answer.

### J3: Biome JSON crates plus a repository editor

- Design:
   `biome_json_parser` lossless tree,
   edits through `biome_rowan` mutations or range splices.
- Pros:
   lossless round trip 10 of 10;
   comments kept as trivia;
   a large,
   active upstream.
- Cons:
  - crates.io releases stopped in 2024 while the source moved on under the same version number;
     current code needs a git dependency on the Biome monorepo.
  - The published crates do not compile under default resolution;
     seven crates had to be pinned to `=0.5.7`.
  - Every edit except value replacement is repository code.
- Disqualifying problems:
   none strictly;
   the build needs exact pins to a 2024 snapshot or a monorepo git dependency.

### J4: Rust port of `module-jsonc-edit`

- Design:
   port the hand-written parser and canonical emitter (`JE/`,
   3,873 production lines measured with `wc --lines`).
- Pros:
   comments as addressable data;
   behavior specified by mutation-tested unit tests.
- Cons:
  - Canonical output rewrites whitespace of every managed file on first write (by design).
  - Verified defects:
     deletes a blank-line-separated comment (`j07`),
     reattaches a commented-out line to a different element (`j05`),
     moves document-trailing comments,
     mixed line endings and no final newline on CRLF input,
     first emission not a fixpoint per its own property test,
     no rename.
- Disqualifying problems:
   disqualified if D1 and D4 hold (byte-identical untouched regions);
   otherwise `j07` data loss must be fixed in the port.

### J5: `json-five` round-trip model plus a repository editor

- Design:
   edit `json_five::rt` values and whitespace contexts.
- Pros:
   owned,
   editable model.
- Cons:
   JSON5 superset accepts inputs JSONC rejects;
   inserting members means building whitespace contexts by hand.
- Disqualifying problems:
   serializes `/* strictness */` as `/* strictness *`,
   producing unparseable output (`j08`).

### Screened, not probed

- `edikt-jsonc` 0.4.0:
   lossless `rowan` plus `logos` CST for JSONC,
   JSON5,
   and JSON;
   behavior Unverified;
   6,036 recent downloads,
   1 star.
- `tree-sitter-json` (not fetched):
   C sources would need a musl C toolchain per target,
   and an edit layer would be repository code (both Unverified).

## XML options

### X1: `roxmltree` strict parse plus a repository splice editor

- Design:
   parse with `allow_dtd: true`;
   index keyed entries,
   `option` elements,
   and the last `</map>` from `range` and `range_value`;
   apply byte splices;
   re-parse after every op and fail on any error;
   attribute-level `setOption` for owned options,
   whole-entry insert or replace otherwise.
- Pros:
  - 9 of 9,
     idempotent,
     all comments and bytes outside the edit preserved.
  - Well-formedness,
     entity,
     and DTD handling belong to the parser,
     with line and column diagnostics.
  - `#![forbid(unsafe_code)]`,
     only `memchr`,
     +77,072 bytes on x86_64 musl.
- Cons:
  - One release in the past year.
  - `range_value` is wrong for qualified names over `u16::MAX` bytes or more than `u8::MAX` spaces around `=`
     (`REG/roxmltree-0.21.1/src/lib.rs:620-621`).
  - Decoding an option value for the equality check needs the tree or a second parse.
- Disqualifying problems:
   none found.

### X2: `quick-xml` event scan plus the same splice editor

- Design:
   as X1,
   with a repository well-formedness pass over `quick-xml` events.
- Pros:
  - 9 of 9,
     idempotent.
  - Most adopted and actively released (10 releases),
     smallest (+42,880 bytes),
     `#![forbid(unsafe_code)]`.
- Cons:
  - Unclosed elements at end of input,
     root count,
     and DOCTYPE entity declarations are repository checks;
     the probe's `<!ENTITY>` scan is naive.
  - Attribute error positions are relative to the attribute value.
  - Attribute API changed between 0.41 and 0.42 (`unescape_value_with` deprecated for `normalized_value_with`,
     `REG/quick-xml-0.42.0/src/events/attributes.rs:285-287`).
  - Byte offsets of attribute values rely on borrowed values from `Reader::from_str`.
- Disqualifying problems:
   none found.

### X3: Port of `pipeline/xml.ts` with error recovery

- Design:
   keep `@lezer/xml`-style recovery through an error-tolerant Rust parser.
- Pros:
   byte parity with today on malformed files.
- Cons:
   no Rust `@lezer/xml`;
   tree-sitter needs C sources (Unverified).
- Disqualifying problems:
   contradicts the user decision that malformed XML fails;
   the strict port is X1 or X2.

### X4: `xot` tree

- Pros:
   keeps comments and whitespace text;
   mutable tree API.
- Cons:
   rewrites quoting and empty-element spelling across the file;
   0 releases in the past year.
- Disqualifying problems:
   raw tab and newline in attribute values,
   which XML attribute-value normalization turns into spaces;
   rejects DOCTYPE.

### X5: `xml` (xml-rs) event copy

- Pros:
   keeps comments and whitespace;
   streaming.
- Cons:
   adds an XML declaration,
   rewrites empty elements and quotes.
- Disqualifying problems:
   raw tab in attribute values;
   whole-document rewrite.

### X6: `xmltree`

- Disqualifying problems:
   drops all whitespace text,
   reorders attributes nondeterministically,
   not idempotent,
   raw tab in attribute values.

### Screened, not probed

- `twig-doc` 3.4.0 claims lossless XML round trips (Unverified),
   1 star.
- `uppsala` 0.10.1 is a DOM with XPath and XSD validation;
   formatting fidelity Unverified.
- `xmloxide` 0.5.0 reimplements libxml2 in Rust;
   formatting fidelity Unverified.

## Rankings

Separate rankings,
 because each format is an independent choice (rule `QSP`).

### TOML

T1 > T2 > T3 > `toml_dom` > `fig` > `oxc-toml` > T4 > T5.

- T1 over T2:
   T1 passed every edit case on a published,
   heavily used crate with a small wrapper;
   T2's editor does not exist yet and its parser costs 1.9 MB and a git dependency,
   while its advantage is three fixtures T1 cannot round-trip.
- T2 over T3:
   T2 starts from a verified lossless TOML 1.1 tree and new splice rules;
   T3 ports verified comment-loss defects and still needs a Rust parser layer.
- T3 over `toml_dom`:
   T3's behavior is measured and its defects enumerated;
   `toml_dom` is unprobed with 655 recent downloads.
- `toml_dom` over `fig`:
   `toml_dom` is a TOML-specific read,
   modify,
   write library;
   `fig` spreads one editor over several formats with similar adoption;
   both Unverified.
- `fig` over `oxc-toml`:
   `fig` claims comment-preserving edits;
   `oxc-toml` describes itself as a formatter.
- `oxc-toml` over T4:
   an unprobed option may still work;
   T4 is verified to lose comments on the Cargo plan.
- T4 over T5:
   T4 parses TOML 1.1 and is maintained;
   T5 rejects valid TOML 1.1 manifests.

### JSONC

J1 > J2 > J3 > J4 > `edikt-jsonc` > J5.

- J1 over J2:
   the wrapper fixes both verified defects (`j11`,
   `j07`) in 111 lines of edit logic.
- J2 over J3:
   J2 has a complete edit API passing 14 of 16;
   J3 offers a tree whose published crates need exact pins and an editor still to write.
- J3 over J4:
   J3's tree is lossless;
   J4 rewrites every file and has verified comment data loss.
- J4 over `edikt-jsonc`:
   J4 is repository-owned and measured;
   `edikt-jsonc` is unprobed with one star.
- `edikt-jsonc` over J5:
   J5 is verified to corrupt block comments.

### XML

X1 > X2 > X3 > `twig-doc` > `uppsala` > `xmloxide` > X4 > X5 > X6.

- X1 over X2:
   both pass 9 of 9;
   X1 gets the user-required malformed-file diagnostics from the parser with line and column,
   while X2 needs repository well-formedness code that the probe had to write,
   and the 34 KB size difference does not outweigh that.
- X2 over X3:
   X2 satisfies the malformed-file decision;
   X3 contradicts it.
- X3 over `twig-doc`:
   X3's disqualification is a requirement mismatch that turns into X1 or X2 when strict;
   `twig-doc`'s lossless claim is unprobed with one star.
- `twig-doc` over `uppsala`:
   `twig-doc` claims byte-lossless round trips;
   `uppsala` is a DOM with no stated formatting fidelity.
- `uppsala` over `xmloxide`:
   `uppsala` has 171,212 recent downloads against 17,671;
   both Unverified.
- `xmloxide` over X4:
   unprobed versus verified attribute corruption.
- X4 over X5:
   X4 keeps empty-element and declaration layout closer to the source;
   X5 adds a declaration and expands empty elements.
- X5 over X6:
   X5 keeps whitespace and is deterministic;
   X6 flattens the file and reorders attributes.

### Shared shape across formats

- Recommended from the probes (not a decision):
   one repository edit layer per format with the same op vocabulary,
   equality guards,
   post-edit re-parse,
   and the probe case sets as regression tests,
   because every passing candidate needed exactly those pieces.

## Documentation problems recorded

- `toml_edit`:
   the `Table::insert` doc comment says only "Inserts a key-value pair into the map."
   (`REG/toml_edit-0.25.15+spec-1.1.0/src/table.rs:428`)
   while the occupied branch resets key formatting;
   whether docs mention the `Item::None` placeholder from `get_mut` was not checked.
- `jsonc-parser`:
   the crate description is "JSONC parser."
   while default options accept JSON5-style syntax.
- Biome:
   repository `Cargo.toml` files still claim version 0.5.7 with `publish = true` although crates.io 0.5.7 dates
   from 2024-03-12.
- `json-five`:
   the README claims "Supports round-trip use cases with preservation/editing of whitespace and comments"
   while block comments lose their closing slash.
- `quick-xml`:
   attribute error positions are relative to the attribute value.

## Open questions

- JSONC write model:
   splice that keeps untouched bytes (J1,
   recommended) or canonical formatting as `module-jsonc-edit` does (J4)?
  Splice keeps diffs to the edited members in files other repositories own;
   canonical keeps one house style.
- Deleted nodes:
   remove directly attached leading comments with the node (`module-jsonc-edit`,
   `toml_edit`,
   recommended),
   or keep them as `module-toml-edit` and `jsonc-parser` do?
- New TOML arrays and inline tables:
   incumbent style `[ "derive", ]` and `{ version = "4", features = [ "derive", ], }`
   (TOML 1.1 trailing comma in inline tables),
   or `toml_edit` style `["derive"]` and `{ version = "4", features = ["derive"] }` (TOML 1.0-valid,
   recommended)?
- XML entries:
   edit owned options at attribute level and keep comments inside entries (recommended),
   or keep whole-entry replacement,
   which drops comments inside a managed entry?
- TOML files `toml_edit` cannot round-trip (out-of-order dotted keys,
   missing final newline):
   fail with a diagnostic naming the file,
   or accept the one-time regrouping?
- Line endings:
   preserve CRLF per file (both passing wrappers do) or normalize to LF as `module-toml-edit` does?
- Which JSONC files will `meow` manage?
   file-enforcer edits none today,
   and 179 tracked `.json` files already need a JSONC parser;
   the answer sets how much of J1's API is needed.
- Follow-up outside this read-only task:
   the external-tool quirks found here (`toml_edit` decor lumping,
   Biome crate resolution,
   `json-five` block comments,
   `xot` attribute whitespace)
   qualify for `doc/troubleshooting/` entries under the repository's `troubleshooting-doc` skill.
