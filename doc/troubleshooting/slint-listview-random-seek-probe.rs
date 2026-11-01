//! Standalone driver around the exact upstream virtualization function,
//!  with scalar geometry/property adapters.

/// Cell/RefCell model property writes and row-position output without a GUI or filesystem access.
use std::cell::{Cell, RefCell};
/// Desktop Slint uses floating-point coordinates;
///  integer-coordinate embedded builds are outside this probe.
type Coord = f32;
/// Minimal adapter for the upstream logical-length accessors used by the extracted function.
#[derive(Default, Clone, Copy)]
struct LogicalLength(f32);
impl LogicalLength {
    fn new(value: f32) -> Self { Self(value) }
    fn get(self) -> f32 { self.0 }
}
/// Same state fields as v1.18.1's RepeaterLayoutState.
#[derive(Default, Debug)]
struct RepeaterLayoutState { offset: usize, cached_item_height: Coord, previous_content_y: Coord, anchor_y: Coord }
/// Same instance operations exercised by update_visible_instances.
trait RepeaterInstanceOps {
    fn len(&self) -> usize;
    fn splice(&mut self, position: usize, remove: usize, add: usize);
    fn ensure_updated(&mut self, instance: usize, row: usize) -> bool;
    fn height(&self, instance: usize) -> Option<Coord>;
    fn listview_layout(&self, instance: usize, y: &mut Coord) -> Coord;
}
/// Same geometry/property operations used by the extracted source function.
trait ListViewProperties {
    fn content_y_get(&self) -> LogicalLength;
    fn content_y_get_internal(&self) -> LogicalLength;
    fn content_y_set(&self, value: LogicalLength);
    fn content_y_has_binding(&self) -> bool;
    fn computes_content_height(&self) -> bool;
    fn content_width_set(&self, value: LogicalLength);
    fn content_height_set(&self, value: LogicalLength);
}
// Include generated verbatim source rather than rewriting its control flow in the probe.
include!("algorithm.rs");

/// Bounded row instances retain their model indices and expose chosen per-row heights.
struct Rows { heights: Vec<f32>, instances: Vec<Option<usize>>, positions: RefCell<Vec<(usize, f32, f32)>> }
impl RepeaterInstanceOps for Rows {
    fn len(&self) -> usize { self.instances.len() }
    fn splice(&mut self, position: usize, remove: usize, add: usize) {
        self.instances.splice(position..position+remove, vec![None; add]);
    }
    fn ensure_updated(&mut self, instance: usize, row: usize) -> bool {
        let changed = self.instances[instance] != Some(row);
        self.instances[instance] = Some(row);
        changed
    }
    fn height(&self, instance: usize) -> Option<Coord> { self.instances[instance].map(|row| self.heights[row]) }
    fn listview_layout(&self, instance: usize, y: &mut Coord) -> Coord {
        let row = self.instances[instance].expect("materialized row");
        let height = self.heights[row];
        self.positions.borrow_mut().push((row, *y, height));
        *y += height;
        200.0
    }
}
/// Cells behave like direct,
///  unbound geometry properties at this algorithm boundary.
#[derive(Default)]
struct Props { y: Cell<f32>, height: Cell<f32>, width: Cell<f32>, binding: bool }
impl ListViewProperties for Props {
    fn content_y_get(&self) -> LogicalLength { LogicalLength(self.y.get()) }
    fn content_y_get_internal(&self) -> LogicalLength { LogicalLength(self.y.get()) }
    fn content_y_set(&self, value: LogicalLength) { self.y.set(value.get()); }
    fn content_y_has_binding(&self) -> bool { self.binding }
    fn computes_content_height(&self) -> bool { true }
    fn content_width_set(&self, value: LogicalLength) { self.width.set(value.get()); }
    fn content_height_set(&self, value: LogicalLength) { self.height.set(value.get()); }
}
/// Exercise the function with real row materialization and output-property mutation.
fn step(rows: &mut Rows, state: &mut RepeaterLayoutState, props: &Props, viewport: f32) {
    rows.positions.borrow_mut().clear();
    let count = rows.heights.len();
    update_visible_instances(rows, state, count, props, LogicalLength(200.0), LogicalLength(viewport));
}
/// Construct independent instances for every control case.
fn fixture(heights: Vec<f32>, viewport: f32) -> (Rows, RepeaterLayoutState, Props) {
    let mut rows = Rows { heights, instances: Vec::new(), positions: RefCell::new(Vec::new()) };
    let mut state = RepeaterLayoutState::default();
    let props = Props::default();
    step(&mut rows, &mut state, &props, viewport);
    (rows, state, props)
}

#[test]
fn uniform_rows_preserve_fractional_forward_backward_and_end_seeks() {
    for height in [24.0, 48.0, 65.5] {
        for viewport in [101.0, 300.0, 628.0] {
            let (mut rows, mut state, props) = fixture(vec![height; 64], viewport);
            let maximum = height * 64.0 - viewport;
            for offset in [height * 32.0 + 13.25, maximum, height * 2.0 + 7.75] {
                props.y.set(-offset);
                step(&mut rows, &mut state, &props, viewport);
                assert!((props.y.get() + offset).abs() < 0.01,
                    "height {height}, viewport {viewport}, requested {}, got {}", -offset, props.y.get());
            }
        }
    }
}

#[test]
fn small_equal_height_case_from_issue_4463_is_a_control() {
    let (mut rows, mut state, props) = fixture(vec![800.0; 2], 300.0);
    props.y.set(-200.0);
    step(&mut rows, &mut state, &props, 300.0);
    assert_eq!(props.y.get(), -200.0);
}

#[test]
fn variable_rows_keep_contiguous_materialized_coverage() {
    for heights in [vec![24.0, 48.0, 72.0, 96.0].repeat(20), vec![90.0, 25.0, 40.0].repeat(30)] {
        let (mut rows, mut state, props) = fixture(heights, 300.0);
        for offset in [1000.25, 17.5, 2100.75, 400.25, 0.0] {
            props.y.set(-offset);
            step(&mut rows, &mut state, &props, 300.0);
            let positions = rows.positions.borrow();
            assert!(!positions.is_empty());
            for pair in positions.windows(2) {
                assert_eq!(pair[0].0 + 1, pair[1].0);
                assert!((pair[0].1 + pair[0].2 - pair[1].1).abs() < 0.01);
            }
            let first = positions.first().unwrap();
            let last = positions.last().unwrap();
            assert!(first.1 <= 0.0, "offset {offset}, first {first:?}, state {state:?}, output {}", props.y.get());
            assert!(last.1 + last.2 >= 300.0 || last.0 + 1 == rows.heights.len());
        }
    }
}

#[test]
fn explicit_positive_overscroll_binding_remains_intact() {
    let (mut rows, mut state, mut props) = fixture(vec![48.0; 60], 628.0);
    props.binding = true;
    props.y.set(12.25);
    step(&mut rows, &mut state, &props, 628.0);
    assert_eq!(props.y.get(), 12.25);
}

#[test]
fn empty_model_clears_geometry() {
    let (rows, _state, props) = fixture(Vec::new(), 300.0);
    assert!(rows.instances.is_empty());
    assert_eq!(props.y.get(), 0.0);
    assert_eq!(props.height.get(), 0.0);
}
