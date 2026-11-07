# agent-browser 0.38.1 PDF extraction: checkmark source glyphs reorder in pdftotext 26.01.0

## Symptom

The complete Promise lesson's `mise run test:presentation` passed disclosure restoration
and the printed appendix inventory,
then failed the independent source-text inventory.
The affected entry was `ui-source:728`.
The authored reference source contained:

```js
// /home/user/temp/agent/promises-revision/chat-ui-before-ticket-print.local.mjs
responseEntry.textContent = '✓ ' + reply;
```

The PDF extraction placed the checkmark before the assignment and left the literal empty:

```text
✓
responseEntry.textContent = ' ' + reply;
```

The same displacement affected two more checkmarks in that reference source block.
The emitted PDF still contained all three glyphs,
but its extracted text did not retain their
source-code positions.
This is a **text-extraction observation**,
not a demonstrated visual
PDF defect or loss of the runtime UI glyph.

## Root cause and boundary

The precise glyph-positioning cause in the PDF renderer or Poppler is **not established**.
Do not attribute it to font fallback without a deciding source trace.
The confirmed pipeline is:

1.  The lesson renders `#ui-source` from its authored JavaScript.
    `/home/user/temp/agent/promises-revision/chat-ui-before-ticket-print.local.mjs:102-106`
    used literal checkmarks in three string values.
2.  The verifier invokes agent-browser `pdf`.
    In upstream agent-browser 0.38.1,
    commit `d01253d`,
    `cli/src/native/actions.rs:8102-8115` sends the active page to CDP:

    ```rust
    let result = mgr
        .client
        .send_command("Page.printToPDF", Some(params), Some(&session_id))
        .await?;
    ```

3.  `/home/user/temp/agent/promises-revision/verify-presentation.mjs:149-160`
    calls `pdftotext -layout` on that PDF,
    removes whitespace,
    and compares
    source passages character-for-character against its independent DOM inventory.
    This detects the glyph displacement even though source text is otherwise complete.

No upstream source was modified.
The agent-browser checkout's `origin` is
`https://github.com/vercel-labs/agent-browser.git`;
its `cli/Cargo.toml:3` identifies version 0.38.1.

## Verification

Measured versions:
agent-browser 0.38.1 and `pdftotext` (Poppler) 26.01.0.
The failing `proc_3572` ran `mise run test:foundations && mise run test:presentation`
with literal `✓` in the reference source.
The failure listed only the `ui-source` passage
after the scene's intentionally hidden labels received printable counterparts.

`mise run probe:ticket-print-gap` located the first mismatch after 4,406 compacted characters.
Removing only `✓` from both streams made all 9,146 remaining compacted source characters
match in order.
This is a positive control that the comparison can detect displacement
without silently dropping the entire reference module.

Working catalog:

- `mise run test:presentation` in `proc_270f` passed after the authored source used
  Unicode escape spellings for the same runtime strings.
- `mise run test:reference-sending` and `mise run test:exports` passed in `proc_9c4f`,
  exercising the reference UI and its standalone download after that source change.

Failing catalog:

- Literal checkmarks in the reference's JavaScript strings caused exact PDF text inventory
  failure under the recorded generated-lesson layout in `proc_3572`.
- Plain whitespace compaction did not repair the reordered glyph positions.

## Verified workaround

Keep the runtime character while writing ASCII escape syntax into the authored JavaScript:

```js
// /home/user/temp/agent/promises-revision/chat-ui.mjs
responseEntry.textContent = '\u2713 ' + reply;
progress.textContent = '\u2713 Reply received for this send';
thread.status.textContent = label + ' · \u2713 Send fulfilled';
```

JavaScript evaluates each escape to the same checkmark in the running UI.
The printed source is now ASCII at those positions,
so the PDF source inventory passes.
The unchanged runtime result was exercised by the reference-sending and export verifiers.
Tradeoff:
the displayed source is less immediately legible than literal checkmarks.
The lesson's actual chat text remains the same.

## What does not work

- Excluding `#ui-source` from the independent print inventory would hide a real content-loss
  regression and was not used.
- Treating a successful `Page.printToPDF` call as proof that copied PDF source is in order
  does not test the extraction boundary.
- Changing disclosure restoration would target a different print lifecycle issue;
  disclosure state already passed during this incident.

## Upstream filing decision

The repository's `.out-of-scope/` directory was checked;
no matching exemption was found.
A search of the agent-browser issue and pull-request tracker for
`PDF Unicode glyph pdftotext ordering` returned no match.
No issue or comment is proposed because upstream responsibility is unproven.

1.  **Upstream fault?**
    Not established.
    CDP PDF generation and Poppler extraction are
    different operations;
    the deciding positioning source has not been traced.
2.  **Can upstream change it?**
    Possibly,
    but no specific upstream correction is identified.
3.  **Supported use case?**
    agent-browser exposes PDF generation;
    exact ordering in a
    separate extractor is not established as its guarantee.
4.  **Would upstream welcome a contribution?**
    Not assessed because no fault is attributed.
5.  **Likely fix?**
    No maintainer decision about this combined pipeline was found.
6.  **Minimal compatible upstream fix prototyped?**
    No. The verified change is in our
    authored JavaScript source;
    the upstream-fix gate does not activate while constraint 1 fails.

There is no fileable upstream draft or additive comment from this evidence.
