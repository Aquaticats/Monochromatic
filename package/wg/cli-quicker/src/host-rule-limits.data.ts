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

/**
 Seconds before the next refresh when every lookup of a refresh timed out.

 The live restart of 2026-10-31 found the tunnel's DNS silent for the first minute after the tunnel was created:
 all 21 lookups timed out at the ten-second limit,
 and the retry 30 seconds later succeeded.
 Five seconds after a refresh that already took ten brings the rules in as soon as the tunnel answers,
 and a persistent outage costs one set of lookups per fifteen seconds at most,
 while the delivery limits keep the loud report from repeating.
 */
export const ALL_LOOKUPS_TIMED_OUT_RETRY_SECONDS = 5;
