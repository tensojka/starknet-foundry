use crate::test_case_summary::TestCaseSummary;
use crate::{CrateLocation, TestCrateSummary};
use anyhow::Error;
use console::style;

pub fn print_error_message(error: &Error) {
    let error_tag = style("ERROR").red();
    println!("[{error_tag}] {error}");
}

pub(crate) fn print_collected_tests_count(tests_num: usize, package_name: &str) {
    let plain_text = format!("\n\nCollected {tests_num} test(s) from {package_name} package");
    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_running_tests(test_crate_file: CrateLocation, tests_num: usize) {
    let dir_name = match test_crate_file {
        CrateLocation::Lib => "src",
        CrateLocation::Tests => "tests",
    };
    let plain_text = format!("Running {tests_num} test(s) from {dir_name}/");

    println!("{}", style(plain_text).bold());
}

pub(crate) fn print_test_summary(summaries: &[TestCrateSummary]) {
    let passed: usize = summaries.iter().map(TestCrateSummary::count_passed).sum();
    let skipped: usize = summaries.iter().map(TestCrateSummary::count_skipped).sum();
    let failed: usize = summaries.iter().map(TestCrateSummary::count_failed).sum();

    println!(
        "{}: {} passed, {} failed, {} skipped",
        style("Tests").bold(),
        passed,
        failed,
        skipped,
    );
}

pub(crate) fn print_test_seed(seed: u64) {
    println!("{}: {seed}", style("Fuzzer seed").bold());
}

pub(crate) fn print_test_result(test_result: &TestCaseSummary) {
    let result_header = match test_result {
        TestCaseSummary::Passed { .. } => format!("[{}]", style("PASS").green()),
        TestCaseSummary::Failed { .. } => format!("[{}]", style("FAIL").red()),
        TestCaseSummary::Ignored { .. } => format!("[{}]", style("IGNORE").yellow()),
        TestCaseSummary::Skipped { .. } => format!("[{}]", style("SKIP").color256(11)),
        TestCaseSummary::Interrupted {} => {
            unreachable!()
        }
    };

    let result_name = match test_result {
        TestCaseSummary::Skipped { name }
        | TestCaseSummary::Ignored { name }
        | TestCaseSummary::Failed { name, .. }
        | TestCaseSummary::Passed { name, .. } => name,
        TestCaseSummary::Interrupted {} => {
            unreachable!()
        }
    };

    let result_message = match test_result {
        TestCaseSummary::Passed { msg: Some(msg), .. } => format!("\n\nSuccess data:{msg}"),
        TestCaseSummary::Failed { msg: Some(msg), .. } => format!("\n\nFailure data:{msg}"),
        _ => String::new(),
    };

    let fuzzer_report = match test_result.runs() {
        None => String::new(),
        Some(runs) => {
            if matches!(test_result, TestCaseSummary::Failed { .. }) {
                let arguments = test_result.arguments();
                format!(" (fuzzer runs = {runs}, arguments = {arguments:?})")
            } else {
                format!(" (fuzzer runs = {runs})")
            }
        }
    };

    println!("{result_header} {result_name}{fuzzer_report}{result_message}");
}

pub fn print_failures(all_failed_tests: &[TestCaseSummary]) {
    if all_failed_tests.is_empty() {
        return;
    }

    let failed_tests_names: Vec<&String> = all_failed_tests
        .iter()
        .map(|test_case_summary| match test_case_summary {
            TestCaseSummary::Failed { name, .. } => name,
            TestCaseSummary::Passed { .. }
            | TestCaseSummary::Ignored { .. }
            | TestCaseSummary::Skipped { .. }
            | TestCaseSummary::Interrupted {} => unreachable!(),
        })
        .collect();

    println!("\nFailures:");
    for name in failed_tests_names {
        println!("    {name}");
    }
}
