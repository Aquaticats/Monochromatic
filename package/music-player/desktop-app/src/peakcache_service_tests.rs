// What:     Unit tests for `peakcache_service.rs`, pulled in by
//           `#[cfg(test)] #[path = "peakcache_service_tests.rs"] mod tests;` at the bottom of that file.
//           `use super::*` reaches the actor's private items because this module is its child.
// Why:      The write drain decides how many rows share one commit; it is tested here without a database.

// What:     `use super::*;`. Bring the actor module's items (`drain_writes`, `Write`, `Upsert`) into scope.
// Why:      The tests call the drain directly.
use super::*;
// What:     `use truepeak_core::DecisionKind;`. The decision tag.
// Why:      The fixture builds a `Decision`.
use truepeak_core::DecisionKind;

// What:     `fn put(fingerprint: u64) -> Write`. One queued write for the given key.
// Why:      Each test fills a queue with distinct, recognisable keys.
fn put(fingerprint: u64) -> Write {
    let decision = Decision { gain: 0.5, kind: DecisionKind::FullScanExact, measured_peak: 1.2, duration_secs: 200.0 };
    return Write::Put(Upsert { fingerprint, decision })
}

// What:     `fn keys(batch: &[Upsert]) -> Vec<u64>`. The fingerprints of a batch, in order.
// Why:      Assertions compare order and membership by key.
fn keys(batch: &[Upsert]) -> Vec<u64> {
    return batch.iter().map(|upsert| return upsert.fingerprint).collect()
}

// A lone write with nothing queued behind it is a batch of one: the drain never waits for more.
#[test]
fn a_lone_write_is_a_batch_of_one() {
    let (_write_tx, mut write_rx) = mpsc::unbounded_channel::<Write>();
    let batch = drain_writes(put(7), &mut write_rx);
    assert_eq!(keys(&batch), vec![7]);
}

// Writes already queued join the batch in arrival order, after the one just received.
#[test]
fn queued_writes_join_the_batch_in_order() {
    let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Write>();
    for key in 2..=5 {
        assert!(write_tx.send(put(key)).is_ok(), "queue should accept a write");
    }
    let batch = drain_writes(put(1), &mut write_rx);
    assert_eq!(keys(&batch), vec![1, 2, 3, 4, 5]);
    assert!(write_rx.try_recv().is_err(), "the drain should leave the queue empty");
}

// A backlog larger than the limit is split: the first batch stops at the limit and the rest stays queued.
#[test]
fn a_backlog_is_cut_at_the_batch_limit() {
    let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Write>();
    let backlog = u64::try_from(WRITE_BATCH_LIMIT).expect("limit fits u64") + 10;
    for key in 1..=backlog {
        assert!(write_tx.send(put(key)).is_ok(), "queue should accept a write");
    }
    let first = write_rx.try_recv().expect("positive control: the backlog is queued");
    let batch = drain_writes(first, &mut write_rx);
    assert_eq!(batch.len(), WRITE_BATCH_LIMIT);
    assert_eq!(batch.first().map(|upsert| return upsert.fingerprint), Some(1));
    let next = write_rx.try_recv().expect("the rows past the limit should stay queued");
    let rest = drain_writes(next, &mut write_rx);
    assert_eq!(rest.len(), 10);
    assert_eq!(rest.last().map(|upsert| return upsert.fingerprint), Some(backlog));
}

// What:     A real actor over a real database file: more writes than one batch holds all become visible, and the
//           exact-decision snapshot a read returns matches what was sent.
// Why:      `drain_writes` is tested above without a database; this shows the actor wiring (drain, one
//           `put_batch` per batch, read-biased loop) stores every row.
#[tokio::test]
async fn the_actor_stores_a_backlog_larger_than_one_batch() {
    let rows = u64::try_from(WRITE_BATCH_LIMIT).expect("limit fits u64") * 2 + 37;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock is after the epoch")
        .as_nanos();
    let dir = crate::test_scratch::scratch_root().join(format!("mp_peakcache_batch_{nanos}"));
    std::fs::create_dir_all(&dir).expect("scratch dir should be created");
    let (read_tx, write_tx) = spawn(Some(dir.join("decisions.db")));
    for fingerprint in 1..=rows {
        assert!(write_tx.send(put(fingerprint)).is_ok(), "the actor should accept a write");
    }
    let mut seen = HashSet::new();
    for _ in 0..20_000 {
        let (reply, answer) = oneshot::channel();
        assert!(read_tx.send(Read::Known { reply }).is_ok(), "the actor should accept a read");
        seen = answer.await.expect("the actor should answer a snapshot read");
        if u64::try_from(seen.len()).expect("length fits u64") == rows {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let expected: HashSet<u64> = (1..=rows).collect();
    assert_eq!(seen, expected);
    drop(read_tx);
    drop(write_tx);
    let _ = std::fs::remove_dir_all(&dir);
}
