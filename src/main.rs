use gui_experiment::SudokuModel;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let sudoku_model = SudokuModel::example();

    #[cfg(feature = "floem")]
    gui_experiment::floem::main(sudoku_model);

    #[cfg(feature = "iced")]
    gui_experiment::iced::main(sudoku_model).unwrap();

    #[cfg(feature = "slint")]
    gui_experiment::slint::main(sudoku_model).unwrap();

    #[cfg(feature = "egui")]
    gui_experiment::egui::main(sudoku_model).unwrap();

    #[cfg(feature = "gpui")]
    gui_experiment::gpui::main(sudoku_model);

    #[cfg(feature = "xilem")]
    gui_experiment::xilem::main(sudoku_model).unwrap();

    #[cfg(feature = "leptos")]
    gui_experiment::leptos::main(sudoku_model).unwrap();

    #[cfg(feature = "rui")]
    gui_experiment::rui::main(sudoku_model).unwrap();

    #[cfg(feature = "ratatui")]
    gui_experiment::ratatui::main(sudoku_model).unwrap();

    #[cfg(feature = "kas")]
    gui_experiment::kas::main(sudoku_model).unwrap();
}
