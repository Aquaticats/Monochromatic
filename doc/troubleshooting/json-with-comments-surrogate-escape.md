# json-with-comments v0.1.5: escaped UTF-16 surrogates reach an unchecked Rust character

Status:
 **source-proven safety failure with a checked prototype verified in an isolated container**.
 The original unsafe path was never run on surrogate input.
 The prototype is not an upstream release and still rejects unpaired surrogates,
 which this repository's JSONC editor must retain.
 No upstream issue or patch was sent.

## Symptom

Passing source text such as `r#""\uD800""#` to `json_with_comments::from_str::<String>` reaches
 `char::from_u32_unchecked(0xD800)`.
 The same applies to an isolated low surrogate and to the high half of a valid pair such as `\uD83D\uDE00`.
 A surrogate is not a valid Rust `char`,
 so this is undefined behavior under [Rust's documented safety precondition][rust-char],
 not a predictable parsing error.
 There is no observed runtime error string to quote because the unsafe original was not executed on these inputs.
 A JSON source with two backslashes before `u`,
 such as `r#""\\uD800""#`,
 is a different case:
 it escapes a literal backslash rather than constructing a surrogate.

## Root cause

The inspected repository is `hayas1/json-with-comments` at tag `v0.1.5`,
 commit `6584e3495fecca3a1d73a14b8f0f11794faf5e0e`.
 Its `Cargo.toml:1-9` identifies package `json-with-comments` version 0.1.5.
 That tag is not a verified crates.io publication:
 exact-name crates.io API lookup returned no crate,
 and the README instead documents a Git dependency.
 The source call chain is:

1. Public `from_str` delegates to a string tokenizer (`src/de.rs:61-65`):

   ```rust
   // src/de.rs:61-65
   pub fn from_str<'de, D>(s: &'de str) -> crate::Result<D>
   where
       D: de::Deserialize<'de>,
   {
       from_tokenizer(StrTokenizer::new(s))
   }
   ```

2. `StrTokenizer` delegates escape handling to the common token parser
   (`src/de/token/str.rs:43-46`),
   which dispatches `\u` to `parse_unicode` (`src/de/token.rs:151-160`):

   ```rust
   // src/de/token/str.rs:43-46
   fn parse_escape_sequence(&mut self, buff: &mut Vec<u8>) -> crate::Result<()> {
       self.unescaped = true;
       self.delegate.parse_escape_sequence_super(buff)
   }
   ```

   ```rust
   // src/de/token.rs:157-160
   (_, b't') => Ok(buff.push(b'\t')),
   (_, b'u') => Ok(self.parse_unicode(buff)?),
   (pos, found) => Err(SyntaxError::InvalidEscapeSequence { pos, found })?,
   ```

3. `parse_unicode` checks only that four bytes are hexadecimal,
   then treats the resulting 16-bit code unit as a Rust Unicode scalar
   (`src/de/token.rs:166-178`):

   ```rust
   // src/de/token.rs:166-178
   fn parse_unicode(&mut self, buff: &mut Vec<u8>) -> crate::Result<()> {
       let mut hex: u32 = 0;
       for i in 0..4 {
           match self.eat()?.ok_or(SyntaxError::EofWhileParsingEscapeSequence)? {
               (_, c @ b'0'..=b'9') => hex += ((c - b'0') as u32) << (4 * (3 - i)),
               (_, c @ b'a'..=b'f') => hex += ((c - b'a' + 10) as u32) << (4 * (3 - i)),
               (_, c @ b'A'..=b'F') => hex += ((c - b'A' + 10) as u32) << (4 * (3 - i)),
               (pos, found) => return Err(SyntaxError::InvalidUnicodeEscape { pos, found })?,
           }
       }
       let ch = unsafe { char::from_u32_unchecked(hex) }; // TODO maybe safe
       Ok(buff.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()))
   }
   ```

   Four hexadecimal digits can encode `0xD800..=0xDFFF`.
   Rust excludes that interval from valid `char` values.
   The source does not combine a high and low half before the unchecked conversion.

4. The apparently raw public route does **not** avoid this call:
   `from_str_raw` calls `from_raw` (`src/de.rs:100-106`),
   `from_raw` constructs `RawTokenizer` (`src/de.rs:252-258`),
   and `RawTokenizer::parse_string_content` calls the shared `parse_string_content_super`
   before returning original bytes (`src/de/token/raw.rs:23-28`):

   ```rust
   // src/de/token/raw.rs:23-28
   fn parse_string_content(&mut self) -> crate::Result<ParsedString<'de>> {
       let offset = self.delegate.current;
       let _ = self.parse_string_content_super()?;
       let raw = &self.delegate.slice[offset..self.delegate.current];
       Ok(ParsedString::Borrowed(std::str::from_utf8(raw)?))
   }
   ```

## Verification

The source checkout at `~/temp/agent/json-with-comments-2026-09-24` was pinned to the tag's commit.
 The disposable prototype lives under `~/temp/agent/upstream-prototype.9JtuZCZL/json-with-comments`.
 Its origin and commit were checked before editing.
 The attached [patch](json-with-comments-surrogate-escape.patch) changes only `src/de/token.rs`
 and adds `tests/surrogate_safety.rs`.
 It never invokes the original unchecked constructor on invalid input.

The focused `mise run test:json-with-comments:prototype` and full
 `mise run test:json-with-comments:full-prototype` tasks ran the patched clone inside
 `podman run --memory=2g --cpus=2 --rm` with only that disposable clone mounted,
 using `docker.io/library/rust:1.97-bookworm` at local image ID
 `a0635962c16d5f26400703edd4317175cf9531f3285d632613f9da227f9c71d1`.
 The final full run passed 86 unit tests,
 three added surrogate tests,
 and 64 existing integration tests.
 The `cargo test --all-targets` process exited 0.
 The test output also contained an unrelated compiler warning at
 `src/value/de/deserializer.rs:38:23` about `mismatched_lifetime_syntaxes`.

- Accepted controls in the **patched** clone:
   `"\u0041"` decodes to `A`;
   `"\uD83D\uDE00"` decodes to `😀`;
   `"\\uD800"` decodes to literal `\uD800`;
   the raw route retains authored bytes for a valid pair.
- Rejected controls in the **patched** clone:
   isolated `"\uD800"` and `"\uDC00"`;
   `"\uD800\u0041"` with the wrong low half;
   missing or wrong second escape prefix,
   invalid hexadecimal digits,
   and a truncated Unicode escape.
   Both `from_str` and `from_str_raw` returned errors for these cases.

The pre-patch finding is a source-level proof of a violated unsafe precondition,
 **not** a runtime reproduction of the original undefined behavior.
 The invalid controls were deliberately not run on the unpatched tag.
 A reader can verify the patch in a **fresh disposable** checkout at the pinned tag from this repository root.
 The command sequence does not run the original unsafe code on the invalid controls:

```bash
# doc/troubleshooting/json-with-comments-surrogate-escape.md
mkdir --parents "$HOME/temp/agent"
chmod 700 "$HOME/temp/agent"
gh repo clone hayas1/json-with-comments "$HOME/temp/agent/json-with-comments-pinned" -- --depth 1
git -C "$HOME/temp/agent/json-with-comments-pinned" fetch --depth 1 origin tag v0.1.5
git -C "$HOME/temp/agent/json-with-comments-pinned" checkout --detach v0.1.5
git -C "$HOME/temp/agent/json-with-comments-pinned" apply "$PWD/doc/troubleshooting/json-with-comments-surrogate-escape.patch"
podman run --memory=2g --cpus=2 --rm --volume "$HOME/temp/agent/json-with-comments-pinned:/workspace:Z" --workdir /workspace docker.io/library/rust:1.97-bookworm cargo test --all-targets
```

The checked patch's integration test file contains the complete inputs and assertions.

## Verified workarounds

- The disposable [patch](json-with-comments-surrogate-escape.patch) uses checked `char::from_u32`,
   combines valid UTF-16 pairs,
   and returns a typed error for isolated halves.
   The test runs verify that safety behavior on the listed inputs.
   Its tradeoff is deliberate rejection of otherwise syntactically admitted unpaired JSON escapes;
   it therefore is **not** a product foundation for this port as-is.

## What does not work

- Selecting `from_str_raw` as a workaround:
   `RawTokenizer` still invokes the common string decoder before returning the original slice
   (`src/de/token/raw.rs:23-28`).
- Replacing the unsafe constructor with `char::from_u32` and rejecting every surrogate:
   that avoids this undefined behavior but also rejects valid escaped pairs.
   The verified patch joins pairs before checking the scalar.
- Running the unpatched parser on a surrogate to obtain an error message:
   the result would be undefined behavior,
   so that is not a valid diagnostic probe.

## Upstream filing decision

No matching exemption exists under this repository's `.out-of-scope/` entries.
 Searches of upstream open and closed issues and PRs for `surrogate` and `unsafe unicode` found no duplicate;
 the upstream issue list contained only an unrelated comment-value request ([issue 35][issue-35]).
 The upstream release list names `v0.1.5` as the latest published GitHub release.
 No external message was sent.

1. **Upstream fault:** yes.
   The source invokes `char::from_u32_unchecked` with a possible surrogate without its required safety proof.
2. **Fixability:** yes.
   Checked construction plus valid pair decoding was prototyped within the parser's current design.
3. **Supported use case:** yes for parsing JSONC strings and valid escaped pairs,
   per the README's JSONC parser examples and `from_str` escaped-string documentation (`src/de.rs:41-59`).
   The README does not promise retaining unpaired surrogates as Rust strings.
4. **Contribution welcome:** no explicit restriction or AI-report ban was found in the inspected README
   and `.github/` workflows;
   the repository has merged outside pull requests,
   but there is no `CONTRIBUTING.md` or issue template in the checked clone.
5. **Maintainer disposition:** no source policy or searched issue/PR explicitly declines such a safety fix.
   This is not a prediction of response time or acceptance.
6. **Prototype:** yes.
   The linked patch was checked against the pinned tag in the bounded container;
   its new and existing tests passed.

These satisfy the technical filing gates,
 but upstream communication remains separate from this repository's port request and needs authorization.
 The following draft is **local only**.

~~~md
Title: Escaped surrogate reaches `char::from_u32_unchecked` in JSONC string parsing

At v0.1.5 (`6584e3495fecca3a1d73a14b8f0f11794faf5e0e`),
`from_str::<String>` routes `\uXXXX` through
`src/de/token.rs:166-178`.
Four valid hexadecimal digits can form `0xD800..=0xDFFF`,
which is not a Rust `char`.
The call to `char::from_u32_unchecked(hex)` therefore lacks its safety precondition.
`from_str_raw` also calls the same parser through `RawTokenizer::parse_string_content`.
Please do not execute the unpatched parser on `"\uD800"` to reproduce this;
the source trace establishes the invalid call without invoking undefined behavior.

A disposable patch replaces the unchecked conversion with validated four-digit parsing,
combines a valid high/low escape pair before creating a scalar,
and returns an error for isolated halves.
In a 2 GiB/2 CPU container,
`cargo test --all-targets` passed 86 unit tests,
three new surrogate tests,
and 64 existing integration tests.
The new tests check valid `"\uD83D\uDE00"`,
ordinary escaped characters,
literal backslashes,
invalid single halves,
wrong second halves,
malformed prefixes and incomplete escapes on both public string routes.
The patch is kept at `doc/troubleshooting/json-with-comments-surrogate-escape.patch`
in the Monochromatic repository and can be provided for review.
~~~

[rust-char]: https://doc.rust-lang.org/std/primitive.char.html#method.from_u32_unchecked
[issue-35]: https://github.com/hayas1/json-with-comments/issues/35
