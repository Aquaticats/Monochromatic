# Second-pass measurements of the wrapper's releases

What the second passes over a release measured with more than one built
binary,
and the one figure of a release that no pass explains.

Status:
**unapproved and non-final**.
Written by the coordinating session from the second passes' reports.
Each pass was another agent of the same vendor,
so a second pass and not an independent review,
and each is named by the release it read.

## The alpha.49 structure pass: 61 commands through three builds

Of 61 commands run through `0.6.1-alpha.48`,
`0.6.1-alpha.49` and `0.6.1-alpha.49` again as the control,
57 were identical and the control differed in none.
The four that differed are the merge pass's intended change.
The pass read the two merges that claim to change no behaviour
(`02d0a6e8e6` and `97d9afb21c`),
and that control is what makes its reading of them a measurement of the built
binary rather than of the source.

## The release binary's size between alpha.48 and alpha.49

The release binary's code grew by about 112 KiB between `0.6.1-alpha.48` and
`0.6.1-alpha.49`.
Neither of the two second passes over `0.6.1-alpha.49` says which merge grew it,
so the figure stands with no cause beside it.
No record measures this binary's size before these two releases,
so nothing here says whether the growth is unusual.
