extern crate clap;
extern crate indicatif;
use clap::Parser;

use indicatif::ProgressBar;

use std::collections::HashSet;
use std::fmt::{Display, Write};
use std::str::FromStr;

const CONFIGURATIONS: &'static [(Property, usize)] =
    &[(Property::Deletion, 3), (Property::Storage, 1)];

const ALL_KNOWN_VARIANTS: &'static [&'static str] = &["lib", "baseline", "strict"];

/// Batch executor for the evaluation of our 2023 HotOS paper.
///
/// Be aware that this tool does not install dfpp itself but assumes the latest
/// version is already present and in the $PATH.
#[derive(Parser)]
struct Args {
    /// Print complete error messages for called programs on failure (implies
    /// `--verbose-commands`)
    #[clap(long)]
    verbose: bool,

    /// Print the shell commands we are running
    #[clap(long)]
    verbose_commands: bool,

    /// Version of the properties to run
    property_versions: Vec<String>,

    /// Location of the WebSubmit repo
    #[clap(long, default_value = "..")]
    directory: std::path::PathBuf,

    /// Only run the specified edits. Uses the same format as printing edits,
    /// aka `edit-<property>-<articulation point>-<short edit type>`, e.g. `edit-del-2-a`
    #[clap(long)]
    only: Option<Vec<Edit>>,

    /// Only run these properties (similar to --only but selects edits for a
    /// whole property)
    #[clap(long)]
    only_property: Option<Vec<Property>>,
}

impl Args {
    fn verbose_commands(&self) -> bool {
        self.verbose || self.verbose_commands
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum Property {
    Deletion,
    Storage,
    Disclosure,
}

impl Display for Property {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(match self {
            Property::Deletion => "del",
            Property::Storage => "sc",
            Property::Disclosure => "dis",
        })
    }
}

impl FromStr for Property {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "del" => Ok(Property::Deletion),
            "sc" => Ok(Property::Storage),
            "dis" => Ok(Property::Disclosure),
            _ => Err(format!("Unknown property type {s}")),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum Severity {
    Benign,
    Bug,
    Intentional,
}

impl Severity {
    fn expected_result(self) -> RunResult {
        match self {
            Severity::Benign => RunResult::Success,
            Severity::Bug | Severity::Intentional => RunResult::CheckError,
        }
    }
}

impl Display for Severity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(match self {
            Severity::Benign => "a",
            Severity::Bug => "b",
            Severity::Intentional => "c",
        })
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(Severity::Benign),
            "b" => Ok(Severity::Bug),
            "c" => Ok(Severity::Intentional),
            _ => Err(format!("Unrecognized severity type {s}")),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Edit {
    property: Property,
    articulation_point: usize,
    severity: Severity,
}

impl FromStr for Edit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.split('-');
        let split_err = "Wrong number of '-' splits";
        if it.next() != Some("edit") {
            return Err("Odd start sequence".to_string());
        }
        let property = it.next().ok_or(split_err)?.parse()?;
        let articulation_point = it
            .next()
            .ok_or(split_err)?
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        let severity = it.next().ok_or(split_err)?.parse()?;
        Ok(Edit {
            property,
            articulation_point,
            severity,
        })
    }
}

impl Display for Edit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("edit-")?;
        self.property.fmt(formatter)?;
        formatter.write_char('-')?;
        self.articulation_point.fmt(formatter)?;
        formatter.write_char('-')?;
        self.severity.fmt(formatter)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
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
        use std::fmt::Alignment;
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
    typ: Property,
    versions: &[String],
    edit: Option<&Edit>,
    cd: &std::path::Path,
    verbose: bool,
    verbose_commands: bool,
    progress: &ProgressBar,
) -> Vec<RunResult> {
    progress.set_message(edit.map_or("default".to_string(), |e| e.to_string()));
    use std::process::*;

    versions
        .iter()
        .map(|version| {
            let mut dfpp_cmd = Command::new("cargo");
            dfpp_cmd.current_dir(cd).arg("dfpp").stdin(Stdio::null());
            let external_ann_file: std::path::PathBuf =
                format!("{version}-external-annotations.json").into();
            if external_ann_file.exists() {
                dfpp_cmd.args(&[
                    "--external-annotations",
                    external_ann_file.to_str().unwrap(),
                ]);
            }
            dfpp_cmd.args(&["--", "--features", &format!("v-ann-{version}")]);
            if let Some(edit) = edit {
                dfpp_cmd.args(&["--features", &edit.to_string()]);
            }
            if !verbose {
                dfpp_cmd.stderr(Stdio::null()).stdout(Stdio::null());
            }
            if verbose_commands {
                progress.suspend(|| println!("Executing compile command: {:?}", dfpp_cmd));
            }
            let status = dfpp_cmd.status().unwrap();
            progress.inc(1);
            if !status.success() {
                progress.inc(1);
                return RunResult::CompilationError;
            }

            let propfile = format!("{version}-{typ}-props.frg");
            let mut racket_cmd = Command::new("racket");
            racket_cmd
                .current_dir(cd)
                .arg(propfile)
                .stdin(Stdio::null());
            if !verbose {
                racket_cmd.stderr(Stdio::null()).stdout(Stdio::null());
            }
            if verbose_commands {
                progress.suspend(|| println!("Executing check command: {:?}", racket_cmd));
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
    let args = {
        let mut args = Args::parse();
        if args.property_versions.is_empty() {
            println!("INFO: No specification variants to run given, running all known ones");
            args.property_versions = ALL_KNOWN_VARIANTS
                .iter()
                .cloned()
                .map(str::to_string)
                .collect();
        }
        args
    };

    let head_cell_width = 12;
    let body_cell_width = 8;
    let ref is_selected = {
        let as_ref_v = args
            .only
            .as_ref()
            .map(|v| v.iter().cloned().collect::<HashSet<Edit>>());
        move |s: &Edit| as_ref_v.as_ref().map_or(true, |v| v.contains(s))
    };

    let num_versions = args.property_versions.len();

    let configurations: Vec<(_, Vec<_>)> = CONFIGURATIONS
        .iter()
        .filter(|conf| {
            args.only_property
                .as_ref()
                .map_or(true, |p| p.contains(&conf.0))
        })
        .flat_map(|&(property, num_edits)| {
            assert!(num_edits > 0);
            let new_edits = (1..=num_edits)
                .flat_map(|articulation_point| {
                    [Severity::Benign, Severity::Bug, Severity::Intentional]
                        .into_iter()
                        .map(move |severity| Edit {
                            severity,
                            articulation_point,
                            property,
                        })
                        .filter(|e| is_selected(e))
                })
                .collect::<Vec<_>>();
            (!new_edits.is_empty()).then_some((property, new_edits))
        })
        .collect();

    let num_configurations = configurations
        .iter()
        .map(
            |(_, inner)| inner.len() + 1, // default (no edits)
        )
        .sum::<usize>()
        * (2 // compile 
            * num_versions);

    let progress = ProgressBar::new(num_configurations as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg:11} {bar:40} {pos:>3}/{len:3}")
            .unwrap(),
    );

    let results = configurations
        .iter()
        .map(|(typ, edits)| {
            (
                *typ,
                edits
                    .iter()
                    .map(Some)
                    .chain([None])
                    .map(|e| {
                        (
                            e,
                            run_edit(
                                *typ,
                                args.property_versions.as_slice(),
                                e,
                                &args.directory,
                                args.verbose,
                                args.verbose_commands(),
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
            let mut false_negatives = Vec::with_capacity(num_versions);
            false_negatives.resize(num_versions, 0);
            let mut false_positives = Vec::with_capacity(num_versions);
            false_positives.resize(num_versions, 0);

            write!(w, " {:head_cell_width$} ", typ.to_string(),)?;
            write!(w, "| {:body_cell_width$} ", "expected")?;
            for version in args.property_versions.iter() {
                write!(w, "| {:body_cell_width$} ", version)?
            }
            writeln!(w, "")?;
            write!(w, "-{:-<head_cell_width$}-", "")?;
            for _ in 0..args.property_versions.len() + 1 {
                write!(w, "+-{:-<body_cell_width$}-", "")?
            }
            writeln!(w, "")?;
            for (edit, versions) in results {
                let (edit, expected) = edit.map_or(("none".to_string(), RunResult::Success), |e| {
                    (e.to_string(), e.severity.expected_result())
                });
                write!(w, " {:head_cell_width$} ", edit)?;
                write!(w, "| {:^body_cell_width$} ", expected)?;
                for (i, result) in versions.into_iter().enumerate() {
                    match (&expected, &result) {
                        (RunResult::Success, RunResult::CheckError) => false_positives[i] += 1,
                        (RunResult::CheckError, RunResult::Success) => false_negatives[i] += 1,
                        _ => (),
                    };
                    write!(w, "| {:^body_cell_width$} ", result)?;
                }
                writeln!(w, "")?;
            }
            write!(w, "-{:-<head_cell_width$}-", "")?;
            for _ in 0..args.property_versions.len() + 1 {
                write!(w, "+-{:-<body_cell_width$}-", "")?
            }
            writeln!(w, "")?;

            write!(w, " {:head_cell_width$} ", "false neg")?;
            write!(w, "| {:^body_cell_width$} ", "-")?;
            for p in false_negatives {
                write!(w, "| {:^body_cell_width$} ", p)?;
            }
            writeln!(w, "")?;
            write!(w, " {:head_cell_width$} ", "false pos")?;
            write!(w, "| {:^body_cell_width$} ", "-")?;
            for p in false_positives {
                write!(w, "| {:^body_cell_width$} ", p)?;
            }
            writeln!(w, "")?;
        }
        Ok(())
    })()
    .unwrap()
}
