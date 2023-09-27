use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::diagnostics::DiagnosticsReporter;
use cairo_lang_compiler::project::{check_compiler_path, setup_project};
use cairo_lang_filesystem::db::init_dev_corelib;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_runner::{SierraCasmRunner, StarknetState};

use cairo_lang_diagnostics::ToOption;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::{DebugReplacer, SierraIdReplacer};
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use camino::Utf8PathBuf;
use clap::command;
use clap::Args;

#[derive(Args)]
#[command(about = "")]
pub struct Script {
    /// Path to the script
    pub script_path: Utf8PathBuf,
}

pub fn run(script_path: Utf8PathBuf) -> Result<()> {
    check_compiler_path(true, Path::new(&script_path))?;

    // let db = &mut RootDatabase::builder().detect_corelib().build()?;
    let db = &mut RootDatabase::builder().build()?;
    let corelib_path = PathBuf::from("/Users/kamiljankowski/Documents/GitHub/cairo/corelib/src/");
    init_dev_corelib(db, corelib_path);

    let main_crate_ids = setup_project(db, Path::new(&script_path))?;
    if DiagnosticsReporter::stderr().check(db) {
        anyhow::bail!("failed to compile: {}", script_path);
    }

    let sierra_program = db
        .get_sierra_program(main_crate_ids.clone())
        .to_option()
        .with_context(|| "Compilation failed without any diagnostics.")?;
    let replacer = DebugReplacer { db };

    let runner = SierraCasmRunner::new(
        replacer.apply(&sierra_program),
        None,
        OrderedHashMap::default(),
    )
    .with_context(|| "Failed setting up runner.")?;

    let result = runner
        .run_function_with_starknet_context(
            runner.find_function("::main")?,
            &[],
            None,
            StarknetState::default(),
        )
        .with_context(|| "Failed to run the function.")?;

    match result.value {
        cairo_lang_runner::RunResultValue::Success(values) => {
            println!("Run completed successfully, returning {values:?}")
        }
        cairo_lang_runner::RunResultValue::Panic(values) => {
            print!("Run panicked with [");
            for value in &values {
                match as_cairo_short_string(value) {
                    Some(as_string) => print!("{value} ('{as_string}'), "),
                    None => print!("{value}, "),
                }
            }
            println!("].")
        }
    }
    Ok(())
}
