# Rust Book companion

This workspace contains page-by-page companion material for
[The Rust Programming Language: Experimental Edition](https://rust-book.cs.brown.edu/).
It supplements the book with retrieval practice,
 TypeScript comparisons,
 and links into this monorepo's Rust source.
It does not reproduce or replace the book.

## Use

1. Open the matching file in `lesson/` directly in a browser.
2. Read the source page linked at the start of the lesson.
3. If the source uses unexplained labels or diagrams,
    open the reference linked by the lesson.
4. Close or hide the source page before attempting retrieval prompts.
5. Reveal feedback only after committing to an answer.
6. Ask the teaching agent about anything unclear.
7. Revisit the linked reference material when the concept appears in monorepo code.

## Structure

- `lesson/`:
   sequential companions named `NNNN-<source-page>.html`.
- `reference/`:
   durable,
   compressed concept references linked by lessons.
- `learning-record/`:
   demonstrated understanding and established prior knowledge.
- `MISSION.md`:
   concrete learning outcome and boundaries.
- `RESOURCES.md`:
   curated primary sources and communities.
- `NOTES.md`:
   teaching and presentation preferences.

## HTML contract

- Every HTML file is complete and functional when opened through a `file:///` URL.
- This workspace has no build tasks.
- Every HTML file repeats the approved automatic-dark-mode CSS inline.
- Any lesson-specific JavaScript is embedded in that HTML file.
- Graphs and diagrams use HTML and CSS rather than generated images.
- TypeScript comparisons are self-contained and copyable without a Playground link.
- Routine edits do not require `agent-browser` verification.
  Use browser verification when a change introduces JavaScript,
   a new interaction,
   or browser-dependent behavior.
