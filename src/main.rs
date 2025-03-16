use gui_experiment::SudokuModel;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let sudoku_model = SudokuModel::from([
        [1, 6, 7, 8, 9, 2, 3, 4, 5],
        [4, 2, 8, 0, 0, 0, 0, 0, 0],
        [5, 9, 3, 0, 0, 0, 0, 0, 0],
        [6, 0, 0, 4, 0, 0, 0, 0, 0],
        [7, 0, 0, 0, 5, 0, 0, 0, 0],
        [8, 0, 0, 0, 0, 6, 0, 0, 0],
        [9, 0, 0, 0, 0, 0, 7, 0, 0],
        [2, 0, 0, 0, 0, 0, 0, 8, 0],
        [3, 0, 0, 0, 0, 0, 0, 0, 9],
    ]);

    #[cfg(feature = "floem")]
    gui_experiment::floem::main(sudoku_model);

    #[cfg(feature = "iced")]
    gui_experiment::iced::main(sudoku_model).unwrap();

    #[cfg(feature = "slint")]
    gui_experiment::slint::main(sudoku_model).unwrap();
}
