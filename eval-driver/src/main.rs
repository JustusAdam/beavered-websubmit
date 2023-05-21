#![feature(scoped_threads)]

extern crate anyhow;
extern crate clap;
extern crate either;
extern crate humantime;
extern crate indicatif;
use clap::Parser;

use either::Either;

use indicatif::ProgressBar;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Display, Write};
use std::str::FromStr;

use std::sync::{mpsc::channel, Arc, Mutex};

const CONFIGURATIONS: &[(Property, usize)] = &[
    (Property::Deletion, 3),
    (Property::Storage, 1),
    (Property::Disclosure, 3),
];

const ERR_MSG_VERSIONS: &[&str] = &["original", "optimized", "minimal"];

type Version<'a> = (&'a str, &'a [&'a str]);

const ALL_KNOWN_VARIANTS: &[Version] = &[
    ("lib", &["dfpp-props/basic-helpers", "lib_framework_helpers"]),
    ("baseline", &["dfpp-props/basic-helpers", "framework_helpers"]),
    ("strict", &["dfpp-props/basic-helpers", "strict_framework_helpers"]),
];

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

    #[clap(long, default_value = "verification")]
    output_directory: std::path::PathBuf,

    #[clap(long, default_value = "frg")]
    forge_source_dir: std::path::PathBuf,

    #[clap(long, default_value = "1h")]
    err_msg_timeout: humantime::Duration,

    #[clap(long, default_value = "10m")]
    check_timeout: humantime::Duration,

    /// Only run the specified edits. Uses the same format as printing edits,
    /// aka `edit-<property>-<articulation point>-<short edit type>`, e.g. `edit-del-2-a`
    #[clap(long)]
    only: Option<Vec<Edit>>,

    /// Only run these properties (similar to --only but selects edits for a
    /// whole property)
    #[clap(long)]
    only_property: Option<Vec<Property>>,

    /// Error message version to run. Options: "original", "minimal",
    /// "optimized", default to all
    #[clap(long = "emv")]
    error_message_versions: Option<Vec<String>>,

    /// Don't run any edits
    #[clap(long)]
    no_edits: bool,

    #[clap(long, default_value_t = 4)]
    parallelism: usize,
}

impl Args {
    fn verbose_commands(&self) -> bool {
        self.verbose || self.verbose_commands
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum Severity {
    Benign,
    Bug,
    Intentional,
}

impl Severity {
    fn expected_result(self, result: &RunResult) -> bool {
        match self {
            Severity::Benign => matches!(result, RunResult::Success(_)),
            Severity::Bug | Severity::Intentional => matches!(result, RunResult::CheckError(_)),
        }
    }

    fn expected_emoji(&self) -> &'static str {
        match self {
            Severity::Benign => "✅",
            _ => "❌",
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

#[derive(Clone, Eq, PartialEq, Hash, Copy, PartialOrd, Ord)]
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

use std::time::Duration;

#[derive(Clone, Copy)]
enum RunResult {
    Success(Duration),
    CompilationError,
    CheckError(Duration),
    Timeout,
}


impl std::fmt::Display for RunResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Alignment;
        let width = formatter.width().unwrap_or(2);
        let selfstr = match self {
            RunResult::Success(dur) => format!("✅ ({})", humantime::format_duration(*dur)),
            RunResult::CompilationError => "️🚧".to_string(),
            RunResult::CheckError(dur) => format!("❌ ({})", humantime::format_duration(*dur)),
            RunResult::Timeout => "⏲".to_string(),
        };
        let selfwidth = selfstr.len();
        let (before, after) = match formatter.align() {
            None => (0, width - selfwidth),
            _ if width < selfwidth => (0, 0),
            Some(Alignment::Left) => (0, width - selfwidth),
            Some(Alignment::Right) => (width - selfwidth, 0),
            Some(Alignment::Center) => {
                let left = (width - selfwidth) / 2;
                (left, width - selfwidth - left)
            }
        };
        let fill_chr = formatter.fill();
        for _ in 0..before {
            formatter.write_char(fill_chr)?;
        }
        formatter.write_str(&selfstr)?;
        for _ in 0..after {
            formatter.write_char(fill_chr)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct StringErr(String);

impl std::fmt::Display for StringErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StringErr {}

fn read_and_count_forge_unsat_instance(all: &str) -> Result<u32, String> {
    extern crate serde_lexpr as sexpr;
    use std::io::Read;
    let target = all
        .split_once("'(#hash")
        .ok_or("Did not find pattern \"'(#hash\"")?
        .1;
    let target = target
        .rsplit_once("'((")
        .ok_or("Did not find pattern \"'((\" at the file end")?
        .0;
    let target = target
        .rsplit_once(")")
        .ok_or("Did not find pattern \")\" before \"'((\" at the file end")?
        .0;
    let value = sexpr::parse::from_str(target).map_err(|e| e.to_string())?;
    let flow = value
        .get("minimal_subflow")
        .ok_or("Did not find 'minimal_subflow' key")?;
    Ok(flow
        .list_iter()
        .ok_or("'minimal_subflow' is not an s-expression list")?
        .map(|v| {
            match v
                .to_ref_vec()
                .ok_or("'minimal_subflow' elements are not lists")?
                .as_slice()
            {
                [_, from, to] => Ok((
                    from.as_symbol().ok_or(
                        "Second elements of 'minimal_subflow' elements should be a symbol",
                    )?,
                    to.as_symbol()
                        .ok_or("Third elements of 'minimal_subflow' elements should be a symbol")?,
                    0,
                )),
                _ => Err("'minimal_subflow' list elements should be 3-tuples"),
            }
        })
        .count() as u32)
}

#[derive(Clone, Copy)]
enum ErrMsgResult {
    Timeout,
    Success(std::time::Duration, u32),
    Sat(std::time::Duration),
}

impl std::fmt::Display for ErrMsgResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrMsgResult::Timeout => f.write_str("timed out"),
            ErrMsgResult::Success(time, edgecount) => write!(
                f,
                "succeeded in {} with a graph of {edgecount}",
                humantime::format_duration(*time)
            ),
            ErrMsgResult::Sat(duration) => write!(
                f,
                "was satisfiable in {}",
                humantime::format_duration(*duration)
            ),
        }
    }
}

fn wait_with_timeout(timeout: Duration, proc: &mut std::process::Child) -> std::io::Result<Option<std::process::ExitStatus>> {
    let mut status;
    let time = std::time::Instant::now();
    while {
        status = proc.try_wait()?;
        status.is_none()
    } {
        if  time.elapsed() > timeout {
            proc.kill()?;
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(status)
}

struct RunConfiguration {
    typ: Property,
    version: Version<'static>,
    edit: Option<Edit>,
    progress: &'static ProgressBar,
    args: &'static Args,
}

impl RunConfiguration {
    fn describe(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        write!(s, "{}-{}-", self.typ, self.version.0).unwrap();
        if let Some(edit) = self.edit {
            write!(s, "{edit}").unwrap();
        } else {
            s.push_str("original");
        }
        s
    }
    fn err_msg_timeout(&self) -> std::time::Duration {
        self.args.err_msg_timeout.into()
    }
    fn check_timeout(&self) -> std::time::Duration {
        self.args.check_timeout.into()
    }
    fn forge_source_dir(&self) -> &std::path::Path {
        self.args.forge_source_dir.as_path()
    }
    fn verbose(&self) -> bool {
        self.args.verbose
    }
    fn verbose_commands(&self) -> bool {
        self.args.verbose_commands()
    }
    fn outpath(&self) -> std::path::PathBuf {
        let edit = if let Some(edit) = self.edit {
            format!("{edit}")
        } else {
            "original".to_string()
        };
        self.args.output_directory.join(edit)
    }
    fn forge_file_name_for(&self, what: &str) -> String {
        format!("{}-{}-{what}.frg", self.version.0, self.typ)
    }
    fn forge_in_file(&self, what: &str) -> std::path::PathBuf {
        self.forge_source_dir().join(self.forge_file_name_for(what))
    }
    fn forge_out_file(&self, what: &str) -> std::path::PathBuf {
        self.outpath().join(self.forge_file_name_for(what))
    }
    fn analysis_result_path(&self) -> std::path::PathBuf {
        self.forge_out_file("analysis-result")
    }
    fn compile_edit(&self) -> anyhow::Result<bool> {
        use std::process::*;
        let (version, includes) = self.version;

        let result_file_path = self.analysis_result_path();
        let mut dfpp_cmd = Command::new("cargo");
        dfpp_cmd
            .args([
                "dfpp",
                "--result-path",
                &result_file_path.to_string_lossy(),
                "--model-version",
                "v2",
                "--inline-elision",
                "--skip-sigs",
                "--abort-after-analysis",
            ])
            .stdin(Stdio::null());
        let external_ann_file_name = format!("{}-external-annotations.toml", self.version.0);
        if std::path::Path::new(&external_ann_file_name).exists() {
            dfpp_cmd.args(&["--external-annotations", external_ann_file_name.as_str()]);
        }
        dfpp_cmd.args(&["--", "--features", &format!("v-ann-{version}")]);
        if let Some(edit) = self.edit {
            dfpp_cmd.args(&["--features", &edit.to_string()]);
        }
        if !self.verbose() {
            dfpp_cmd.stderr(Stdio::null()).stdout(Stdio::null());
        }
        if self.verbose_commands() {
            self.progress
                .suspend(|| println!("Executing compile command: {:?}", dfpp_cmd));
        }
        let status = dfpp_cmd.status()?;
        self.progress.inc(1);
        Ok(status.success())
    }

    fn run_edit(&self) -> anyhow::Result<RunResult> {
        let now = std::time::Instant::now();
        use std::process::*;
        let check_file_path = self.forge_out_file("check");
        {
            use std::io::{Read, Write};
            let mut w = std::fs::OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(&check_file_path)?;
            self.write_headers_and_prop(&mut w, "dfpp-props/sigs")?;
            writeln!(
                w,
                "test expect {{ {}_{}: {{ property[flow, labels] }} for Flows is theorem }}",
                self.version.0, self.typ
            )?;
        }
        let mut racket_cmd = Command::new("racket");
        racket_cmd
            .arg(&check_file_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped());
        if !self.verbose() {
            racket_cmd.stderr(Stdio::null()).stdout(Stdio::null());
        }
        if self.verbose_commands() {
            self.progress
                .suspend(|| println!("Executing check command: {:?}", racket_cmd));
        }
        let status = wait_with_timeout(self.check_timeout(), &mut racket_cmd.spawn()?)?;
        self.progress.inc(1);
        status.map_or(Ok(RunResult::Timeout), |status| {
            if status.success() {
                Ok(RunResult::Success(now.elapsed()))
            } else {
                Ok(RunResult::CheckError(now.elapsed()))
            }
        })
    }

    fn run_error_msg(&self, template: &str) -> anyhow::Result<ErrMsgResult> {
        use std::process::*;
        let frg_file = self.forge_out_file(&format!("err-msg-check-{template}"));
        {
            use std::io::{copy, Read, Write};
            let mut w = std::fs::OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(&frg_file)?;
            let sig_file = if template == "optimized" {
                "dfpp-props/err_msg_optimized_sigs"
            } else {
                "dfpp-props/err_msg_sigs"
            };
            self.write_headers_and_prop(&mut w, sig_file)?;
            let template_file = self
                .forge_source_dir()
                .join(&format!("dfpp-props/err_msg_template_{template}.frg"));
            copy(&mut std::fs::File::open(template_file)?, &mut w)?;
        }
        let mut racket_cmd = Command::new("racket");
        racket_cmd.arg(&frg_file).stdin(Stdio::null());
        if !self.verbose() {
            racket_cmd.stderr(Stdio::null()).stdout(Stdio::null());
        }
        if self.verbose_commands() {
            self.progress
                .suspend(|| println!("Executing check command: {:?}", racket_cmd));
        }
        let time = std::time::Instant::now();
        let mut child = racket_cmd.spawn()?;

        let status = wait_with_timeout(self.err_msg_timeout(), &mut child)?;

        if let Some(status) = status {
            let output = child.wait_with_output()?;
            if status.success() {
                Ok(ErrMsgResult::Sat(time.elapsed()))
            } else {
                let forge_output_str = String::from_utf8_lossy(&output.stdout);
                let counting_tesult = read_and_count_forge_unsat_instance(&forge_output_str);
                if counting_tesult.is_err() {
                    use std::io::Write;
                    write!(
                        &mut std::fs::OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(self.args.output_directory.join("err_msg_ouput.txt"))?,
                        "{}",
                        forge_output_str,
                    )?;
                }
                Ok(ErrMsgResult::Success(
                    time.elapsed(),
                    counting_tesult.map_err(StringErr)?,
                ))
            }
        } else {
             Ok(ErrMsgResult::Timeout)
        }
    }

    fn write_headers_and_prop<W: std::io::Write>(
        &self,
        mut w: W,
        sigs: &str,
    ) -> std::io::Result<()> {
        use std::io::{copy, Read, Write};
        let propfile = self.forge_in_file("props");
        writeln!(w, "#lang forge")?;
        let ana_path = self.analysis_result_path();
        use Either::*;
        for include in [Right(sigs), Left(ana_path)]
            .into_iter()
            .chain(self.version.1.iter().copied().map(Right))
        {
            writeln!(w)?;
            let path = match include {
                Right(include) => self.forge_source_dir().join(include).with_extension("frg"),
                Left(path) => path,
            };
            writeln!(w, "// {}", path.display())?;
            copy(&mut std::fs::File::open(path)?, &mut w)?;
        }
        copy(&mut std::fs::File::open(propfile)?, &mut w)?;
        Ok(())
    }
}


type ResultTable = HashMap<
    Property,
    HashMap<
        Option<Edit>,
        HashMap<
            &'static str,
            (
                RunConfiguration,
                (Option<RunResult>, Vec<(&'static str, ErrMsgResult)>),
            ),
        >,
    >,
>;

fn print_results_for_property<W: std::io::Write>(
    mut w: W,
    num_versions: usize,
    property_versions: &[Version],
    results: &ResultTable,
) -> std::io::Result<()> {
    let head_cell_width = 12;
    let body_cell_width = 30;
    for (typ, results) in results.iter() {
        let mut false_negatives = Vec::with_capacity(num_versions);
        false_negatives.resize(num_versions, 0);
        let mut false_positives = Vec::with_capacity(num_versions);
        false_positives.resize(num_versions, 0);

        write!(w, " {:head_cell_width$} ", typ.to_string(),)?;
        write!(w, "| {:body_cell_width$} ", "expected")?;
        for (version, _) in property_versions.iter() {
            write!(w, "| {:body_cell_width$} ", version)?
        }
        writeln!(w, "")?;
        write!(w, "-{:-<head_cell_width$}-", "")?;
        for _ in 0..property_versions.len() + 1 {
            write!(w, "+-{:-<body_cell_width$}-", "")?
        }
        writeln!(w, "")?;
        let mut edits = results.iter().collect::<Vec<_>>();
        edits.sort_by_key(|e| e.0);
        for (edit, versions) in edits {
            let edit_str = edit.as_ref().map_or("none".to_string(), Edit::to_string);
            write!(w, " {:head_cell_width$} ", edit_str)?;
            write!(w, "| {:^body_cell_width$} ", if let Some(edit) = &edit {
                edit.severity.expected_emoji()
            } else {
                "✅"
            })?;
            for (i, (_, (_, mutex))) in versions.iter().enumerate() {
                let result = mutex;
                let run_result = result.0.unwrap();
                let was_expected = if let Some(edit) = edit {
                    edit.severity.expected_result(&run_result)
                } else {
                    matches!(run_result, RunResult::Success(_))
                };
                if !was_expected {
                    match run_result {
                        RunResult::CheckError(_) => false_positives[i] += 1,
                        RunResult::Success(_) => false_negatives[i] += 1,
                        _ => (),
                    };
                }
                write!(w, "| {:^body_cell_width$} ", run_result)?;
            }
            writeln!(w, "")?;
        }
        write!(w, "-{:-<head_cell_width$}-", "")?;
        for _ in 0..property_versions.len() + 1 {
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
}

fn main() {
    let args = Box::leak::<'static>(Box::new(Args::parse()));
    std::env::set_current_dir(&args.directory);
    use std::io::Write;
    assert!(args.parallelism > 0);
    let property_versions: Vec<_> = if args.property_versions.is_empty() {
        println!("INFO: No specification variants to run given, running all known ones");
        ALL_KNOWN_VARIANTS.to_vec()
    } else {
        ALL_KNOWN_VARIANTS
            .iter()
            .cloned()
            .filter(|v| args.property_versions.iter().any(|e| e.as_str() == v.0))
            .collect()
    };

    let error_message_versions: Vec<_> = if let Some(v) = args.error_message_versions.as_ref() {
        let str_refs = v.iter().map(String::as_str).collect::<Vec<_>>();
        if let ["none"] = str_refs.as_slice() {
            vec![]
        } else {
            str_refs
        }
    } else {
        ERR_MSG_VERSIONS.to_vec()
    };

    let ref is_selected = {
        let as_ref_v = args
            .only
            .as_ref()
            .map(|v| v.iter().cloned().collect::<HashSet<Edit>>());
        let args_ref = &args;
        move |s: &Edit| !args_ref.no_edits && as_ref_v.as_ref().map_or(true, |v| v.contains(s))
    };

    let num_versions = property_versions.len();

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
            (args.no_edits || !new_edits.is_empty()).then_some((property, new_edits))
        })
        .collect();

    let num_configurations = configurations
        .iter()
        .map(
            |(_, inner)| inner.len() + 1, // default (no edits)
        )
        .sum::<usize>()
        * (2 * num_versions // compile + prop check
           + num_versions * error_message_versions.len());

    let mut progress = Box::leak::<'static>(Box::new(
        ProgressBar::new(num_configurations as u64).with_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg:11} {bar:40} {pos:>3}/{len:3}")
                .unwrap(),
        ),
    ));

    let mut w = std::io::stdout();
    let mut dir_builder = std::fs::DirBuilder::new();
    dir_builder.recursive(true);
    let mut results: ResultTable = configurations
        .into_iter()
        .map(|(typ, edits)| {
            (
                typ,
                edits
                    .iter()
                    .copied()
                    .map(Some)
                    .chain([None])
                    .map(|edit| {
                        progress.set_message(edit.map_or("default".to_string(), |e| e.to_string()));
                        (
                            edit,
                            property_versions
                                .iter()
                                .map(|&version| {
                                    let config = RunConfiguration {
                                        typ,
                                        version,
                                        edit,
                                        progress,
                                        args,
                                    };
                                    let outpath = config.outpath();
                                    if !outpath.exists() {
                                        dir_builder.create(outpath).unwrap();
                                    }
                                    assert!(config.compile_edit().unwrap());
                                    (version.0, (config, (None, vec![])))
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            )
        })
        .collect();

    for t in results.values_mut() {
        for e in t.values_mut() {
            for (config, results) in e.values_mut() {
                assert!(results.0.replace(config.run_edit().unwrap()).is_none());
            }
        }
    }


    print_results_for_property(
        &mut w,
        num_versions,
        property_versions.as_slice(),
        &results,
    )
    .unwrap();
    writeln!(w, "Error message results:").unwrap();

    for t in results.values_mut() {
        for e in t.values_mut() {
            for (config, mutex) in e.values_mut() {
                if matches!(
                    mutex.0.unwrap(),
                    RunResult::CheckError(_)
                ) {
                    for emv in error_message_versions.iter() {
                        let emvresult = config.run_error_msg(emv).unwrap();
                        progress.inc(1);
                        mutex.1.push((emv, emvresult));
                    }
                } else {
                    progress.inc(error_message_versions.len() as u64);
                }
            }
        }
    }


    for type_results in results.values() {
        for edit_results in type_results.values() {
            for (config, result) in edit_results.values() {
                if matches!(result.0, Some(RunResult::CheckError(_))) {
                    for (emv, result) in result.1.iter() {
                        progress.suspend(|| {
                            writeln!(w, "{}: {emv} {result}", config.describe()).unwrap();
                        });
                    }
                } 
            }
        }
    }
    progress.finish_and_clear();
}


