mod io_interpreter;

use nickel_lang_core::{
    error::NullReporter,
    eval::cache::CacheImpl,
    program::Program,
};
use std::path::PathBuf;

fn main() {
    let ncl_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: nickel-io <file.ncl>");
        std::process::exit(1);
    });

    // Resolve the ncl file path
    let ncl_path = PathBuf::from(&ncl_path)
        .canonicalize()
        .unwrap_or_else(|e| {
            eprintln!("Cannot resolve path '{ncl_path}': {e}");
            std::process::exit(1);
        });

    // Create program
    let mut program: Program<CacheImpl> = Program::new_from_file(
        &ncl_path,
        std::io::stderr(),
        NullReporter {},
    )
    .unwrap_or_else(|e| {
        eprintln!("Cannot load '{ncl_path:?}': {e}");
        std::process::exit(1);
    });

    // Add ncl/ directory to import paths so `import "io.ncl"` works
    let ncl_lib_dir = ncl_path.parent().unwrap().to_path_buf();
    program.add_import_paths([ncl_lib_dir].into_iter());

    // Evaluate the program to get the IO action tree
    let io_tree = match program.eval() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Evaluation error: {e:?}");
            std::process::exit(1);
        }
    };

    // Run the IO interpreter
    if let Err(e) = io_interpreter::interpret_io(&mut program, io_tree) {
        eprintln!("IO error: {e}");
        std::process::exit(1);
    }
}
