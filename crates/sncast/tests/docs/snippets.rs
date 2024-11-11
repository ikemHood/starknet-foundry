use crate::helpers::runner::runner;
use regex::Regex;
use shared::test_utils::docs_validation::{
    extract_from_directory, get_parent_dir, parse_snippet_str_to_command_args,
};
use tempfile::tempdir;
#[test]
fn test_docs_snippets() {
    let tempdir: tempfile::TempDir = tempdir().expect("Unable to create a temporary directory");
    let root_dir = get_parent_dir(2);

    let re = Regex::new(r"(?ms)```shell\n\$ sncast(.*?)").expect("Invalid regex pattern");
    let extension = Some("md");
    let snippets = extract_from_directory(&root_dir, &re, extension)
        .expect("Failed to extract sncast command snippets");

    let skipped_args = [
        // snippet "$ sncast <subcommand>"
        vec!["<subcommand>"],
        // snippet with interactive import example
        vec![
            "account",
            "import",
            "--url",
            "http://127.0.0.1:5050",
            "--name",
            "account_123",
            "--address",
            "0x1",
            "--type",
            "oz",
        ],
    ];

    for snippet in snippets.clone() {
        let args = parse_snippet_str_to_command_args(snippet.as_str());
        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        if skipped_args.contains(&args) {
            continue;
        }

        let snapbox = runner(&args).current_dir(tempdir.path());
        let output = snapbox.output().expect("Failed to execute the command");
        let exit_code = output.status.code().unwrap_or_default();
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_ne!(exit_code, 2, "The command failed. Stderr: {stderr}");
    }

    println!("count: {}", snippets.len());
}
