# Audit ledger: references to titles

How a title named in running text is recognized,
glossed and rewritten.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B57: a page link's text read back to any bracket, across lines and links

Found by the tenth coverage batch (census-VW2tHL),
reading `title-reference-locate.ts:196`,
the arm where no `[` stands before the page's link.
`locateLink` read the link text as everything after `lastIndexOf('[', middle)`,
with no bound,
so a page link to the title's destination that lost its opening bracket
took the text back to any earlier `[` in the slice,
and the unify pass rewrote all of it to the heading's rendering.
A probe over the built package
(`~/temp/agent/audit-glossary-fix/title-ref-probe.mjs`,
cat-themed fixtures):
a footnote definition line ahead of such a link on the page became part of the link text and was deleted,
and so was an earlier link on the same line.
At the pin none of the page links to the 22 destinations the originals link a Han heading title to is malformed
(`b57-link-census.mjs`,
over the 214 settled pages whose entry the pin holds).

Fixed (`571e47c64` red,
`fdceec645`):
`linkTextBefore` reads leftward from the link's `](` to the nearest `[`
and reads no link where a newline or another link's `]` comes first,
or where no `[` does,
so the reference falls through to the other shapes and is reported unplaced if none holds it.
Removing the break check failed the two red cases alone.
The reading moved with the link search into `title-reference-link.ts` under B59.

Recurrence:
a span read back from an anchor is bounded by the syntax it belongs to,
here the line and the brackets of one link,
and a test gives it a malformed neighbour.

## B58: a glossed title's lead-in words rewritten away

Found by the tenth coverage batch,
reading `title-reference-unify.ts:342`,
the arm where a glossed run is too long to be the title.
`rewriteLocated` read the run before a Han gloss (`Title (标题)`) as the title
whenever it had at most two words more than the heading's rendering
(`RUN_SLACK`,
added in `2f8bcfe93` with no reason given for the two),
and rewrote the whole run,
so the words leading into the title went with it:
in the probe the credits `—— Yunmao, from Afternoon Cat Talk (午后猫语)`
and `—— Yunmao, sung in Afternoon Cat Talk (午后猫语)`
both became `—— Yunmao, Afternoon Cat Murmurs (午后猫语)`.
At the pin 2 glossed references to a Han heading title stand on the 214 pairable settled pages,
neither opening with a small letter
(`b57-gloss-run-census.mjs`).

Fixed (`344e1b79a` red,
`ae29c52fa`):
`leadInEnd` starts a glossed run's title at its first word that does not open with a small letter,
since English title case capitalizes a title's first word,
and the rewrite keeps the words before it;
small is `isSmallLetter`'s general category,
read by whole code point.
`RUN_SLACK` is gone:
a capitalized word leading in ("——From Afternoon Cat Talk") cannot be told from a title's first word,
so a glossed title with more words than the heading's rendering is reported rather than rewritten,
and a run whose every word opens small is reported with its own reason.
That a longer English rendering of the same title is now reported where it was rewritten
is a quality call open to veto:
no word is dropped unseen,
at the cost of one more finding to read.
Forcing the lead-in to none failed the lead-in and all-small cases;
restoring the two words of slack failed the capitalized case alone.

Recurrence:
a tolerance constant states the evidence for its value,
and a rewrite of a span whose edge is guessed keeps the words outside the guess or reports it.

## B59: the link and gloss searches read only the first occurrence

Found with B57 and B58,
by probe against the locator's own rule:
a slice offering two spans of one shape is reported,
not guessed at.
The bracket and quote searches kept it;
the link search took the page's first link to the destination,
and the gloss search the first `(title)`.
So where the original links the destination from another word as well as the title
(`[这里](url)` and `《[title](url)》`),
the page's first link,
"here",
was rewritten to the title,
and a slice glossing the title twice unified the first gloss only.
At the pin no destination the originals link a Han heading title to is linked twice in an original or on a page,
and no page glosses a heading title twice.

Fixed (`b926e5c9c` red,
`04788c132`):
`locateLink` (now in `title-reference-link.ts`) counts the links to the destination in the original and on the page,
takes the page's link at the title link's place where the counts agree,
and reports the reference ambiguous where they do not;
`locateGlosses` reads every gloss,
since each names the title;
and the rewrite (now in `title-reference-rewrite.ts`) returns the span it replaces,
so the renderings of one reference are each rewritten against the same text and spliced in one pass.
The two moves kept `title-reference-locate.ts` and `title-reference-unify.ts` under `max-lines`.
Taking the first link and the first gloss again failed the three red cases alone.

Recurrence:
a search whose module promises to report two spans of one shape counts its hits before taking one.

## B60: a title two headings render alike dropped as ambiguous

Found by the tenth coverage batch,
reading `title-reference-unify.ts:197` and `:209`,
the arm that drops a title two headings share.
`renderedHeadings` dropped every such title "as ambiguous",
which it is only where the headings render it apart;
rendered alike,
a reference to either has one rendering to take,
and the drop left it unrewritten.
At the pin no original heads a Han title twice.

Fixed as a quality call open to veto (`a90f7fd98`):
a shared title is kept where its renderings agree,
read through the typography fold (B24),
and dropped where they differ;
`RenderedHeading` carries every slice heading the title,
the pass skips all of them,
and the finding names them.
Comparing the renderings without the fold failed the agreeing case alone.

Recurrence:
a rule that drops a case "as ambiguous" states what makes it ambiguous and keeps the cases where that does not hold.
