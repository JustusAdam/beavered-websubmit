extern crate clap;
extern crate indicatif;
use clap::Parser;

use indicatif::ProgressBar;

const CONFIGURATIONS: &'static [(&'static str, &'static [&'static str])] = &[(
    "del",
    &[
        "edit-1-1",
        "edit-1-2",
        "edit-1-3",
        "edit-1-4",
        "edit-1-4-a",
        "edit-1-4-b",
        "edit-1-5",
        //"edit-1-6",
        "edit-1-7",
        "edit-1-8",
        "edit-1-9",
        "edit-1-10",
        "edit-1-11",
    ],
)];

/// Batch executor for the evaluation of our 2023 HotOS paper.
///
/// Be aware that this tool does not install dfpp itself but assumes the latest
/// version is already present and in the $PATH.
#[derive(Parser)]
struct Args {
    /// Print complete error messages for called programs on failure
    #[clap(long)]
    verbose: bool,

    /// Version of the properties to run
    prop: Vec<String>,

    /// Location of the WebSubmit repo
    #[clap(long, default_value = "..")]
    directory: std::path::PathBuf,
}

#[derive(Clone)]
enum RunResult {
    Success,
    CompilationError,
    CheckError,
}

impl std::fmt::Display for RunResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::{Alignment, Write};
        let width = formatter.width().unwrap_or(2);
        let (before, after) = match formatter.align() {
            None => (0, width - 2),
            _ if width < 2 => (0, 0),
            Some(Alignment::Left) => (0, width - 2),
            Some(Alignment::Right) => (width - 2, 0),
            Some(Alignment::Center) => {
                let left = (width - 2) / 2;
                (left, width - 2 - left)
            }
        };
        let fill_chr = formatter.fill();
        for _ in 0..before {
            formatter.write_char(fill_chr)?;
        }
        match self {
            RunResult::Success => formatter.write_str("✅"),
            RunResult::CompilationError => formatter.write_str("️🚧"),
            RunResult::CheckError => formatter.write_str("❌"),
        }?;
        for _ in 0..after {
            formatter.write_char(fill_chr)?;
        }
        Ok(())
    }
}

fn run_edit(
    typ: &str,
    versions: &[String],
    edit: Option<&'static str>,
    cd: &std::path::Path,
    verbose: bool,
    progress: &ProgressBar,
) -> Vec<RunResult> {
    progress.set_message(edit.unwrap_or("default"));
    use std::process::*;
    let mut dfpp_cmd = Command::new("cargo");
    dfpp_cmd.current_dir(cd).arg("dfpp").stdin(Stdio::null());
    if let Some(edit) = edit {
        dfpp_cmd.args(&["--", "--features", edit]);
    }
    if !verbose {
        dfpp_cmd.stderr(Stdio::null()).stdout(Stdio::null());
    }
    let status = dfpp_cmd.status().unwrap();
    progress.inc(1);
    if !status.success() {
        let handled = versions.len();
        progress.inc(handled as u64);
        return std::iter::repeat(RunResult::CompilationError)
            .take(handled)
            .collect();
    }

    versions
        .iter()
        .map(|version| {
            let propfile = format!("{version}-{typ}-props.frg");
            let mut racket_cmd = Command::new("racket");
            racket_cmd
                .current_dir(cd)
                .arg(propfile)
                .stdin(Stdio::null());
            if !verbose {
                racket_cmd.stderr(Stdio::null()).stdout(Stdio::null());
            }
            let status = racket_cmd.status().unwrap();
            progress.inc(1);
            if status.success() {
                RunResult::Success
            } else {
                RunResult::CheckError
            }
        })
        .collect()
}

fn main() {
    use std::io::Write;
    let args = Args::parse();
    let head_cell_width = 12;
    let body_cell_width = args.prop.iter().map(|e| e.len()).max().unwrap_or(2);

    let configurations = CONFIGURATIONS
        .iter()
        .map(
            |(_, inner)| inner.len() + 1, // default (no edits)
        )
        .sum::<usize>()
        * (1 // compile 
            + args.prop.len());

    let progress = ProgressBar::new(configurations as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg:10} {bar} {pos:>3}/{len:3}")
            .unwrap(),
    );

    let results = CONFIGURATIONS
        .iter()
        .map(|(typ, edits)| {
            (
                typ,
                edits
                    .iter()
                    .cloned()
                    .map(Some)
                    .chain([None])
                    .map(|e| {
                        (
                            e,
                            run_edit(typ, &args.prop, e, &args.directory, args.verbose, &progress),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    println!("{}", progress.position());
    progress.finish_and_clear();

    let ref mut w = std::io::stdout();
    (|| -> std::io::Result<()> {
        for (typ, results) in results {
            write!(w, " {:head_cell_width$} ", typ,)?;
            for version in args.prop.iter() {
                write!(w, "| {:body_cell_width$} ", version)?
            }
            writeln!(w, "")?;
            write!(w, "-{:-<head_cell_width$}-", "")?;
            for _ in args.prop.iter() {
                write!(w, "+-{:-<body_cell_width$}-", "")?
            }
            writeln!(w, "")?;
            for (edit, versions) in results {
                write!(w, " {:head_cell_width$} ", edit.unwrap_or(&"none"))?;
                for result in versions {
                    write!(w, "| {:^body_cell_width$} ", result)?;
                }
                writeln!(w, "")?;
            }
        }
        Ok(())
    })()
    .unwrap()
}
