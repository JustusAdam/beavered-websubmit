extern crate clap;
extern crate indicatif;
use clap::Parser;

use indicatif::ProgressBar;

use std::collections::HashSet;

const CONFIGURATIONS: &'static [(&'static str, &'static [(&'static str, bool)])] = &[(
    "del",
    &[
        ("edit-1-1", false),
        ("edit-1-2", true),
        ("edit-1-3", true),
        ("edit-1-4-a", true),
        ("edit-1-4-b", true),
        ("edit-1-4-c", true),
        ("edit-1-5", false),
        //"edit-1-6",
        ("edit-1-7", false),
        ("edit-1-8", false),
        ("edit-1-9", false),
        ("edit-1-10", true),
        ("edit-1-11", true),
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

    #[clap(long)]
    only: Option<Vec<String>>,
}

#[derive(Clone)]
enum RunResult {
    Success,
    CompilationError,
    CheckError,
}

impl From<bool> for RunResult {
    fn from(b: bool) -> Self {
        if b {
            RunResult::Success
        } else {
            RunResult::CheckError
        }
    }
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
    let body_cell_width = args.prop.iter().map(|e| e.len()).max().unwrap_or(0).max(8);
    let ref is_selected = {
        let as_ref_v = args
            .only
            .as_ref()
            .map(|v| v.iter().map(String::as_str).collect::<HashSet<&str>>());
        move |s| as_ref_v.as_ref().map_or(true, |v| v.contains(s))
    };

    let num_versions = args.prop.len();

    let configurations: Vec<(_, Vec<_>)> = CONFIGURATIONS
        .iter()
        .filter_map(|(t, edits)| {
            let new_edits = edits
                .iter()
                .filter(|e| is_selected(e.0))
                .collect::<Vec<_>>();
            (!new_edits.is_empty()).then_some((t, new_edits))
        })
        .collect();

    let num_configurations = configurations
        .iter()
        .map(
            |(_, inner)| inner.len() + 1, // default (no edits)
        )
        .sum::<usize>()
        * (1 // compile 
            + num_versions);

    let progress = ProgressBar::new(num_configurations as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg:10} {bar:40} {pos:>3}/{len:3}")
            .unwrap(),
    );

    let results = configurations
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
                            run_edit(
                                typ,
                                &args.prop,
                                e.map(|e| e.0),
                                &args.directory,
                                args.verbose,
                                &progress,
                            ),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    progress.finish_and_clear();

    let ref mut w = std::io::stdout();
    (|| -> std::io::Result<()> {
        for (typ, results) in results {
            let mut false_positives = Vec::with_capacity(num_versions);
            false_positives.resize(num_versions, 0);
            let mut false_negatives = Vec::with_capacity(num_versions);
            false_negatives.resize(num_versions, 0);

            write!(w, " {:head_cell_width$} ", typ,)?;
            let exp = "expected".to_string();
            for version in [&exp].into_iter().chain(args.prop.iter()) {
                write!(w, "| {:body_cell_width$} ", version)?
            }
            writeln!(w, "")?;
            write!(w, "-{:-<head_cell_width$}-", "")?;
            for _ in 0..args.prop.len() + 1 {
                write!(w, "+-{:-<body_cell_width$}-", "")?
            }
            writeln!(w, "")?;
            for (edit, versions) in results {
                let (edit, exp_bool) = edit.cloned().unwrap_or((&"none", true));
                let expected: RunResult = exp_bool.into();
                write!(w, " {:head_cell_width$} ", edit)?;
                write!(w, "| {:^body_cell_width$} ", expected)?;
                for (i, result) in versions.into_iter().enumerate() {
                    match (&expected, &result) {
                        (RunResult::Success, RunResult::CheckError) => false_negatives[i] += 1,
                        (RunResult::CheckError, RunResult::Success) => false_positives[i] += 1,
                        _ => (),
                    };
                    write!(w, "| {:^body_cell_width$} ", result)?;
                }
                writeln!(w, "")?;
            }
            write!(w, "-{:-<head_cell_width$}-", "")?;
            for _ in 0..args.prop.len() + 1 {
                write!(w, "+-{:-<body_cell_width$}-", "")?
            }
            writeln!(w, "")?;

            write!(w, " {:head_cell_width$} ", "false pos")?;
            write!(w, "| {:^body_cell_width$} ", "-")?;
            for p in false_positives {
                write!(w, "| {:^body_cell_width$} ", p)?;
            }
            writeln!(w, "")?;
            write!(w, " {:head_cell_width$} ", "false neg")?;
            write!(w, "| {:^body_cell_width$} ", "-")?;
            for p in false_negatives {
                write!(w, "| {:^body_cell_width$} ", p)?;
            }
            writeln!(w, "")?;
        }
        Ok(())
    })()
    .unwrap()
}
