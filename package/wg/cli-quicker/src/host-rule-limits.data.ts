/**
 Most addresses one update host may contribute rules for.

 Measured on 2026-10-31 through the tunnel's own DNS servers for all 21 hosts in the list:
 132 answered addresses in total,
 92 distinct,
 and at most 24 for one host (`registry.npmjs.org`, 12 A and 12 AAAA records).
 64 is about 2.7 times the largest answer,
 so ordinary CDN growth fits,
 while one hostile or broken answer with thousands of records cannot make thousands of rules.
 */
export const MAX_ADDRESSES_PER_HOST = 64;

/**
 Most distinct addresses all update hosts together may hold rules for.

 About 4 times the 92 distinct addresses measured on 2026-10-31,
 and 2.9 times the 132 answered ones.
 Every address beyond it gets no rule and is named in the loud report.
 */
export const MAX_ADDRESSES_TOTAL = 384;

/**
 Most `ip rule` commands running at once.

 The first refresh after an address-rich answer needs about a hundred additions;
 starting them all together would spawn a hundred processes in a root watcher.
 */
export const MAX_CONCURRENT_IP_COMMANDS = 8;
