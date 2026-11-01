# Design commitments: the consistency pass

The commitments on rendering repeated source lines one way on a page,
and on the pass that carries words and leaves every mark where it was.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **Repeated source lines are rendered one way on a page.**
  Owner,
  2026-10-11,
  after the refrain 所以她是个……女孩吧 rendered four ways on TianqiChen666 twice:
  "Page-level consistency pass",
  the option reading:
  after assembly,
  find source sentences or refrains that repeat,
  and if their renderings differ,
  have the judges pick one rendering
  (or a refrain pattern)
  and apply it to all,
  respecting slices where context calls for different wording.
  An occurrence in a slice where the archive stands is replaced too,
  when the judged rendering passes every floor there
  (owner,
  2026-10-11:
  "Yes,
  replace it").
  A repeat with a quoted occurrence stays out of the groups
  (owner,
  2026-10-11:
  "Leave quoted repeats out").
  The repeats are located by ledger B423,
  with a repeat that touches quoted speech anywhere left out (B434)
  and no two groups claiming one span (B435);
  the pass that judges them and carries the choice to every place is ledger B431.
- **The consistency pass carries words and leaves every mark where it was.**
  Rules of the lead on 2026-10-13,
  2026-10-14 and 2026-10-15 after the independent reviews of the pass,
  and of the owner's question of 2026-10-13,
  "Can't we pre-fix the apostrophe things deterministically?"
  Recorded in ledger B463 to B626 and B628.
  Landed on 2026-10-15:
  the rules of the earlier rounds with the tests `7ae9914d3d` and the code `1fb06389e1` (by the lead's notes),
  and the shown words at a span edge,
  the pairing of typography on shown text,
  the page's definitions for every reader of a slice,
  the tie rule for `'the '90s'` and the smaller findings (B597,
  B598,
  B604,
  B605 and B609 to B615),
  with the corrections of the review that followed (B616 to B618,
  B621 and B622) and the group line's reason (B628),
  with the tests `5fe3173cc5` and the code `277145a862`.
  Open:
  the definitions inside containers (B620).
  The rows of B619 are fixed by n166,
  n168 and n169,
  landed with n171 in the tests `555bb74aa0` and the code `29d6064708`,
  with the rows left to them taken by n173 (`ff6be38eef`,
  `43428948cb`).
  Landed with them:
  a carry choice with no strict majority is run off among its tied leaders,
  with the quorum,
  window and label rules of the bench's run-off,
  and a second split keeps the page as it stands (B641;
  the lead's adoption of the owner's run-off rule of 2026-10-11,
  open to the owner's veto).
  Extended on 2026-10-18 to a lone leader (B656):
  when the choices reach the quorum and no label has a strict majority,
  the finalists are the labels with the top count;
  when exactly one label has the top count,
  the labels with the second-highest count join it.
  A tie for first behaves as before,
  and a second split keeps the page as it stands.
  The rule covers the first round of a group and a carry choice alike.
  The sheet of a carry choice's run-off says that a first round was split and the judge is asked again (B657).
  The rule holds from artifact generation 18,
  and the readers of generation 17 and earlier keep the rule they were written under,
  ties only (B664).
  The pass now stamps generation 19,
  which adds the stricter recount of a carry's outcomes (B682).
  A carry whose choice split and names finalists must record its run-off from generation 18 (B666).
  A located place of a frame also records the page definitions that move its slot from generation 18 (B665).
  It extends the owner's run-off rule of 2026-10-11 ("Run off the tied finalists"),
  which covered ties only;
  the lead proposed it,
  and the owner kept it on 2026-10-18 when asked whether a 2-1-1 split should run off the labels in second place
  or only labels tied for first;
  the consolidation's run-off of a tied slate is that rule itself and is unchanged.
  The words rule:
  the span the pass replaces is the words alone.
  The single marks that touch each edge of the span are read at the chosen place and at the receiving place.
  Eight single marks count when a splice is judged:
  `'`,
  `‘`,
  `’`,
  U+02BC,
  `‚`,
  `‛`,
  `‹` and `›`.
  The straight mark,
  `’` and U+02BC are one mark,
  and `‘` equals only itself.
  From artifact generation 17 the record of a located place stores four of them
  (`'`,
  `‘`,
  `’` and U+02BC).
  A splice is unavailable in each of these cases.
  The two places differ in a touching mark.
  The loose marks inside the two sets of words differ in sequence.
  A mark stands beside the receiving words and the edge character of the incoming words is of another class
  (a letter or digit,
  any other character,
  or nothing).
  Or the two sets of words differ in block breaks,
  since no splice of one set into the other can pass the floor that holds the page's block count.
  A loose mark is a single mark with no letter or digit directly on both sides of it.
  When a splice goes ahead each loose mark of the incoming words is written in the receiving place's form,
  so a straight mark carried onto a curly page arrives curly.
  A place the splice cannot carry is carried by a writer's paragraph when a writer was asked and the bench chooses it,
  and otherwise keeps its words.
  A writer's paragraph is withheld when it no longer holds,
  at the block's edge,
  the words of any place of the group that runs over that edge and must stay:
  a receiving place the splice cannot carry,
  the chosen place,
  a place kept for its context,
  and a place that already reads as the chosen one.
  Two groups that share a block are settled together over the English as it stood when the pass began,
  and the group that gives way keeps its places' words.
  A gain is a splice whose incoming words bring an edge mark that the receiving words lack:
  an apostrophe at the start or the end of the span.
  A gain is carried only when the apostrophe is proven.
  An edge single mark is an apostrophe only when nothing in the whole page's prose can pair with it.
  The proof reads the prose of the English:
  code,
  markup and its attributes,
  comments,
  link destinations,
  addresses and emphasis marks are blanked first.
  Every single quotation character is a possible partner.
  The backtick and `‘`,
  `‚`,
  `‛`,
  `‹` and `›` count as an opener and a closer wherever they stand,
  and `'`,
  `’` and U+02BC count by kind:
  an attached mark is inert,
  a trailing mark is a closer,
  a leading mark of any apostrophe-like form is an opener,
  and a free mark both.
  A mark reads attached,
  trailing,
  leading or free by what stands beside it:
  a trailing mark is followed by white space,
  punctuation other than a single mark,
  or the end of the text,
  a leading mark is preceded the same way,
  a letter or digit of another script,
  a symbol or a pictograph beside it makes it free,
  and so does another single mark directly beside it.
  Closing emphasis,
  strong or strikethrough delimiters from the parse and a link's closing tail
  can stand between a word and its trailing mark;
  a closing double quotation mark or a closing parenthesis cannot.
  A trailing mark is proven when no possible opener stands before it,
  and a leading mark when no possible closer stands after it.
  A proven apostrophe belongs to its word,
  is inside the span and travels with it.
  The proof is decided on the page with every edit of the pass applied:
  an edit is left out,
  with a finding,
  when the apostrophe it carries in is not proven there,
  and the check repeats until no edit drops.
  A splice that adds or removes a mark that is a quotation mark on the page,
  free or partnered,
  is refused.
  The refusal run.
  A gain is also refused when a kept mark would stand next to the arriving one for the eye.
  The pass looks across the run between the receiving words and the kept mark on the gain side,
  or one wide run away,
  and refuses when it reaches a mark.
  The run is stated by what the page shows between the two marks,
  and never by a list of characters.
  It steps over white space of every kind,
  a single line break inside a paragraph or a setext heading
  (a soft break,
  two spaces and a break,
  a backslash and a break)
  together with the container markers and the indentation that open the next line inside the same paragraph,
  a line made only of invisible or non-ASCII space characters,
  every character that shows nothing or a blank gap
  (the default-ignorable characters,
  the C0 and C1 controls other than the breaks and U+0000,
  and U+2800),
  U+0085,
  U+2028 and U+2029 as line breaks,
  the characters of emphasis,
  strikethrough and code markup (`*`,
  `_`,
  `~` and the backtick),
  literal or not,
  tags,
  footnote references and links,
  a backslash escape read as the character it shows
  (an escaped `*` steps,
  an escaped `&` is a visible character),
  and a character reference that is not itself a mark.
  It ends at a character that shows,
  a visible backslash (`\’` is no escape),
  a blank line,
  and any other block.
  Whether two marks share a paragraph is read from the page's own parse,
  with a line of invisible characters taken as inside the paragraph.
  The page-wide reference refusal:
  a character reference that decodes to a quotation-like character
  (23 named references,
  decimal ones of up to 7 digits and hexadecimal ones of up to 6)
  makes every splice of that page unavailable,
  wherever the reference stands,
  with the kept reason `edge-reference-mark`.
  It counts only where the page shows the reference as text:
  not in code,
  a tag attribute,
  a link destination or title,
  a comment,
  or behind a backslash.
  The English of every slice of the page is read,
  slices read alone included.
  Span edges are whole units on shown words.
  A backslash escape of ASCII punctuation and a character reference that the parser decodes are each one unit,
  shown as the character they stand for.
  An edge cuts or keeps a unit exactly as it would that character,
  and never splits it;
  a quote whose edge falls inside a unit is widened to the whole unit.
  Text that only looks like a reference (`Q&A;`) is no unit at an edge.
  The word test at an edge reads shown text,
  and runs on the final,
  widened edges:
  a unit counts as its shown character,
  default-ignorable characters are transparent,
  and `&` between two word characters joins them.
  A word character is a letter,
  a mark or a decimal digit of a script that sets its words apart by spaces,
  so Han,
  Hiragana,
  Katakana and U+02BC are not word characters here,
  and `_` between two word characters joins them.
  What a reader sees decides what is transparent.
  Emphasis,
  strike and code delimiters,
  link brackets with their destination and title,
  tags that wrap text,
  comments and expressions show nothing between the pieces of a word,
  so a span edge beside them is read as beside the word.
  A hidden stretch (a link destination or title,
  a tag attribute,
  a comment) hides every unit in it,
  whatever the width of the unit,
  so a reference,
  an escape or an astral character inside it is hidden whole.
  Markup that separates or annotates ends a word:
  a line break with or without a slash,
  a block tag,
  a table cell,
  an image,
  and the `rt` and `rp` of a ruby annotation.
  Inline wrappers (`b`,
  `i`,
  `em`,
  `strong`,
  `span`,
  `a`,
  `u`,
  `s`,
  `sub`,
  `sup`,
  `ruby`,
  `mark`,
  `small` and `code`) stay transparent.
  The content of an inline code span is a shown word,
  so a text place beside it is refused.
  An edge that would cut a shown word leaves the receiving place as it was,
  and refuses a chosen place.
  Detail:
  ledger B597,
  B598 and B616 to B618.
  Typography acts only on prose that the page parse shows.
  Curling a mark,
  the ellipsis,
  the guillemets and the unification of quote style of a page read one mask built from the parse.
  Front matter,
  comments,
  code of any kind,
  math,
  expressions,
  tags and their attributes,
  link destinations and titles,
  and bare addresses are no prose;
  an indented line is prose,
  since the MDX grammar has no indented code.
  A text that the grammar refuses has no prose,
  and a reader that gets such a mask logs a finding that names the slice.
  Every reader that parses a slice reads it with the page's definitions,
  so a reference label that holds a mark is no prose where its definition stands in another slice,
  and a shortcut reference is read the same way.
  An edit-region fragment is read where it lands in its slice,
  and a landing outside the slice throws `FragmentLandingError`.
  The text of a link whose label is an address is no prose.
  The pairing context of typography reads the same shown text as the word test,
  so `<b>'hi'</b>` curls and `rock *'n'* roll` takes `’n’`.
  Detail:
  ledger B605 and B609 to B615,
  B621 and B622;
  the definitions inside containers are an open item (B620).
  The elision and quoted-number readings of a straight mark on a curly page:
  a listed elision (`'tis`,
  `'twas`,
  `'til`,
  `'em`,
  `'cause`,
  `'round`,
  `'n'` between two words,
  and a decade or a year of two digits and no third such as `'90s` and `'99`)
  curls to `’` when it is unpaired,
  and when it begins a quotation the pair curls as a quotation.
  An elision inside a quotation curls to `’` and never takes the place of the opener that waits.
  A two-digit number directly followed by a closing straight mark (`'42'`) is a quoted number and pairs.
  Pairing is nested,
  emphasis and link delimiters are opening and closing context,
  and an opener followed by markup is an opener.
  A replacement whose marks still cannot all pair keeps them straight,
  with no guessed glyph.
  Between the elision and the quoted-number readings of `'NN`,
  the one that leaves fewer marks unpaired wins,
  and a tie keeps the elision,
  so `'the '90s'` curls to `‘the ’90s’`.
  The log line of a consistency group counts all nine outcomes of its places,
  and when a place was left as it stood,
  or the bench was not asked,
  failed or named no winner,
  it ends with `because=` and each distinct reason,
  each the code's own decision read from the record (ledger B628).
  Detail for each rule is in the ledger:
  the marks and the proof in B463 to B555,
  the refusal run in B557,
  B564 to B567,
  B574 and B575 to B589,
  the page-wide reference refusal in B564 and B575,
  span edges in B590 to B598,
  what the word test reads as shown in B616 to B619,
  elisions and pairing in B563,
  B568 and B599 to B605,
  the prose mask and the readers of a slice in B606 to B615 and B620 to B622,
  the cache versions in B607,
  B608 and B623,
  and the group line in B628.
  Corpus counts (the 92 English pages `people/<id>/page.en.md` of the pinned corpus,
  2,799 prose blocks,
  counted with the package's own parser):
  - The production function proves 27 trailing and 1 leading marks,
    and the pass ships 25 and 1.
  - The run finds a kept mark beyond the receiving words at 50 sites in 24 pages.
  - No page holds a reference that is a mark (0 pages).
  - The prose mask changes no character outside prose on the 92 pages
    (the 1,237 slices;
    ledger B606).
  - Typography changes 0 characters on the 92 pages under the shown-text rules and the page's definitions.
    The word test changes 21 decisions on 8 pages
    (13 from the `_` joiner and 8 from markup in a shown word),
    and the 21 pages that hold definitions change no prose flag
    (ledger B597,
    B598,
    B617).
  What a reader of a page sees as a result of the accepted limits:
  - A place whose edge mark differs from the chosen place's keeps its own wording or takes a writer's paragraph.
    So does a place whose words stand beside a mark and end in another class of character than the incoming words,
    and a place whose words differ from the chosen words in block breaks.
  - A place whose words end in emphasis,
    a closing double mark,
    a closing bracket or a link before a possessive `'s` is never located and keeps its words.
  - Carrying `the parents’` onto `the family`,
    which has no mark,
    keeps the place's words,
    with a finding,
    when another quotation mark of the page's prose could pair with the carried apostrophe.
    Where none can,
    the apostrophe arrives and stays an apostrophe.
  - A decade written `1990's` holds no stand for `1990`.
    A frame whose slot is a bare year written that way gets no slot,
    and both places keep their words.
  - A web or mail address holds no stand.
    A relative link destination is not an address and is still located.
  - A slot that stands twice inside its span leaves the place without a slot.
  - Equal words with different marks go through the writers and the bench,
    which costs model calls.
  - A page whose own apostrophes mix straight and curly keeps the form of the chosen place for an arriving apostrophe,
    since the page offers no one form to follow.
  - Quote-like characters that are none of the eight marks
    (the backtick,
    U+00B4 and U+2032)
    stay inside a span at a quote's edge,
    and a replacement replaces them.
  - A corner bracket directly beside a span is not recorded and stays where it is.
  - A block carried in part leaves the place code could not move as it was,
    with the finding `consistency-partly-carried`.
  - A quoted letter such as `the letter 'n' is` keeps the reading of `rock 'n' roll`,
    since no context separates them.
  - `<!-->` and `<!--->` are read as comments that never end,
    where CommonMark calls them complete.
  - A word in Han,
    Hiragana or Katakana has no word edge for a span to cut,
    so an edge beside such a letter is not refused,
    and U+02BC stays a mark and not a word character.
  - A definition inside a list item,
    a blockquote or an HTML block is not part of the page's definitions,
    and a link definition directly after a footnote definition is read as part of the footnote (ledger B620).
  - The names of the block tags that end a word are typed by hand,
    under a test that reads the installed parser's list and fails when a name it holds is missing (ledger B617).
  - The generation 17 reader holds no English,
    so it cannot check a recorded mark or that an edge apostrophe was proven.
    A forged record that the pass never writes
    (a byte-equal copy of another block's record,
    a mark that folds equal)
    is accepted and changes no page.
  - No property check tells an apostrophe from a quotation mark,
    since that is a judgment about the English.
    Hand fixtures cover the shapes known to matter,
    and a shape no fixture holds can still lose a quotation mark.
  Open:
  - `blockStandingOf` throws on 80,000 inline siblings (ledger B486).
  - A logger line,
    `sink verification failed for entry 3`,
    prints from three test files,
    and no test fails (ledger B486).
