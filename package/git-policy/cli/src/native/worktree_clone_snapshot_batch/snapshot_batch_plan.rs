//! What:
//!  The requests a verified snapshot set becomes:
//! no request for an empty set,
//! one
//!  when the whole set fits the budget,
//! and otherwise as many as the largest fitting
//!  prefix of the remaining names needs.
//! Why:
//!  The design takes the largest fitting prefix repeatedly,
//! sorted by name,
//! and
//!  stops rather than retrying by name when even one tuple cannot fit
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//!  step 3).
//!  Every chunk is measured by packing the exact request it becomes,
//! so the split
//!  moves with the encoder's bytes and not with a name count.
//!  A split reopens the
//!  inspection-to-use window and can leave partial cleanup across requests:
//!  the caller
//!  reproves the remaining inventory and lineage before each chunk,
//! and this module
//!  decides only what those requests are.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planRequests(pool, names, budget);
//! plan.chunks(); // [] for an empty set, else sorted largest-fitting prefixes
//! plan.encodedSizes(); // one exact packed input size per chunk
//! ```

/// The caller's request budget.
use super::request_budget::RequestBudget;
/// The refusal a set this route will not submit reports.
use super::request_refusal::BatchRefusal;
/// The exact packed size of one candidate request.
use super::snapshot_destroy_request::encoded_request_bytes;

/// What:
///  The requests one verified set is submitted as.
/// Why:
///  The chunks are the named facts a caller iterates,
///  and the sizes are kept
///  beside them so the removal record and the run's report can say what each
///  request carried without encoding it again.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestPlan {
    /// The pool every request names.
    pool: String,
    /// The budget the split was made against.
    budget: RequestBudget,
    /// The name lists,
    /// sorted,
    /// each at most as large as the budget allows.
    chunks: Vec<Vec<String>>,
    /// The exact packed input size of each chunk's request.
    encoded_sizes: Vec<usize>,
}

/// The methods of `RequestPlan`.
impl RequestPlan {
    /// The pool every request names.
    pub fn pool(&self) -> &str {
        return &self.pool;
    }

    /// The budget the split was made against.
    pub fn budget(&self) -> RequestBudget {
        return self.budget;
    }

    /// The name lists one request each,
    /// empty when the verified set was empty.
    pub fn chunks(&self) -> &[Vec<String>] {
        return &self.chunks;
    }

    /// The exact packed input size of each chunk's request.
    pub fn encoded_sizes(&self) -> &[usize] {
        return &self.encoded_sizes;
    }

    /// Whether no request is made at all,
    /// which is what an empty set means.
    pub fn is_empty(&self) -> bool {
        return self.chunks.is_empty();
    }

    /// How many names the plan covers.
    pub fn name_count(&self) -> usize {
        return self
            .chunks
            .iter()
            .map(|chunk| {
                return chunk.len();
            })
            .sum();
    }
}

/// What:
///  Plan the requests that destroy `names` in `pool` within `budget`.
/// Why:
///  An empty set makes no request,
/// the names are sorted so a split is the same for
///  the same set,
/// and one name larger than the whole budget refuses the plan
///  before any request is sent.
///
/// # Throws
///
/// [`BatchRefusal::RepeatedName`] when the set holds one name twice,
/// [`BatchRefusal::OneNameOverTheBudget`] when a name cannot fit alone,
/// and
/// [`BatchRefusal::Unbuildable`] when the binding refuses a name.
pub fn plan_requests(
    pool: &str,
    names: &[String],
    budget: RequestBudget,
) -> Result<RequestPlan, BatchRefusal> {
    let mut ordered: Vec<String> = names.to_vec();
    ordered.sort_unstable();
    if let Some(repeated) = ordered.windows(2).find(|pair| {
        return pair[0] == pair[1];
    }) {
        return Err(BatchRefusal::RepeatedName {
            name: repeated[0].clone(),
        });
    }
    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut encoded_sizes: Vec<usize> = Vec::new();
    let mut remaining: &[String] = ordered.as_slice();
    while !remaining.is_empty() {
        let fitting: usize = largest_fitting_prefix(pool, remaining, budget)?;
        let chunk: Vec<String> = remaining[..fitting].to_vec();
        let size: usize = encoded_request_bytes(pool, chunk.as_slice())?;
        chunks.push(chunk);
        encoded_sizes.push(size);
        remaining = &remaining[fitting..];
    }
    return Ok(RequestPlan {
        pool: pool.to_string(),
        budget,
        chunks,
        encoded_sizes,
    });
}

/// What:
///  The largest prefix of `names` whose own request fits `budget`.
/// Why:
///  Each added name adds its pair's bytes,
/// so the packed size rises strictly and
///  the fitting prefixes are a contiguous prefix of the sorted names;
/// a binary search
///  therefore packs a logarithmic number of candidates per chunk.
///
/// # Throws
///
/// [`BatchRefusal::OneNameOverTheBudget`] when even the first name alone does
/// not fit,
/// and [`BatchRefusal::Unbuildable`] when the binding refuses a name.
fn largest_fitting_prefix(
    pool: &str,
    names: &[String],
    budget: RequestBudget,
) -> Result<usize, BatchRefusal> {
    if names.is_empty() {
        return Ok(0);
    }
    let first: usize = encoded_request_bytes(pool, &names[..1])?;
    if first > budget.input_bytes() {
        return Err(BatchRefusal::OneNameOverTheBudget {
            name: names[0].clone(),
            request_bytes: first,
            budget_bytes: budget.input_bytes(),
        });
    }
    let mut fitting: usize = 1;
    let mut beyond: usize = names.len();
    while fitting < beyond {
        let middle: usize = fitting + (beyond - fitting + 1) / 2;
        let size: usize = encoded_request_bytes(pool, &names[..middle])?;
        if size <= budget.input_bytes() {
            fitting = middle;
        } else {
            beyond = middle - 1;
        }
    }
    return Ok(fitting);
}

/// The controls of the sort and the split.
#[cfg(test)]
#[path = "snapshot_batch_plan_tests.rs"]
mod tests;
