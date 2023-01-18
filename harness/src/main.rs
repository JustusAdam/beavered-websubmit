extern crate clap;
use clap::Parser;

const CONFIGURATIONS : &'static [(&'static str, &'static[&'static str])] = &[
    ("del", &[
        "edit-1-1",
        "edit-1-2",
        "edit-1-3",
        "edit-1-4",
        "edit-1-4-a",
        "edit-1-4-b",
        "edit-1-5",
        "edit-1-6",
        "edit-1-7",
        "edit-1-8",
        "edit-1-9",
        "edit-1-10",
        "edit-1-11",
    ])
];

#[derive(Parser)]
struct Args {
    /// Print complete error messages
    #[clap(long)]
    verbose: bool,

    /// Version of the properties to run
    prop: String,

    #[clap(long, default_value = "..")]
    directory: std::path::PathBuf,
}

enum RunResult {
    Success,
    CompilationError,
    CheckError
}

impl std::fmt::Display for RunResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RunResult::Success => 
                formatter.pad("✅"),
            RunResult::CompilationError => 
                formatter.pad("️🚧"),
            RunResult::CheckError => 
                formatter.pad("❌")
        }
    }
}

fn run_edit(typ: &str, version: &str, edit: Option<&str>, cd: &std::path::Path, verbose: bool) -> RunResult {
    use std::process::*;
    let mut dfpp_cmd = Command::new("cargo");
    dfpp_cmd
        .current_dir(cd)
        .arg("dfpp")
        .stdin(Stdio::null());
    if let Some(edit) = edit {
        dfpp_cmd.args(&["--", "--features", edit]);
    }
    if !verbose {
        dfpp_cmd
            .stderr(Stdio::null())
            .stdout(Stdio::null());
    }
    if !dfpp_cmd.status().unwrap().success() {
        return RunResult::CompilationError;
    }

    let propfile = format!("{version}-{typ}-props.frg");
    let mut racket_cmd = Command::new("racket");
    racket_cmd
        .current_dir(cd)
        .arg(propfile)
        .stdin(Stdio::null());
    if !verbose {
        racket_cmd
            .stderr(Stdio::null())
            .stdout(Stdio::null());
    }
    if racket_cmd.status().unwrap().success() {
        RunResult::Success
    } else {
        RunResult::CheckError
    }
}

fn main() {
    let args = Args::parse();
    let head_cell_width = 12;
    let body_cell_width = 10;

    let results = CONFIGURATIONS.iter().map(|(typ, edits)| {
        (typ, edits
                .iter()
                .cloned()
                .map(Some)
                .chain([None])
                .map(|e| (e, run_edit(typ, &args.prop, e, &args.directory, args.verbose))).collect::<Vec<_>>())
    });

    for (typ, results) in results {
        println!(" {:head_cell_width$} | {:body_cell_width$}", typ, args.prop);
        println!("-{:-<head_cell_width$}-+-{:-<body_cell_width$}", "", "");
        for (edit, result) in results {
            println!("{:head_cell_width$}|{:body_cell_width$}", edit.unwrap_or(&"none"), result);
        }
    }
}
