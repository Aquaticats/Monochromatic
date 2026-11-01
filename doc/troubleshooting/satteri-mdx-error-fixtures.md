# Sätteri 0.6.3 invalid ESM does not exercise the MDX error vector

## Symptom

The new adapter's negative test used `export const =\n`
and expected rejection.
The native parser returned an ESM node without an entry in its error vector,
so the test panicked with `invalid MDX must be rejected`.

The test confused invalid JavaScript with a parser-reported MDX failure.
The adapter promises to reject reported errors;
it is not an independent JavaScript validator.

## Root cause

The installed `satteri-pulldown-cmark` 0.6.3 source uses Oxc's ESM result
when deciding whether to extend an ESM block across blank lines.
It does not turn the error result into an emitted MDX error at this boundary.

The read-only source checkout is `~/temp/agent/satteri-offset-source.dR6ZsYSZ`,
revision `b3d38e1e341c809b20b76a655e9b1601d11bd1f0`.
The relevant statements in `crates/satteri-pulldown-cmark/src/firstpass.rs:905-910` are:

```rust
// crates/satteri-pulldown-cmark/src/firstpass.rs:905-910, selected statements
EsmParseResult::Error => {}
// ...
return self.parse_mdx_esm(ix, ix + final_end);
```

`crates/satteri-pulldown-cmark/src/mdx.rs:1840-1851`
constructs an ESM item from that source interval:

```rust
// crates/satteri-pulldown-cmark/src/mdx.rs:1840-1851, selected statements
let content = self.text[start_ix..end_ix].trim_end_matches(['\n', '\r']);
let cow_ix = self.allocs.allocate_cow(content.into());
self.tree.append(Item {
    start: start_ix,
    end: end_ix,
    body: ItemBody::MdxEsm(cow_ix),
});
```

The value trims final line terminators,
but the node's source interval retains them.
That distinction also corrected a test expecting the node slice to omit its final LF.

In `crates/satteri-pulldown-cmark/src/arena_build.rs:1852-1867`,
the ESM item becomes an `MdxjsEsm` leaf using its existing `start` and `end`:

```rust
// crates/satteri-pulldown-cmark/src/arena_build.rs:1852-1867
ItemBody::MdxEsm(cow_ix) => {
    let cow = inner.allocs.take_cow(cow_ix);
    let sr = builder.alloc_string(&cow);
    builder.add_leaf_full(
        MdastNodeType::MdxjsEsm as u8,
        start,
        end,
        start_line,
        start_col,
        end_line,
        end_col,
        &ExpressionData { value: sr }.to_bytes(),
    );
    inner.tree.next_sibling(cur_ix);
}
```

Mismatched JSX closing tags take a different path.
`arena_build.rs:1326-1333` emits an error:

```rust
// crates/satteri-pulldown-cmark/src/arena_build.rs:1326-1333
mdx_errors.push((
    start as usize,
    format!(
        "Unexpected closing tag `</{close_name}>`, expected \
         corresponding closing tag for `<{open_name}>` ({open_loc})"
    ),
));
```

## Verification

The reproducible controls are in
`package/linter/monochromatic-lint/src/markdown_source_tests.rs`:

- `esm_node_acceptance_does_not_claim_javascript_validity`
  confirms an ESM node is constructed for the malformed JavaScript,
  remains hidden from prose checks,
  and keeps `export const =\n` as its exact source slice.
- `reported_mdx_errors_are_processing_failures`
  first asserts that the native parser actually reports a mismatched closing tag,
  then verifies adapter rejection and original byte positions,
  with and without a BOM.

Working error-boundary inputs:

- `<A>\n</B>\n`.
- `\u{feff}<A>\n</B>\n`.

Rejected fixture assumptions:

- `export const =\n` does not enter this parser error path.
- An ESM node's source slice is not identical to its newline-trimmed value.

Run the owning task:

```bash
# Repository root
mise run //package/linter/monochromatic-lint:lint:container
```

Process `proc_adf2` passed the then-current 83 tests and Clippy.
That run predates the subsequent semantic Rust conformance additions;
it is not evidence about those later checks.

## Verified correction and tradeoffs

The negative test now proves its fixture reaches the native error vector
before asserting that the adapter rejects the partial tree.
The original malformed ESM remains a separate accepted-input control,
so the limitation is not hidden by replacing its text.

No parser error is suppressed,
no production parser timeout was added,
and no upstream source was edited.
The remaining limitation is unchanged:
the parser's ESM node construction is not a complete JavaScript syntax-validation guarantee.

## What does not work

- Choosing an invalid JavaScript string without checking whether the selected parser reports it.
- Treating every accepted MDX tree as proof of valid embedded JavaScript.
- Comparing the original node interval with a value that intentionally trims final newlines.

## Upstream filing decision

The inspected `.out-of-scope/` filenames contain no Sätteri-specific exemption.
No external issue or comment is proposed.

1.  Upstream fault for the test failure: no.
    The local test assumed a stronger boundary than it exercised.
2.  Fixability: the consumer error-boundary test has been corrected and run.
3.  Supported use: the adapter consumes the native tree and reported-error vector;
    independent JavaScript validation was not its implemented contract.
4.  Contribution policy: not evaluated for filing because this record is a consumer-test correction.
5.  Expected upstream action: none is requested for that correction.
6.  Upstream prototype: not applicable to the local fixture mistake;
    the corrected consumer controls provide the relevant runnable evidence.

Upstream filing artifact:
nothing to add about the consumer-test correction.
A proposal for stricter upstream ESM validation would be a separate investigation,
not a claim established by this test failure.
