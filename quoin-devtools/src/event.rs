//! Trace event types and the bounded [`EventLog`] that stores them.
//!
//! The log is a ring buffer: once `capacity` events are retained, the oldest
//! are overwritten. Aggregate views ([`EventLog::summary`],
//! [`EventLog::hot_signals`]) read **incremental counters** that are updated
//! as events are pushed and evicted, so they cost one small map clone (O
//! distinct keys) instead of a scan over the retained window. The raw event
//! slice is still available in one lock acquisition via
//! [`EventLog::with_events`], which never clones the event list.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The kind of recorded event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    /// A signal was created.
    SignalCreated,
    /// A signal was written (`set` or `update`).
    SignalMutated,
    /// `request_update` was called.
    UpdateRequested,
    /// An async task was spawned.
    TaskSpawned,
    /// A running task was aborted.
    TaskAborted,
    /// A task ran to completion.
    TaskCompleted,
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl EventKind {
    /// Short human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SignalCreated => "created",
            Self::SignalMutated => "mutated",
            Self::UpdateRequested => "update",
            Self::TaskSpawned => "spawn",
            Self::TaskAborted => "abort",
            Self::TaskCompleted => "done",
        }
    }
}

/// One recorded trace event.
#[derive(Debug, Clone)]
pub struct Event {
    /// Monotonic sequence number within the log.
    pub seq: u64,
    /// When the event happened (relative to log creation).
    pub at: Instant,
    /// What happened.
    pub kind: EventKind,
    /// The instrumented scope label (e.g. `"app"`, `"search-panel"`).
    pub scope: &'static str,
    /// Rust type name of the signal payload, when applicable.
    pub type_name: Option<&'static str>,
    /// Debug snapshot of the new value, when captured via
    /// [`snapshot`](fn@snapshot) or `TracedSignal::set_dbg`.
    pub value: Option<String>,
    /// Thread that produced the event (cached per thread, cheap to clone).
    pub thread: Arc<str>,
}

impl Event {
    pub(crate) fn now(
        seq: u64,
        kind: EventKind,
        scope: &'static str,
        type_name: Option<&'static str>,
    ) -> Self {
        Self {
            seq,
            at: Instant::now(),
            kind,
            scope,
            type_name,
            value: None,
            thread: crate::record::thread_label(),
        }
    }
}

/// Bounded, thread-safe ring of [`Event`]s with aggregate views.
///
/// Each log owns its own enable flag, so different parts of an app (or
/// different tests) can record into independent logs without interfering.
#[derive(Debug)]
pub struct EventLog {
    events: Mutex<Ring>,
    seq: AtomicU64,
    started: Instant,
    enabled: AtomicBool,
}

#[derive(Debug)]
struct Ring {
    buffer: VecDeque<Event>,
    capacity: usize,
    /// Incremental per-`(scope, kind)` counts over the retained window.
    /// Updated on push and on eviction, so [`EventLog::summary`] never has
    /// to scan the buffer. Stored as a small association list: sessions have
    /// few distinct keys, and a pointer-compare scan beats hashing there.
    scope_kind_counts: Vec<((&'static str, EventKind), usize)>,
    /// Incremental mutation counts per payload type (the "hot signal" view).
    hot_counts: Vec<(&'static str, usize)>,
}

impl Ring {
    /// Accounts `event` into the incremental aggregates.
    fn count_event(&mut self, event: &Event) {
        count_in_list(&mut self.scope_kind_counts, &(event.scope, event.kind));
        if event.kind == EventKind::SignalMutated
            && let Some(name) = event.type_name
        {
            count_in_list(&mut self.hot_counts, &name);
        }
    }

    /// Removes `event` from the incremental aggregates (ring eviction).
    ///
    /// Keys whose count reaches zero are dropped entirely, so the aggregate
    /// views only ever report keys with retained events (matching what a
    /// fresh scan of the buffer would produce).
    fn uncount_event(&mut self, event: &Event) {
        uncount_in_list(&mut self.scope_kind_counts, &(event.scope, event.kind));
        if event.kind == EventKind::SignalMutated
            && let Some(name) = event.type_name
        {
            uncount_in_list(&mut self.hot_counts, &name);
        }
    }
}

/// Adds one to the entry for `key`, appending the key on first sight.
fn count_in_list<K: PartialEq + Copy>(list: &mut Vec<(K, usize)>, key: &K) {
    match list.iter_mut().find(|(existing, _)| existing == key) {
        Some((_, count)) => *count += 1,
        None => list.push((*key, 1)),
    }
}

/// Removes one from the entry for `key`, dropping the entry entirely when
/// its count reaches zero (aggregate views only report retained keys).
fn uncount_in_list<K: PartialEq>(list: &mut Vec<(K, usize)>, key: &K) {
    if let Some(index) = list.iter().position(|(existing, _)| existing == key) {
        if list[index].1 <= 1 {
            list.swap_remove(index);
        } else {
            list[index].1 -= 1;
        }
    }
}

impl EventLog {
    /// Creates a log holding up to `capacity` events (oldest dropped first).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Mutex::new(Ring {
                buffer: VecDeque::with_capacity(capacity.min(1024)),
                capacity,
                scope_kind_counts: Vec::new(),
                hot_counts: Vec::new(),
            }),
            seq: AtomicU64::new(0),
            started: Instant::now(),
            enabled: AtomicBool::new(false),
        }
    }

    /// Whether this log records events.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Turns recording on or off for this log.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// The maximum number of retained events.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (a prior panic while
    /// recording or inspecting the log).
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.events.lock().unwrap().capacity
    }

    /// Records `event`, assigning the next sequence number. Internal.
    pub(crate) fn push(&self, mut event: Event) {
        event.seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let mut ring = self.events.lock().unwrap();
        if ring.buffer.len() >= ring.capacity
            && let Some(evicted) = ring.buffer.pop_front()
        {
            ring.uncount_event(&evicted);
        }
        ring.count_event(&event);
        ring.buffer.push_back(event);
    }

    /// The sequence number the next recorded event will carry.
    ///
    /// Useful to detect that events were recorded (it increases) even after
    /// a [`clear`](Self::clear).
    #[must_use]
    pub fn next_seq(&self) -> u64 {
        self.seq.load(Ordering::Relaxed)
    }

    /// Runs `f` with a locked snapshot of the retained events, oldest first.
    ///
    /// This is the single-lock primitive every aggregate view builds on:
    /// inspecting the log never clones the event list unless the caller asks
    /// for owned copies. The lock is held for the duration of `f`, so keep
    /// `f` short and non-reentrant.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (a prior panic while
    /// recording or inspecting the log).
    pub fn with_events<R>(&self, f: impl FnOnce(&[Event]) -> R) -> R {
        let mut ring = self.events.lock().unwrap();
        f(ring.buffer.make_contiguous())
    }

    /// Snapshot of the currently retained events (oldest first).
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn events(&self) -> Vec<Event> {
        self.with_events(<[Event]>::to_vec)
    }

    /// Number of retained events.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.lock().unwrap().buffer.len()
    }

    /// Whether no events are retained.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Drops all retained events (sequence numbers keep increasing).
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn clear(&self) {
        let mut ring = self.events.lock().unwrap();
        ring.buffer.clear();
        ring.scope_kind_counts.clear();
        ring.hot_counts.clear();
    }

    /// Per-`(scope, kind)` event counts over the retained window.
    ///
    /// O(distinct keys): the counts are maintained incrementally as events
    /// are pushed and evicted, so this is a small map clone under one lock,
    /// not a scan of the retained events.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn summary(&self) -> BTreeMap<(&'static str, EventKind), usize> {
        self.events
            .lock()
            .unwrap()
            .scope_kind_counts
            .iter()
            .copied()
            .collect()
    }

    /// Per-signal-type mutation counts (the "hot signal" view), hottest
    /// first. Ties are broken alphabetically by type name.
    ///
    /// Like [`summary`](Self::summary), this reads the incrementally
    /// maintained counters: O(distinct type names), no event scan.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    #[must_use]
    pub fn hot_signals(&self) -> Vec<(&'static str, usize)> {
        let mut v: Vec<(&'static str, usize)> = self.events.lock().unwrap().hot_counts.clone();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        v
    }

    /// Wall-clock age of the log.
    #[must_use]
    pub fn uptime(&self) -> Duration {
        self.started.elapsed()
    }
}

impl EventLog {
    /// Prints a compact human-readable table to stderr.
    ///
    /// One pass over the retained events: per-`(scope, kind)` counts plus the
    /// hot-signal ranking.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn dump(&self) {
        let retained = self.len();
        eprintln!("== quoin-devtools ({retained} events retained) ==");
        for ((scope, kind), count) in &self.summary() {
            eprintln!("  {scope:<14} {kind:<9} {count:>8}");
        }
        let hot = self.hot_signals();
        if !hot.is_empty() {
            eprintln!("hot signals (mutations by payload type):");
            for (name, count) in hot.iter().take(10) {
                eprintln!("  {name:<34} {count:>8}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn enabled_log(capacity: usize) -> EventLog {
        let log = EventLog::with_capacity(capacity);
        log.set_enabled(true);
        log
    }

    #[test]
    fn event_kind_labels_and_display() {
        assert_eq!(EventKind::SignalCreated.label(), "created");
        assert_eq!(EventKind::SignalMutated.label(), "mutated");
        assert_eq!(EventKind::UpdateRequested.label(), "update");
        assert_eq!(EventKind::TaskSpawned.label(), "spawn");
        assert_eq!(EventKind::TaskAborted.label(), "abort");
        assert_eq!(EventKind::TaskCompleted.label(), "done");
        assert_eq!(EventKind::TaskSpawned.to_string(), "spawn");
        // Ordered enums keep the lifecycle order stable for summaries.
        assert!(EventKind::SignalCreated < EventKind::SignalMutated);
    }

    #[test]
    fn ring_evicts_oldest_beyond_capacity() {
        let log = enabled_log(4);
        for _ in 0..10 {
            log.push(Event::now(0, EventKind::UpdateRequested, "ring", None));
        }
        assert_eq!(log.len(), 4);
        assert_eq!(log.capacity(), 4);
        let events = log.events();
        // The 10 events consumed seq numbers 0..10; the retained ones must be
        // the newest four (6, 7, 8, 9).
        let seqs: Vec<u64> = events
            .iter()
            .map(std::clone::Clone::clone)
            .map(|e| e.seq)
            .collect();
        assert_eq!(seqs, vec![6, 7, 8, 9]);
    }

    #[test]
    fn clear_drops_events_but_keeps_counting() {
        let log = enabled_log(8);
        let first_seq = log.next_seq();
        log.push(Event::now(0, EventKind::UpdateRequested, "clear", None));
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.next_seq(), first_seq + 1, "seq keeps increasing");
        log.push(Event::now(0, EventKind::TaskSpawned, "clear", None));
        let events = log.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].seq, first_seq + 1);
    }

    #[test]
    fn summary_groups_by_scope_and_kind() {
        let log = enabled_log(16);
        for kind in [
            EventKind::UpdateRequested,
            EventKind::UpdateRequested,
            EventKind::TaskSpawned,
        ] {
            log.push(Event::now(0, kind, "summary", None));
        }
        let summary = log.summary();
        assert_eq!(
            summary.get(&("summary", EventKind::UpdateRequested)),
            Some(&2)
        );
        assert_eq!(summary.get(&("summary", EventKind::TaskSpawned)), Some(&1));
    }

    #[test]
    fn hot_signals_rank_and_tiebreak() {
        let log = enabled_log(16);
        for _ in 0..3 {
            log.push(Event::now(0, EventKind::SignalMutated, "hot", Some("u32")));
        }
        for _ in 0..3 {
            log.push(Event::now(0, EventKind::SignalMutated, "hot", Some("i64")));
        }
        log.push(Event::now(
            0,
            EventKind::SignalMutated,
            "hot",
            Some("String"),
        ));
        let hot = log.hot_signals();
        assert_eq!(hot.len(), 3);
        assert_eq!(hot[0], ("i64", 3), "tie broken alphabetically");
        assert_eq!(hot[1], ("u32", 3));
        assert_eq!(hot[2], ("String", 1));
    }

    #[test]
    fn hot_signals_ignores_non_mutations_and_untyped() {
        let log = enabled_log(8);
        log.push(Event::now(0, EventKind::UpdateRequested, "x", None));
        log.push(Event::now(0, EventKind::SignalMutated, "x", None));
        assert_eq!(log.hot_signals(), [] as [(&str, usize); 0]);
    }

    #[test]
    fn disabled_log_never_pushes_through_record() {
        let log = EventLog::with_capacity(8);
        crate::record(&log, EventKind::UpdateRequested, "off", None);
        assert!(log.is_empty());
    }

    #[test]
    fn concurrent_recording_preserves_all_events() {
        let log: &'static EventLog = Box::leak(Box::new(EventLog::with_capacity(512)));
        log.set_enabled(true);
        let scopes: Vec<&'static str> = (0..8)
            .map(|i| Box::leak(format!("thread-{i}").into_boxed_str()) as &'static str)
            .collect();
        let handles: Vec<_> = scopes
            .iter()
            .copied()
            .map(|scope| {
                thread::spawn(move || {
                    for _ in 0..32 {
                        crate::record(log, EventKind::TaskSpawned, scope, None);
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("worker thread must not panic");
        }
        assert_eq!(log.len(), 8 * 32);
        let summary = log.summary();
        for scope in &scopes {
            assert_eq!(summary.get(&(*scope, EventKind::TaskSpawned)), Some(&32));
        }
    }

    #[test]
    fn with_events_sees_ring_in_order() {
        let log = enabled_log(3);
        for i in 0..5u64 {
            let mut event = Event::now(0, EventKind::UpdateRequested, "order", None);
            event.seq = i;
            log.push(event);
        }
        log.with_events(|events| {
            let seqs: Vec<u64> = events.iter().map(|e| e.seq).collect();
            assert_eq!(seqs, vec![2, 3, 4]);
        });
    }

    #[test]
    fn uptime_is_monotonic_and_positive() {
        let log = enabled_log(2);
        thread::sleep(Duration::from_millis(2));
        assert!(log.uptime() >= Duration::from_millis(1));
    }

    /// Regression: the incremental aggregate counters must stay consistent
    /// with the retained events even after the ring overflows and evicts.
    #[test]
    fn summary_matches_derived_counts_after_ring_overflow() {
        let log = enabled_log(4);
        for i in 0..20u32 {
            let kind = match i % 3 {
                0 => EventKind::UpdateRequested,
                1 => EventKind::TaskSpawned,
                _ => EventKind::SignalMutated,
            };
            let scope = if i % 2 == 0 { "even" } else { "odd" };
            log.push(Event::now(0, kind, scope, None));
        }
        // Derive the expected counts from the retained window (newest 4).
        let retained = log.events();
        let mut expected = std::collections::BTreeMap::new();
        for event in &retained {
            *expected.entry((event.scope, event.kind)).or_insert(0usize) += 1;
        }
        assert_eq!(log.summary(), expected, "counters must track evictions");
        assert_eq!(retained.len(), 4);
    }

    /// The hot-signal counters must also survive ring overflow: evicted
    /// mutations leave the ranking, retained ones stay.
    #[test]
    fn hot_signals_match_derived_counts_after_ring_overflow() {
        let log = enabled_log(3);
        for i in 0..12u32 {
            let name = match i % 4 {
                0 => "u32",
                1 => "i64",
                2 => "String",
                _ => "bool",
            };
            log.push(Event::now(0, EventKind::SignalMutated, "hot", Some(name)));
        }
        let retained = log.events();
        let mut expected = std::collections::BTreeMap::new();
        for event in &retained {
            if let Some(name) = event.type_name {
                *expected.entry(name).or_insert(0usize) += 1;
            }
        }
        let hot = log.hot_signals();
        let derived: Vec<(&str, usize)> = {
            let mut v: Vec<(&'static str, usize)> = expected.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
            v
        };
        assert_eq!(hot, derived);
        assert_eq!(hot.len(), 3);
    }

    #[test]
    fn clear_resets_incremental_aggregates() {
        let log = enabled_log(8);
        log.push(Event::now(
            0,
            EventKind::SignalMutated,
            "clear",
            Some("u32"),
        ));
        log.push(Event::now(0, EventKind::TaskSpawned, "clear", None));
        log.clear();
        assert!(log.is_empty());
        assert!(log.summary().is_empty(), "summary must reset with the ring");
        assert!(log.hot_signals().is_empty(), "hot view must reset too");
        // Recording continues normally afterwards.
        log.push(Event::now(0, EventKind::UpdateRequested, "clear", None));
        assert_eq!(
            log.summary().get(&("clear", EventKind::UpdateRequested)),
            Some(&1)
        );
    }

    /// Evicting a mutation must remove it from the hot ranking (the
    /// saturating decrement path).
    #[test]
    fn eviction_decrements_hot_counts() {
        let log = enabled_log(3);
        log.push(Event::now(0, EventKind::SignalMutated, "e", Some("u32")));
        log.push(Event::now(0, EventKind::SignalMutated, "e", Some("u32")));
        log.push(Event::now(0, EventKind::UpdateRequested, "e", None));
        assert_eq!(log.hot_signals(), vec![("u32", 2)]);
        // This push evicts the oldest u32 mutation.
        log.push(Event::now(0, EventKind::SignalMutated, "e", Some("i64")));
        // One u32 mutation remains retained; the new i64 joins the ranking.
        // Equal counts are ordered alphabetically (i64 < u32).
        assert_eq!(log.hot_signals(), vec![("i64", 1), ("u32", 1)]);
    }

    /// Model-based property test: after an arbitrary sequence of pushes and
    /// clears on a bounded ring, the incremental aggregate counters must be
    /// exactly what a fresh scan of the retained events would produce.
    /// Deterministic LCG so failures are reproducible without a rand dep.
    #[test]
    fn incremental_counters_match_scan_model_under_random_ops() {
        let kinds = [
            EventKind::SignalCreated,
            EventKind::SignalMutated,
            EventKind::UpdateRequested,
            EventKind::TaskSpawned,
        ];
        let scopes = ["alpha", "beta"];
        let type_names: [Option<&'static str>; 3] = [Some("u32"), Some("Vec<u8>"), None];

        // Deterministic xorshift RNG (no rand dependency) plus a narrowing-
        // safe index helper: the modulo result is always < list length, so
        // the `u64 -> usize` conversion cannot truncate in practice.
        let mut rng: u64 = 0x2545_F491_4F6C_DD1D;
        // Deterministic xorshift step, inlined at each use site (a closure
        // capturing `rng` mutably cannot be shared with another closure).
        macro_rules! next {
            () => {{
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                rng
            }};
        }
        // The modulo result is always < `len`, so the `u64 -> usize`
        // conversion cannot truncate in practice. The length is passed as a
        // `u64` literal context to avoid any narrowing cast.
        macro_rules! index_of {
            ($len:literal) => {
                usize::try_from(next!() % $len as u64).expect("index fits usize")
            };
        }

        for capacity in [1usize, 2, 7, 64] {
            let log = enabled_log(capacity);
            for step in 0..600u64 {
                if next!() % 32 == 0 {
                    log.clear();
                } else {
                    let kind = kinds[index_of!(4)];
                    let scope = scopes[index_of!(2)];
                    let type_name = type_names[index_of!(3)];
                    log.push(Event::now(0, kind, scope, type_name));
                }
                // After every step the views must agree with a fresh scan.
                if step % 25 == 0 || step > 550 {
                    let retained = log.events();
                    let mut expected = std::collections::BTreeMap::new();
                    let mut expected_hot = std::collections::BTreeMap::new();
                    for event in &retained {
                        *expected.entry((event.scope, event.kind)).or_insert(0usize) += 1;
                        if event.kind == EventKind::SignalMutated
                            && let Some(name) = event.type_name
                        {
                            *expected_hot.entry(name).or_insert(0usize) += 1;
                        }
                    }
                    assert_eq!(
                        log.summary(),
                        expected,
                        "summary drifted at step {step} (capacity {capacity})"
                    );
                    let hot = log.hot_signals();
                    assert_eq!(
                        hot.len(),
                        expected_hot.len(),
                        "hot view drifted at step {step} (capacity {capacity})"
                    );
                    for (name, count) in &hot {
                        assert_eq!(
                            expected_hot.get(name),
                            Some(count),
                            "hot count for {name} drifted at step {step}"
                        );
                    }
                }
            }
        }
    }
}
