# Audit ledger: credentials masked at the transport

Credential spellings that could reach a log line,
a refusal or a listing unmasked.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B266: a credential the request sent could leave the transport inside a reply body

Red in `54b3d2535`,
fixed in `a29a9dc42`.

Found on 2026-10-06 (UTC) by the follow-up to the census of B232,
which showed that a provider's reply body reaches `SyntheticHttpError`'s (renamed `ProviderHttpError` on 2026-10-19)
message and `bodyExcerpt`,
the raw reply preview `model-health.ts` prints,
a malformed-completion message and a cached reply.
A provider that echoes the sent `Authorization` value in an authentication failure,
or inside an error event of a 200 stream,
handed the key to every reader of the body.
Nothing measured did this;
nothing proved none did.
The red cases script a reply echoing a stand-in key,
the whole header value and the bare token,
in a failure reply,
in a stream split across two chunks,
and in the partial text of a cut stream.

The fix:
`fetchTransport` masks every credential the request's headers carried out of the assembled reply body
(`maskCredentials`,
new `credential-mask.ts`):
the whole header value and the token after its scheme word,
as written,
as a JSON string holds it and with the slash escaped,
each found in one linear pass with no regular expression built from a key.
A successful reply is masked too,
since an error event can arrive inside a 200 stream,
and so is the partial text a cut stream carries on its error.
The marker is `[masked: credential sent with this request]`;
a secret under 12 UTF-16 units is left alone.

The agent's measurement,
not rerun by the lead:
masking costs about 28 ms per megabyte of body and grows linearly,
a body of repeated prefixes included.
Measured by the lead for B266 to B270:
86 named test files pass,
among them every test file that holds the wording of a lost voice,
a refusal or a provider's excerpt,
and the transport and client suites,
with the 39 source scans and a clean type check.
The whole suite ran on `a29a9dc42` in a side checkout:
1,638 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 39 source scans passing.

Left open at this commit and closed since:
the opening excerpt the stream drain logs (B293),
the listings of the model catalogue and the roster card (B295),
and the two Exa lookups (B300),
each of which read a reply the transport's mask never covered;
and a key in URL-encoded or base64 form,
which the mask did not search for (B329).

Open to the owner's veto:
the minimum of 12 units;
the marker's wording;
masking the cut stream's partial text in the transport.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B293: the stream drain's opening line was never read by the credential mask

Red in `3c74be74a`,
fixed in `040656c51`.

Found on 2026-10-06 (UTC) as the path B266 left open.
`drainBody` logs an opening excerpt of the generated text,
and the upstream's name,
through a line the credential mask of the transport never read.
An endpoint that echoed the key it was sent would have put the key in the log.
No such echo was seen;
the cases use a fixture's invented key,
never one from the environment.

The fix:
`drainBody` takes the request's credentials,
which `fetchTransport` already holds and passes.
The whole assembled opening text is masked before the excerpt is cut from it,
so a key split across chunks or frames is masked whole.
A stream torn down inside a key holds no whole copy,
so `maskCredentialsInCutText` (`credential-mask.ts`) also masks an ending of four units or more
that is the opening of a credential.
The upstream's name is masked too.
`stream-cut.ts`,
which writes the line,
is unchanged.
Four red cases expect a sent credential masked:
echoed whole in the first content;
arriving in two chunks or two frames;
cut inside the key,
where no head of it may show;
and in the gateway's name field.
A fifth case,
a stream that never echoes the key,
pins the line as it was and passes before and after.
`credential-mask.unit.test.ts` gains three cases for the new function.

The lead's edit:
`drainBody` requires its `credentials` argument,
where the agent gave it a default of none.
A default would let a later caller start a drain that masks nothing without saying so.
The transport is the one production caller and names its list,
and each test call that carries none says `credentials: []`.
With the argument made required,
the suite and the scans each failed on one case alone,
the TSDoc example scan naming the example of `drainBody` that lacked the key;
with the key in the example that scan passes run alone,
and nothing but that one comment line changed after the last whole run on the lead's tree.

The agent's measurements,
not rerun by the lead:
over a stream of one megabyte the masked and unmasked drains differ by less than the spread between runs
(25 runs each after a warm-up),
with about 5 ms more at the median where one frame carries the whole megabyte.
The whole suite's run on `040656c51` is recorded in B291.

Open to the owner's veto:
four units as the shortest head masked at a cut,
which leaves the first three units of a key to show there
and masks an excerpt's ending that happens to match a key's head;
the credentials as a parameter of `drainBody`,
where masking inside `stream-cut.ts` is the alternative;
and masking the upstream's name.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries":
every line that carries a provider's text is read by the mask before it is cut.

## B295: two listing readers parsed a provider's reply before any mask read it

Red in `17a66fa66`,
fixed in `097b6ac40`,
whose masking cases the agent ran red in its worktree.

Found on 2026-10-06 (UTC) while moving six sampler and card runners into modules (T8),
as a path B266 left.
`model-catalog` and `roster-card` read a provider's listing with `reply.json()`
and never passed its text through the credential mask.
A reply that echoed the key would have been parsed,
and a catalog id carrying it printed.
No such echo was seen;
the cases use an invented key.

The fix:
both read through `readProviderListing`,
which masks the text before it is parsed,
whatever transport it is handed.

The agent's account,
not rerun by the lead:
its masking cases failed before the mask was written;
it did not take the mask out again afterwards to see them fail a second time.
The whole suite ran on `097b6ac40` in a side checkout,
which holds the fixes of B295 to B298:
1,886 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open to the owner's veto:
a limit of 60 seconds on a listing read,
where there was none;
the mask in `readProviderListing` on top of the transport's own;
listing reads through `fetchTransport` in place of the global `fetch`;
a listing status read as 200 to 299.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries":
a reply is masked as text before anything parses it.

## B300: the two Exa lookups parsed and kept a reply before any mask read it

Red in `251df99c4`,
fixed in `c67279f66`.

Found on 2026-10-06 (UTC) among the paths the agent of B266 listed as not reached
(its list,
in the lead's notes).
`searchWorkTitle` (`work-title-search.ts`) and `fetchCitedReference` (`cited-reference-fetch.ts`)
send the Exa key in an `x-api-key` header and read the answer with `response.json()`.
Neither passed the answer's text through the credential mask,
and neither goes through the transport that masks for the chat clients,
since each takes the `fetch` it is handed,
and a pass hands them the global `fetch` (`corpus-run/pass-outside-reads.ts`).
What they read is kept:
a hit's title,
address and highlight,
and a fetched page's title and text,
go into a sheet a model reads and into the lookup and reference caches on disk.
An endpoint that echoed the key it was sent would have put the key in all three places.
No such echo was seen;
the cases use an invented key from a fixture.

The fix:
both read their reply through `readMaskedJsonBody` (`masked-json-body.ts`).
The body's text is taken once,
every credential the request's headers carried is masked out of it
(`credentialsOfHeaders`,
`maskCredentials`,
the pair the transport uses),
and only then is it parsed.
A parse failure's message,
which quotes the text it refused,
quotes the masked text.
Each lookup builds its headers once and hands the same object to the request and to the read,
so the read cannot mask a different key from the one that was sent.
`masked-json-body.unit.test.ts` holds four cases.

Measured by the lead on the red tree:
`work-title-lookup.unit.test.ts` and `cited-reference-fetch.unit.test.ts` each fail in the one new case,
by `AssertionError`,
and in no other.
On the lead's fix tree the whole unit suite printed 1,899 pass lines and 2 fail lines,
both for one case this change does not touch:
`displacement-probe as built`,
"REFUSES as stated and exits 6 when an artifact names a commit the clone lacks"
(the case and its suite).
Its file,
rerun alone on the same build,
printed one pass line and no fail line.
The whole suite then ran on `c67279f66` in a side checkout:
1,899 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open:
that case fails under load.
The lead's reading,
an inference not measured in any commit through `f8e99b84d`:
the probe reads an entry's two pages with `Promise.all` (`corpus-run/settled-carve.ts`),
both reads fail for a commit the clone lacks,
and which page the refusal names is whichever git child ends first.
It failed again in the whole-suite run on `f8e99b84d` (B305),
and its fix is in no commit through that one.
Measured and fixed since,
in `f3c2bae13` (B316).

Open to the owner's veto:
masking at the read and not by handing the lookups the masking transport;
one helper for both lookups.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries",
and for the case that fails under load,
"Output decided by arrival order".

## B329: a credential a provider echoed in another spelling left the transport unmasked

Red in `03ec6a4fc`,
fixed in `9a32c49`.

Found on 2026-10-06 (UTC) by a census of the forms a credential can take in a reply,
the item B266 left open:
a key in URL-encoded or base64 form was not searched for.
`maskCredentials` searched the header value and the token after its scheme word as written,
as a JSON string holds them and with the slash escaped.
A provider,
proxy or HTTP library that echoed the same key percent-encoded
(whole or in its reserved characters only,
in either hex case),
as a form value,
in base64 or base64url
(alone,
padded or not,
or inside a longer run at each of the three alignments),
as JSON unicode escapes
(once or inside a JSON string that is itself inside one),
percent-encoded inside a JSON string,
or as the decoded pair of a `Basic` header or the decoded form of a percent-escaped header,
handed it to every reader of the reply.
Nothing measured did this;
nothing proved none did.
By the agent's account,
nine cases of `credential-mask.unit.test.ts` failed by assertion on the old mask in its own worktree;
at the red commit that file cannot load,
since its cases read the needles through `needlesOf`,
which the fix exports,
so the gap shows there through the four surfaces that print a reply:
the transport's failure reply (`synthetic-transport`),
the masked JSON read of the two lookups (`masked-json-body`),
the listing (`corpus-run/provider-listing`)
and the cut stream's opening (`stream-drain`).
The lint failed at the red commit,
19 errors and 103 warnings,
each a use of an export the fix adds.

The fix:
`maskCredentials` finds every spelling of every credential in one Aho-Corasick pass
(`credential-search.ts`,
in place of a Knuth-Morris-Pratt pass per spelling);
reads the reply through JSON string escapes and percent escapes,
chained up to three deep,
each unit of a decoded reading mapped back to the stretch of the original it came from
(`credential-views.ts`);
and searches each credential as it stands,
as JSON writes it,
as a form value,
as the bytes of its UTF-8,
and in base64 and base64url at each of the three alignments a prefix of unknown length gives
(`credential-needles.ts`).
A base64 find takes with it the one character at each edge that holds bits of the key,
and the run's padding after the character that ends it.
`credentialsOfHeaders` adds what a decoder reads out of a header's secret:
a base64 body that spells printable text and each side of its first colon,
and the secret with its escapes resolved (`credential-decoded-forms.ts`).

The lead's changes to the agent's patch:
the widening counts only characters that hold bits,
so an `=` is no longer one of them,
and padding is taken only after a tail character the find took;
`Needle.padded`,
which marked every base64 needle and so had the mask take the `=` after a key ending on a whole group of three bytes,
is gone,
and two of the lead's cases hold both edges;
`FIXED_IN_TAIL`,
a table mapping zero,
one and two to themselves read with a `?? 0`,
is the arithmetic it stood for;
reads that cannot miss
(a view's typed-array offsets,
a find's needle,
a reach mapped back to the original)
throw on a miss,
where a zero would have placed a marker at the text's start and left the credential where it stands;
and three comment lines of `credential-views.ts` that held a hyphen where they mean its JSON escape
hold the escape (M135).

By the agent's account,
not rerun by the lead:
the cost per reply is one search pass
plus one decode pass per decoder chain that finds an escape decoding to a unit some needle holds;
under load,
plain replies within 1.1 times the old mask,
ordinary JSON replies within 1.4 times,
and the worst shape measured,
escaped backslashes in every frame,
12 to 20 times at 256 KB to 4 MB,
about 20 ms at 256 KB.
Out of reach by rule:
a key split by a line fold or a soft hyphen,
a spelling no decoder named here recovers
(hex,
HTML entities,
`\xHH`,
`\u{...}`,
a cipher,
a hash,
a case-folded key,
base64 of bytes that are no UTF-8 of the key),
and nesting past three decoders.
The whole suite on `9a32c49`,
run in a side checkout:
2,017 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 44 source scans passing.

Open:
by the agent's inference,
not measured,
`stream-drain.ts` logs `response.url` near lines 55 and 376,
which after a redirect is an address the provider chose,
unmasked;
and the `blind-map-writes` scan reads a typed array's `set` as a Map write,
so the patch wrote loops where `set` was the natural code.
Both closed since:
the address by B341,
the scan by B342.

Open to the owner's veto,
the agent's:
every credential header's secret is decoded,
not only a `Basic` one;
Aho-Corasick in place of the pass per spelling;
three decoders deep;
base64 needles for the whole header value as well as the token;
the plus form of every spelling;
the head minimum of four units for a cut text applies to the base64 spellings too;
the four helpers exported for built-artifact tests.
The lead's:
padding taken only after a tail character.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries".

## B341: the stream drain's cancel line logged a redirected address unmasked

Red in `de6ff17`,
fixed in `a7eca7f`.

Found on 2026-10-06 (UTC),
the redirected address B329 left open.
`stopReading` (`stream-drain.ts`) logged `response.url` when a cancel failed,
and after a redirect that is the address the redirect named,
its path and query as the provider wrote them.
By the agent's probe with node's `fetch` against a local server,
a `302` to `/landing?key=<key>` left that address whole in `response.url`,
and the cancel line printed the key.
`fetch` refuses a redirect to an address with userinfo,
and no request the package sends carries a key in its address.

The fix:
`stopReading` takes the request's credentials
and logs the address through `maskCredentials`.
The case "MASKS THE CREDENTIAL A REDIRECT LEFT IN THE ADDRESS the failed cancel names"
gives its response the address the probe measured `fetch` leaving.

Recurrence:
`mistake-prevention.md`,
"Messages a marked class carries":
an address a provider chose is reply text,
masked before anything logs it.
