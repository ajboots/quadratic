use proptest::prelude::*;
use std::collections::HashMap;

use crate::grid::*;

proptest! {
    #[test]
    fn proptest_set_and_get_cells(cells in strategies::cells_to_set()) {
        test_set_and_get_cells(&cells);
    }

    #[test]
    fn proptest_undo_redo(cell_batches in prop::array::uniform4(strategies::cells_to_set())) {
        test_undo_redo(cell_batches);
    }
}

mod strategies {
    use super::*;

    pub fn smallish_pos() -> impl Strategy<Value = Pos> {
        (-16..16_i64, -16..16_i64).prop_map(|(x, y)| Pos { x, y })
    }

    pub fn cells_to_set() -> impl Strategy<Value = Vec<(Pos, Cell)>> {
        let cell_value = any::<Option<i64>>().prop_map(|i| i.map_or(Cell::Empty, Cell::Int));
        prop::collection::vec((smallish_pos(), cell_value), 0..20)
    }
}

fn test_set_and_get_cells(cells: &[(Pos, Cell)]) {
    // Compare the grid against a hashmap for reference.
    let mut grid = Grid::default();
    let mut hashmap = HashMap::new();
    for (pos, cell) in cells {
        let old_expected = hashmap.insert(*pos, cell);
        let old_actual = grid.set_cell(*pos, cell.clone());
        assert_eq!(old_actual, *old_expected.unwrap_or(&Cell::Empty));
    }
    assert!(dbg!(&grid).is_valid());
    for (pos, cell) in hashmap {
        assert_eq!(cell, grid.get_cell(pos));
    }
}

fn test_undo_redo(cell_batches: [Vec<(Pos, Cell)>; 4]) {
    let [a, b, c, d] = cell_batches.map(|batch| Command::SetCells(batch));

    // For reference
    let mut grid = GridController::new();
    let initial = grid.clone();
    grid.execute(a);
    let grid_a = grid.clone(); // a
    grid.execute(b);
    let grid_b = grid.clone(); // a -> b
    grid.execute(c);
    let grid_c = grid.clone(); // a -> b -> c

    assert!(
        !grid.redo(),
        "redo should fail because there is nothing to redo",
    );
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_c);

    assert!(grid.undo(), "undo should succeed");
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_b);

    assert!(grid.undo(), "undo should succeed");
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_a);

    assert!(grid.undo(), "undo should succeed");
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, initial);

    assert!(
        !grid.undo(),
        "undo should fail because the stack has been exhausted",
    );
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, initial);

    assert!(grid.redo(), "redo should succeed",);
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_a);

    let mut grid_d = grid_a.clone();
    grid_d.execute(d.clone()); // a -> d

    grid.execute(d);
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_d);
    assert!(
        !grid.redo(),
        "redo should fail because new command clears the redo stack",
    );
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_d);
    dbg!(&grid_a);

    dbg!(&grid);
    assert!(grid.undo(), "undo should succeed");
    dbg!(&grid);
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_a);

    assert!(grid.undo(), "undo should succeed");
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, initial);

    assert!(grid.redo(), "redo should succeed");
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_a);

    assert!(grid.redo(), "redo should succeed");
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_d);

    assert!(
        !grid.redo(),
        "redo should fail because the stack has been exhausted",
    );
    assert!(grid.is_valid(), "{:?}", grid);
    assert_eq!(grid, grid_d);
}