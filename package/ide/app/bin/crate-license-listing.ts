// The crate list a `--licenses` section prints, read with cursor scans instead of regular expressions.
// What: the label that starts a section's crate list, and the `name version` pairs the list holds.
// Why: what the executable prints is the specification, so every scan here answers exactly what the pattern it
// replaced answered. The printed form is `Used by: ` followed by `name version` items ending in commas, wrapped at
// the width the Rust side formats to, with each continuation line indented by the label's length.

/// The label that starts a section's crate list; every continuation line holds spaces of the same length instead.
const USED_BY_PREFIX = 'Used by: ';

/**
 The crate list of one license section without the label it starts with.

 What: a prefix comparison, the answer an anchored `Used by: ` pattern gives at the start of the block.

 Why: only the first line carries the label; the continuations carry spaces, so a block that starts with anything else
 is returned unchanged rather than dropped.

 @param usedBy - one section's crate list, its lines joined
 @returns the list without its label
 */
export function withoutUsedByPrefix(usedBy: string): string {
  return usedBy.startsWith(USED_BY_PREFIX) ? usedBy.slice(USED_BY_PREFIX.length) : usedBy;
}
