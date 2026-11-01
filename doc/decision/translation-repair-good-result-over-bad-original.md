# The pipeline's job is a good result even when the originals are bad

Decided by the owner on 2026-09-03,
in these words,
in answer to a question that should not have been
asked:
"What?
Are you misunderstanding the pipeline's job?
The pipeline's job is to give a good result
even when the originals are bad."

## The question that prompted it

XIEPT2's archive English page (`people/XIEPT2/page.en.md` at the pinned corpus) is a stub:
front matter,
the line `(To-Do)`,
an HTML comment of translator hints in Chinese,
and nothing else that the ORIGINAL
says.
The pipeline translated the whole ORIGINAL below that opening (27 of 35 slices changed on the
2026-09-03 postscript run) and carried the stub marker and the hint comment through unchanged as archive
text,
so the published page opens with `(To-Do)` over a finished translation.
The agent asked whether to
strip the marker,
strip both,
or keep both as archive text.

## The rule

- An archive that is wrong,
  missing or a placeholder is an input the pipeline must improve on,
  not text
  it must preserve.
  "Never delete archive text" protects a reader from losing content the ORIGINAL
  carries;
  a stub marker and an instruction to a future translator are not content the ORIGINAL carries.
- The same principle already decided the translate lane's existence (a missing page is translated,
  not
  refused),
  the insertion admission (a source passage the archive omitted is filled),
  and the naturalness
  refinement (an archive that reads badly is rewritten).
  This record names the principle so the next
  bad-original case is handled,
  not asked.
- What ships from a stubbed archive:
  the front matter,
  the translation,
  and the archive's HTML comments.
  The reader-visible stub marker does not ship.
  The comments stay because a reader never sees them and
  the pipeline reads them:
  `entry-notes.ts` turns every archive comment into an "ARCHIVE editor
  comment" line of the identity block,
  which is where 起床战争：Bed Wars and WER:
  World Educational
  Robot Contest reach the translators and judges (22 of 92 English pages carry such comments;
  the
  corpus carries exactly one stub marker,
  XIEPT2's).
  Where the archive carries real content beside the
  stub,
  that content is judged as archive text is judged everywhere else.
- The seam is the archive as the pipeline reads it (`corpus-run/pass-archive.ts`),
  so preparation,
  both lanes,
  the artifact's stored archive and the published page all describe the same text and no
  later check has to know a marker was ever there.

## What this is not

- Not a licence to drop archive text on the pipeline's own judgment of quality:
  the contest,
  the
  consolidation and the gates decide between renderings of content.
  This record covers text that is not
  a rendering of anything in the ORIGINAL.

## Where the evidence lives

- `doc/planning/translation-repair-roster-calibration-2026-09-01.md`,
  "XIEPT2 on the postscript rule".
- The published page:
  `~/temp/agent/xiept2-postscript-20260903/fixed/people/XIEPT2/page.en.md`,
  line 8.
