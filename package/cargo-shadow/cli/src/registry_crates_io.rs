//! What:
//! crates.io through a fetcher:
//! the addresses and limits of the sparse index and the web API,
//! the registry that spaces its API requests and retries an HTTP 429,
//! and the history cache it reads before it asks.
//! Why:
//! The API allows about one request per second,
//! so requests are spaced and a 429 is retried after its `Retry-After`,
//! or a doubling backoff,
//! at most five times;
//! and the dump replaces a thousand of those requests when enough crates are uncached.
//! Split out of `registry.rs`;
//! the constants and `CratesIo` are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // registry_crates_io.ts: the API limits and the CratesIo class that implements Registry
//! ```

/// What:
/// `use crate::dump::fill_histories_from_dump;` imports the one streaming pass over the database dump.
/// Why:
/// It fills the cache for every uncached crate at once,
/// instead of one rate-limited request per crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fill_histories_from_dump } from 'crate/dump';
/// ```
use crate::dump::fill_histories_from_dump;
/// What:
/// `use crate::http::{DumpSource, RegistryError, TextFetcher};` imports the one HTTP GET the lookups need,
/// the failure of a lookup and the dump's byte stream.
/// Why:
/// The registry holds one fetcher and an optional dump,
/// and every answer it cannot use is a `RegistryError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DumpSource, RegistryError, TextFetcher } from 'crate/http';
/// ```
use crate::http::{
    DumpSource,
    RegistryError,
    TextFetcher,
};
/// What:
/// `use crate::policy::VersionRecord;` imports the record of one published release that the trust rule reads.
/// Why:
/// The history the registry answers with is a list of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { VersionRecord } from 'crate/policy';
/// ```
use crate::policy::VersionRecord;
/// What:
/// `use crate::registry::{Registry, sparse_index_path};` imports the trait this file implements and the index path of one crate.
/// Why:
/// The answers are the trait's methods,
/// and the sparse index address is built from that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry, sparse_index_path } from 'crate/registry';
/// ```
use crate::registry::{
    Registry,
    sparse_index_path,
};
/// What:
/// `use crate::registry_record_reading::{history_to_json, parse_index_pubtimes, parse_index_yanked, parse_owner_ids, parse_version_history, read_cached_history};`
/// imports the readers of the answers this file fetches and the one writer of the cache file.
/// Why:
/// The client only moves text;
/// every text it receives is turned into records by those readers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { history_to_json, parse_index_pubtimes, parse_index_yanked, parse_owner_ids, parse_version_history, read_cached_history } from 'crate/registry_record_reading';
/// ```
use crate::registry_record_reading::{
    history_to_json,
    parse_index_pubtimes,
    parse_index_yanked,
    parse_owner_ids,
    parse_version_history,
    read_cached_history,
};
/// What:
/// `use std::cell::Cell;` imports a box whose contents can be replaced through a shared reference (sibling:
/// `RefCell`,
/// which also works for types that are not copied).
/// Why:
/// A method that takes `&self` still has to remember when it last made a request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a plain mutable field on the object
/// ```
use std::cell::Cell;
/// What:
/// `use std::collections::{BTreeMap, BTreeSet};` imports the sorted map and the sorted set (siblings:
/// `HashMap`/`HashSet`,
/// unordered and faster).
/// Why:
/// The trait answers publish times and yanked versions as sorted collections,
/// and the uncached names are kept sorted so two runs ask in one order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // new Map() / new Set(), which keep insertion order
/// ```
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// A fetched history is written to the cache,
/// and a cached history is looked for before the dump is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { writeFileSync, mkdirSync, existsSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The cache directory is produced here and outlives the constructor,
/// so it is owned,
/// while the repository root is only lent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::thread::sleep;` imports the call that stops this thread for a length of time.
/// Why:
/// The spacing between two requests and the pause after an HTTP 429 are both slept through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await new Promise((resolve) => setTimeout(resolve, ms))
/// ```
use std::thread::sleep;
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock (sibling of `Instant`:
/// `SystemTime`,
/// the wall clock,
/// which can jump).
/// Why:
/// The spacing is measured on a clock that never jumps back,
/// so a request is never sent sooner than the interval allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a number of milliseconds
/// ```
use std::time::{
    Duration,
    Instant,
};
/// The sparse index host.
const SPARSE_INDEX_URL: &str = "https://index.crates.io";

/// The web API root.
const API_URL: &str = "https://crates.io/api/v1";

/// The least time between two API requests:
/// crates.io allows one per second,
/// plus a margin.
pub const API_MIN_INTERVAL: Duration = Duration::from_millis(1_100);

/// How many times a rate-limited request is tried.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// The number of 429 answers crates.io gives in a row was not observed.
const RATE_LIMIT_ATTEMPTS: u32 = 5;

/// The longest pause for an HTTP 429 that cargo-shadow sits through.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// No `Retry-After` value was observed in this repository's logs or runs.
/// Two minutes assumes crates.io asks for seconds;
/// a longer request ends the call,
/// so the wait never grows past this.
const MAX_PAUSE: Duration = Duration::from_secs(120);

/// The first backoff when a 429 carries no `Retry-After`;
/// it doubles each attempt.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
/// Used only when a 429 carries no `Retry-After`;
/// the values crates.io sends were not observed.
const RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(30);

/// The HTTP status for "too many requests".
const TOO_MANY_REQUESTS: u16 = 429;

/// Where histories are cached,
/// relative to the repository root.
pub const HISTORY_CACHE_DIR: &str = ".cache/cargo-shadow/crates-io";

/// The database dump,
/// a gzip stream of a tar archive.
pub const DATABASE_DUMP_URL: &str = "https://static.crates.io/db-dump.tar.gz";

/// How many uncached crates make the dump cheaper than the API (one request per 1.1 seconds each).
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
pub const DUMP_BREAKEVEN_CRATES: usize = 100;

/// What:
/// `fn registry_error_of_dump(error: DumpFormatError) -> RegistryError` words a malformed dump as a registry failure with no status.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
fn registry_error_of_dump(error: crate::csv::DumpFormatError) -> RegistryError {
    return RegistryError {
        message: error.message,
        status: None,
        retry_after_seconds: 0,
    };
}

/// crates.io through a fetcher,
/// with spaced API requests and the on-disk history cache.
pub struct CratesIo {
    /// What:
    /// Field `cache_dir: PathBuf` is the directory of cached version histories (`PathBuf` is an owned path;
    /// sibling:
    /// `&Path`,
    /// borrowed).
    /// Why:
    /// A published version never changes,
    /// so a history that lists every version being checked is read from disk instead of the network.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cacheDir: string
    /// ```
    cache_dir: PathBuf,
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    /// What:
    /// Field `fetcher: Box<dyn TextFetcher>` is the HTTP source for the index and the API (`Box<dyn Trait>`:
    /// any implementation behind one owning pointer).
    /// Why:
    /// Tests substitute a fetcher that answers from memory or fails on purpose.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fetcher: TextFetcher
    /// ```
    fetcher: Box<dyn TextFetcher>,
    // `dyn` trait object `dyn DumpSource`: any type that implements the trait.
    /// What:
    /// Field `dump` is the optional source of the database dump (`Option`:
    /// it may be absent).
    /// Why:
    /// A registry built without it never reads the dump and uses the API for every crate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dump?: DumpSource
    /// ```
    dump: Option<Box<dyn DumpSource>>,
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    /// What:
    /// Field `dump_breakeven: usize` is how many uncached crates make the dump cheaper than the API (`usize`,
    /// a count).
    /// Why:
    /// The dump costs a download of gigabytes,
    /// the API one request per crate at 1.1 seconds,
    /// so below this count the API wins.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// dumpBreakeven: number
    /// ```
    dump_breakeven: usize,
    /// What:
    /// Field `min_interval: Duration` is the least time between two API requests.
    /// Why:
    /// crates.io allows about one per second;
    /// sending faster earns HTTP 429.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// minIntervalMs: number
    /// ```
    min_interval: Duration,
    /// What:
    /// Field `backoff: Duration` is the first wait after an HTTP 429 that carries no `Retry-After`;
    /// it doubles each attempt.
    /// Why:
    /// The registry retries instead of failing the whole install on a busy minute.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// backoffMs: number
    /// ```
    backoff: Duration,
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    /// What:
    /// Field `last_request` remembers when the last API request started (`Cell` lets a method that only borrows `&self` still replace the value).
    /// Why:
    /// The spacing must hold across calls made through a shared reference to the registry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lastRequestAt?: number
    /// ```
    last_request: Cell<Option<Instant>>,
}

/// What:
/// `impl CratesIo { ... }` attaches the constructors and the spaced API call to the registry struct.
/// Why:
/// The constructors choose real or test timing,
/// and one method owns the rate-limit logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CratesIo { constructor(...) {} spacedApiGet(url) {} }
/// ```
impl CratesIo {
    /// Builds the registry for a repository root with the real spacing and backoff.
    pub fn with_defaults(
        root: &Path,
        fetcher: Box<dyn TextFetcher>,
        dump: Box<dyn DumpSource>,
    ) -> CratesIo {
        // `let mut`: a binding that may change.
        // Typed binding: `registry: CratesIo`.
        let mut registry: CratesIo =
            CratesIo::new(root, fetcher, API_MIN_INTERVAL, RATE_LIMIT_BACKOFF);
        // `Some`: the present or the absent value of an `Option`.
        registry.dump = Some(dump);
        return registry;
    }

    /// Sets how many uncached crates make the dump worth reading;
    /// tests lower it.
    pub fn with_dump(
        mut self,
        dump: Box<dyn DumpSource>,
        breakeven: usize,
    ) -> CratesIo {
        // `Some`: the present or the absent value of an `Option`.
        self.dump = Some(dump);
        self.dump_breakeven = breakeven;
        return self;
    }

    /// Builds the registry for a repository root;
    /// `min_interval` and `backoff` are parameters so tests do not wait.
    pub fn new(
        root: &Path,
        fetcher: Box<dyn TextFetcher>,
        min_interval: Duration,
        backoff: Duration,
    ) -> CratesIo {
        return CratesIo {
            cache_dir: root.join(HISTORY_CACHE_DIR),
            fetcher,
            // `None`: the present or the absent value of an `Option`.
            dump: None,
            dump_breakeven: DUMP_BREAKEVEN_CRATES,
            min_interval,
            backoff,
            // `None`: the present or the absent value of an `Option`.
            // Constructor `Cell::new(`: an empty owned collection or a boxed value.
            last_request: Cell::new(None),
        };
    }

    /// One API GET:
    /// waits out the spacing,
    /// then retries an HTTP 429 up to the attempt limit.
    fn spaced_api_get(
        &self,
        url: &str,
    ) -> Result<String, RegistryError> {
        // `let mut`: a binding that may change.
        // Typed binding: `attempt: u32`.
        let mut attempt: u32 = 1;
        loop {
            // `Some`: the present or the absent value of an `Option`.
            // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
            // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (value !== undefined) { const inner = value; ... }
            // ```
            if let Some(last) = self.last_request.get() {
                // Typed binding: `elapsed: Duration`.
                let elapsed: Duration = last.elapsed();
                if elapsed < self.min_interval {
                    sleep(self.min_interval - elapsed);
                }
            }
            // `Some`: the present or the absent value of an `Option`.
            self.last_request.set(Some(Instant::now()));
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: fetching {url}");
            // `match`: picks the arm whose pattern fits the value.
            match self.fetcher.get(url) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(body) => return Ok(body),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(error) => {
                    // `Some`: the present or the absent value of an `Option`.
                    // Closure `||`: an anonymous function, like a TS arrow function.
                    if error.status != Some(TOO_MANY_REQUESTS) || attempt >= RATE_LIMIT_ATTEMPTS {
                        // `Err(...)`: builds the success or the failure variant of a `Result`.
                        return Err(error);
                    }
                    // Typed binding: `pause: Duration`.
                    let asked: Duration = if error.retry_after_seconds > 0 {
                        Duration::from_secs(error.retry_after_seconds)
                    } else {
                        self.backoff * 2u32.pow(attempt - 1)
                    };
                    // Owner rule of 2026-11-03, "nothing should ever wait forever or indefinitely": a pause longer than the bound ends
                    // the call with an error that names the blocker; every file is kept.
                    if error.retry_after_seconds > 0 && asked > MAX_PAUSE {
                        // `Err(...)`: builds the success or the failure variant of a `Result`.
                        return Err(RegistryError {
                            // Macro `format!`: builds or prints text (or a list) from a template.
                            message: format!(
                                "{url} answered HTTP 429 and asks for a pause of {asked:?}, longer than the {MAX_PAUSE:?} cargo-shadow waits; every file is kept; wait that long, then run the command again"
                            ),
                            status: error.status,
                            retry_after_seconds: error.retry_after_seconds,
                        });
                    }
                    // Typed binding: `pause: Duration`.
                    let pause: Duration = asked.min(MAX_PAUSE);
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!(
                        "cargo-shadow: {url} answered HTTP 429 (attempt {attempt} of {RATE_LIMIT_ATTEMPTS}); retrying in {pause:?}"
                    );
                    sleep(pause);
                    attempt += 1;
                }
            }
        }
    }
}

/// What:
/// `fn version_number_of(record: &VersionRecord) -> &str` lends the version number of one cached release.
/// Why:
/// The cache is trusted only when it already lists every version the lockfile names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const versionNumberOf = (record: VersionRecord) => record.num;
/// ```
fn version_number_of(record: &VersionRecord) -> &str {
    return record.num.as_str();
}

/// What:
/// `impl Registry for CratesIo` makes crates.io usable as the `Registry` the checks ask for.
/// Why:
/// The release-age and trust checks only know the trait,
/// so tests use a fake with canned histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CratesIo implements Registry { ... }
/// ```
impl Registry for CratesIo {
    /// What:
    /// `fn prefetch(&self, names)` fills the history cache for the crates in `names` from the database dump when enough of them are uncached.
    /// Why:
    /// A frozen install of every lockfile asks about a thousand crates;
    /// one streaming pass replaces a thousand rate-limited requests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prefetch(names: Set<string>): void  // throws RegistryError
    /// ```
    fn prefetch(
        &self,
        names: &BTreeSet<String>,
    ) -> Result<(), RegistryError> {
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&dyn`: lends the value without copying or giving it away.
        // Typed binding: `dump: &dyn DumpSource`.
        // `dyn` trait object `dyn DumpSource`: any type that implements the trait.
        let dump: &dyn DumpSource = match &self.dump {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found.as_ref(),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            None => return Ok(()),
        };
        // Closure `|name|`: an anonymous function, like a TS arrow function.
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.filter(`: transforms or selects items with a closure.
        // Typed binding: `uncached: BTreeSet<String>`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let mut uncached: BTreeSet<String> = BTreeSet::new();
        for name in names {
            if !self.cache_dir.join(format!("{name}.json")).exists() {
                uncached.insert(name.clone());
            }
        }
        if uncached.len() < self.dump_breakeven {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!(
                "cargo-shadow: {} crates lack a cached history (below {}); using the crates.io API for them",
                uncached.len(),
                self.dump_breakeven
            );
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(());
        }
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: {} crates lack a cached history; reading {DATABASE_DUMP_URL}",
            uncached.len()
        );
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&uncached`: lends the value without copying or giving it away.
        let result =
            fill_histories_from_dump(dump.open(DATABASE_DUMP_URL)?, &uncached, &self.cache_dir)
                // `None`: the present or the absent value of an `Option`.
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Closure `|error|`: an anonymous function, like a TS arrow function.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                .map_err(registry_error_of_dump)?;
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: dump: {} found, {} written, {} already cached, {} left to the API (possible Trusted Publishing releases)",
            result.found,
            result.written,
            result.kept,
            result.left_to_api.len()
        );
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }

    /// What:
    /// `fn pubtimes(&self, name)` returns the publish time of each version of a crate from the sparse index.
    /// Why:
    /// The release-age rule compares those times with the cutoff.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pubtimes(name: string): Map<string, string>  // throws RegistryError
    /// ```
    fn pubtimes(
        &self,
        name: &str,
    ) -> Result<BTreeMap<String, String>, RegistryError> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.fetcher.get`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return parse_index_pubtimes(
            &self
                .fetcher
                .get(&format!("{SPARSE_INDEX_URL}/{}", sparse_index_path(name)))?,
        );
    }

    /// What:
    /// `fn yanked(&self, name)` returns the yanked versions of a crate from the sparse index.
    /// Why:
    /// A set-aside candidate that was yanked must not be locked in place of a refused version.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// yanked(name: string): Set<string>  // throws RegistryError
    /// ```
    fn yanked(
        &self,
        name: &str,
    ) -> Result<BTreeSet<String>, RegistryError> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.fetcher.get`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return parse_index_yanked(
            &self
                .fetcher
                .get(&format!("{SPARSE_INDEX_URL}/{}", sparse_index_path(name)))?,
        );
    }

    /// What:
    /// `fn owners(&self, name)` returns the account ids of the crate's listed individual owners from the API.
    /// Why:
    /// A first release by a listed owner counts as a known publisher,
    /// so the trust rule asks for owners only for a release it would otherwise refuse.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// owners(name: string): Set<string>  // throws RegistryError
    /// ```
    fn owners(
        &self,
        name: &str,
    ) -> Result<BTreeSet<String>, RegistryError> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.spaced_api_get`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return parse_owner_ids(&self.spaced_api_get(&format!("{API_URL}/crates/{name}/owners"))?);
    }

    /// What:
    /// `fn history(&self, name, versions)` returns every release of a crate,
    /// from the cache when it lists all of `versions`,
    /// otherwise from the API (and writes the cache).
    /// Why:
    /// The trust rule compares a version with all earlier releases,
    /// so it needs the whole list,
    /// and published history never changes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// history(name: string, versions: string[]): VersionRecord[]  // throws RegistryError
    /// ```
    fn history(
        &self,
        name: &str,
        versions: &[String],
    ) -> Result<Vec<VersionRecord>, RegistryError> {
        // Typed binding: `path: PathBuf`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let path: PathBuf = self.cache_dir.join(format!("{name}.json"));
        // Borrow `&path`: lends the value without copying or giving it away.
        // Typed binding: `cached: Vec<VersionRecord>`.
        let cached: Vec<VersionRecord> = read_cached_history(&path);
        // Closure `|record|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // `.collect()`: runs the iterator and gathers the items into the bound collection type.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `known: BTreeSet<&str>`.
        let known: BTreeSet<&str> = cached.iter().map(version_number_of).collect();
        // Closure `|version|`: an anonymous function, like a TS arrow function.
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        let mut all_known: bool = true;
        for version in versions {
            if !known.contains(version.as_str()) {
                all_known = false;
                break;
            }
        }
        if !cached.is_empty() && all_known {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(cached);
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&self.spaced_api_get`: lends the value without copying or giving it away.
        // Typed binding: `fresh: Vec<VersionRecord>`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let fresh: Vec<VersionRecord> = parse_version_history(
            &self.spaced_api_get(&format!("{API_URL}/crates/{name}/versions"))?,
        )?;
        // Borrow `&self.cache_dir`: lends the value without copying or giving it away.
        if fs::create_dir_all(&self.cache_dir).is_ok() {
            // Borrow `&path`: lends the value without copying or giving it away.
            // Macro `format!`: builds or prints text (or a list) from a template.
            // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
            // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // void expr;   // or try { expr } catch {}
            // ```
            let _ = fs::write(&path, format!("{}\n", history_to_json(&fresh)));
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(fresh);
    }
}
