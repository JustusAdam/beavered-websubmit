#![feature(scoped_threads)]

extern crate anyhow;
extern crate clap;
extern crate either;
extern crate humantime;
extern crate indicatif;
use anyhow::Error;
use clap::Parser;

use either::Either;

use indicatif::ProgressBar;
use props::run_del_policy;
use props::run_dis_policy;
use props::run_sc_policy;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Display, Write};
use std::str::FromStr;

use std::sync::{Arc, Mutex};

use paralegal_policy::GraphLocation;

mod props;

const CONFIGURATIONS: &[(Property, usize)] = &[
    (Property::Deletion, 3),
    (Property::Storage, 1),
    (Property::Disclosure, 3),
];

const ERR_MSG_VERSIONS: &[&str] = &["original", "optimized", "minimal", "labels", "min_labels"];

type Version<'a> = (&'a str, &'a [&'a str]);

const ALL_KNOWN_VARIANTS: &[Version] = &[
    (
        "lib",
        &["dfpp-props/basic-helpers", "lib_framework_helpers"],
    ),
    (
        "baseline",
        &["dfpp-props/basic-helpers", "framework_helpers"],
    ),
    (
        "strict",
        &["dfpp-props/basic-helpers", "strict_framework_helpers"],
    ),
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
    #[clap(long)]
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

    #[clap(long, default_value_t = 1)]
    parallelism: usize,

    /// Select what kind of property type (Rust or Forge) to run.
    #[clap(long)]
    prop_type: Option<PropType>,
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
    fn expected_result(self, result: &CheckResult) -> bool {
        match self {
            Severity::Benign => matches!(result, CheckResult::Success(_)),
            Severity::Bug | Severity::Intentional => matches!(result, CheckResult::Error(_)),
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

#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum PropType {
    Rust,
    Forge,
}

impl Display for PropType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(match self {
            PropType::Rust => "r",
            PropType::Forge => "f",
        })
    }
}

impl FromStr for PropType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(PropType::Rust),
            "forge" => Ok(PropType::Forge),
            _ => Err(format!("Unrecognized severity type {s}")),
        }
    }
}

use std::time::Duration;
#[derive(Clone, Copy)]
enum CheckResult {
    Success(Duration),
    Error(Duration),
    Timeout,
}

impl std::fmt::Display for CheckResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let selfstr = match self {
            CheckResult::Success(dur) => {
                format!(
                    "✅({})",
                    humantime::format_duration(Duration::from_millis(
                        Duration::as_millis(dur).try_into().unwrap()
                    )),
                )
            }
            CheckResult::Error(dur) => format!(
                "❌({})",
                humantime::format_duration(Duration::from_millis(
                    Duration::as_millis(dur).try_into().unwrap()
                ))
            ),
            CheckResult::Timeout => "⏲".to_string(),
        };
        formatter.write_str(&selfstr)?;
        Ok(())
    }
}

#[derive(Clone)]
enum RunResult {
    CompilationError,
    CheckResult(Vec<(PropType, CheckResult)>),
}

impl std::fmt::Display for RunResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Alignment;
        let width = formatter.width().unwrap_or(2);
        let selfstr = match self {
            RunResult::CompilationError => "️🚧".to_string(),
            RunResult::CheckResult(results) => {
                let mut s = String::new();
                for (prop_type, r) in results {
                    s.push_str(format!("{}: {},", prop_type, r).as_str())
                }
                s
            }
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

fn read_forge_unsat_instance(all: &str) -> Result<serde_lexpr::Value, String> {
    extern crate serde_lexpr as sexpr;
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
    sexpr::parse::from_str(target).map_err(|e| e.to_string())
}

fn read_and_count_forge_unsat_instance_markers(all: &str) -> Result<usize, String> {
    extern crate serde_lexpr as sexpr;
    let value = read_forge_unsat_instance(all)?;
    Ok(value
        .get("additional_labels")
        .ok_or("Did not find 'additional_labels' key")?
        .list_iter()
        .ok_or("'additional_labels' is not an s-expression list")?
        .map(|v| {
            match v
                .to_ref_vec()
                .ok_or("'additional_labels' elements are not lists")?
                .as_slice()
            {
                [from, to] => Ok((
                    from.as_symbol().ok_or(
                        "Second elements of 'additional_labels' elements should be a symbol",
                    )?,
                    to.as_symbol().ok_or(
                        "Third elements of 'additional_labels' elements should be a symbol",
                    )?,
                    0,
                )),
                _ => Err("'additional_labels' list elements should be 2-tuples"),
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .len())
}

fn read_and_count_forge_unsat_instance_edges(all: &str) -> Result<(usize, usize), String> {
    extern crate serde_lexpr as sexpr;
    let value = read_forge_unsat_instance(all)?;
    let flow = value
        .get("minimal_subflow")
        .ok_or("Did not find 'minimal_subflow' key")?;
    let err_graph_edges = flow
        .list_iter()
        .ok_or("'minimal_subflow' is not an s-expression list")?
        .map(|v| {
            match v
                .to_ref_vec()
                .ok_or("'minimal_subflow' elements are not lists")?
                .as_slice()
            {
                [from, to] => Ok((
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
        .collect::<Result<Vec<_>, _>>()?
        .len();
    let regular_graph_edges = value
        .get("flow")
        .ok_or("could not find 'flow'")?
        .list_iter()
        .ok_or("'flow' is not an s-expression list")?
        .map(|v| {
            match v
                .to_ref_vec()
                .ok_or("'flow' elements are not lists")?
                .as_slice()
            {
                [from, to] => Ok((
                    from.as_symbol()
                        .ok_or("Second elements of 'flow' elements should be a symbol")?,
                    to.as_symbol()
                        .ok_or("Third elements of 'flow' elements should be a symbol")?,
                    0,
                )),
                _ => Err("'minimal_subflow' list elements should be 3-tuples"),
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .len();
    Ok((regular_graph_edges, err_graph_edges))
}

#[derive(Clone, Copy)]
enum ErrMsgResultPayload {
    Markers(usize),
    ErrGraph {
        regular_edges: usize,
        error_edges: usize,
    },
}

#[derive(Clone, Copy)]
enum ErrMsgResult {
    Timeout,
    Success {
        runtime: std::time::Duration,
        payload: ErrMsgResultPayload,
    },
    Sat(std::time::Duration),
}

impl std::fmt::Display for ErrMsgResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrMsgResult::Timeout => f.write_str("timed out"),
            ErrMsgResult::Success {
                runtime,
                payload:
                    ErrMsgResultPayload::ErrGraph {
                        regular_edges,
                        error_edges,
                    },
            } => write!(
                f,
                "succeeded in {} with {error_edges} of {regular_edges} edges",
                humantime::format_duration(*runtime)
            ),
            ErrMsgResult::Success {
                runtime,
                payload: ErrMsgResultPayload::Markers(markers),
            } => write!(
                f,
                "succeeded in {} with {markers} new markers",
                humantime::format_duration(*runtime)
            ),
            ErrMsgResult::Sat(duration) => write!(
                f,
                "was satisfiable in {}",
                humantime::format_duration(*duration)
            ),
        }
    }
}

fn wait_with_timeout(
    timeout: Duration,
    proc: &mut std::process::Child,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let mut status;
    let time = std::time::Instant::now();
    while {
        status = proc.try_wait()?;
        status.is_none()
    } {
        if time.elapsed() > timeout {
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
    fn graph_loc_out_file(&self) -> std::path::PathBuf {
        self.outpath().join("flow-graph.json")
    }
    fn compile_edit(&self) -> anyhow::Result<()> {
        use std::process::*;
        let (version, _) = self.version;

        let result_file_path = self.analysis_result_path();
        let graph_loc_path = self.graph_loc_out_file();
        let mut command = paralegal_policy::SPDGGenCommand::global();
        command
            .get_command()
            .args([
                "--result-path",
                &result_file_path.to_string_lossy(),
                "--graph-loc-path",
                &graph_loc_path.to_string_lossy(),
                "--model-version",
                "v2",
                "--inline-elision",
                "--skip-sigs",
                "--abort-after-analysis",
            ])
            .stdin(Stdio::null());
        let external_ann_file_name = format!("{}-external-annotations.toml", self.version.0);
        if std::path::Path::new(&external_ann_file_name).exists() {
            command
                .get_command()
                .args(&["--external-annotations", external_ann_file_name.as_str()]);
        }
        command
            .get_command()
            .args(&["--", "--features", &format!("v-ann-{version}")]);
        if let Some(edit) = self.edit {
            command
                .get_command()
                .args(&["--features", &edit.to_string()]);
        }
        if !self.verbose() {
            command
                .get_command()
                .stderr(Stdio::null())
                .stdout(Stdio::null());
        }
        if self.verbose_commands() {
            self.progress
                .suspend(|| println!("Executing compile command: {:?}", command));
        }
        command.run(".")?;
        self.progress.inc(1);
        Ok(())
    }

    fn run_edit(&self, compile_result: &Result<(), anyhow::Error>) -> anyhow::Result<RunResult> {
        if let Err(_) = compile_result {
            return Ok(RunResult::CompilationError);
        };

        let results = match self.args.prop_type {
            Some(prop_type) => match prop_type {
                PropType::Rust => vec![(PropType::Rust, self.run_rust_prop()?)],
                PropType::Forge => vec![(PropType::Forge, self.run_forge_prop()?)],
            },
            None => vec![
                (PropType::Forge, self.run_forge_prop()?),
                (PropType::Rust, self.run_rust_prop()?),
            ],
        };
        Ok(RunResult::CheckResult(results))
    }

    fn run_rust_prop(&self) -> anyhow::Result<CheckResult> {
        let now = std::time::Instant::now();

        let gl = GraphLocation::custom(self.graph_loc_out_file());
        let ctx = Arc::new(gl.build_context()?);
        if self.verbose_commands() {
            self.progress
                .suspend(|| println!("Executing check for forge property"));
        }
        if self.verbose() {
            if ctx.desc().controllers.is_empty() {
                self.progress.suspend(|| {
                    println!("No controllers found. Your policy is likely to be vacuous.")
                });
            }
        }
        let prop = match self.typ {
            Property::Deletion => run_del_policy,
            Property::Storage => run_sc_policy,
            Property::Disclosure => run_dis_policy,
        };
        prop(ctx.clone(), self.version.0)?;
        let passed = if self.verbose() {
            self.progress
                .suspend(|| ctx.emit_diagnostics(std::io::stdout()))?
        } else {
            ctx.emit_diagnostics(std::io::sink())?
        };

        if passed {
            Ok(CheckResult::Success(now.elapsed()))
        } else {
            Ok(CheckResult::Error(now.elapsed()))
        }
    }

    fn run_forge_prop(&self) -> anyhow::Result<CheckResult> {
        let now = std::time::Instant::now();

        use std::process::*;
        let check_file_path = self.forge_out_file("check");
        {
            use std::io::Write;
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
        status.map_or(Ok(CheckResult::Timeout), |status| {
            if status.success() {
                Ok(CheckResult::Success(now.elapsed()))
            } else {
                Ok(CheckResult::Error(now.elapsed()))
            }
        })
    }

    fn run_error_msg(&self, template: &str) -> anyhow::Result<ErrMsgResult> {
        use std::process::*;
        let frg_file = self.forge_out_file(&format!("err-msg-check-{template}"));
        {
            use std::io::copy;
            let mut w = std::fs::OpenOptions::new()
                .truncate(true)
                .write(true)
                .create(true)
                .open(&frg_file)?;
            let sig_file = match template {
                "optimized" => "dfpp-props/err_msg_optimized_sigs",
                "labels" | "min_labels" => "dfpp-props/err_msg_labels_sigs",
                "original" | "minimal" => "dfpp-props/err_msg_sigs",
                _ => Err(StringErr(format!("Unknown err message type {template}")))?,
            };
            self.write_headers_and_prop(&mut w, sig_file)?;
            let template_file = self
                .forge_source_dir()
                .join(&format!("dfpp-props/err_msg_template_{template}.frg"));
            copy(&mut std::fs::File::open(template_file)?, &mut w)?;
        }
        let forge_output_path = self.outpath().join(format!(
            "{}-{}-err-msg-result-{template}.txt",
            self.version.0, self.typ
        ));
        let mut racket_cmd = Command::new("racket");
        racket_cmd
            .arg(&frg_file)
            .stdin(Stdio::null())
            .stderr(Stdio::null());
        let forge_output_file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&forge_output_path)?;
        racket_cmd.stdout(forge_output_file);
        if self.verbose_commands() {
            self.progress
                .suspend(|| println!("Executing check command: {:?}", racket_cmd));
        }
        let time = std::time::Instant::now();
        let mut child = racket_cmd.spawn()?;

        let status = wait_with_timeout(self.err_msg_timeout(), &mut child)?;

        if let Some(status) = status {
            if status.success() {
                Ok(ErrMsgResult::Sat(time.elapsed()))
            } else {
                use std::io::Read;
                let mut forge_output_str = String::new();
                std::fs::File::open(forge_output_path)?.read_to_string(&mut forge_output_str)?;
                let payload = match template {
                    "min_labels" | "labels" => {
                        read_and_count_forge_unsat_instance_markers(&forge_output_str)
                            .map(ErrMsgResultPayload::Markers)
                    }
                    "original" | "optimized" | "minimal" => {
                        read_and_count_forge_unsat_instance_edges(&forge_output_str).map(
                            |(regular_edges, error_edges)| ErrMsgResultPayload::ErrGraph {
                                regular_edges,
                                error_edges,
                            },
                        )
                    }
                    _ => Err(format!("Unknown error message version {template}")),
                }
                .map_err(|e| StringErr(format!("{template} {}: ", self.describe()) + &e))?;
                Ok(ErrMsgResult::Success {
                    runtime: time.elapsed(),
                    payload,
                })
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
        use std::io::copy;
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

type ResultPayload = (Option<RunResult>, Vec<(&'static str, ErrMsgResult)>);

type ResultTable<T> = HashMap<
    Property,
    HashMap<Option<Edit>, HashMap<&'static str, (RunConfiguration, Result<(), Error>, T)>>,
>;

type ParResultTable = ResultTable<Mutex<ResultPayload>>;

type SeqResultTable = ResultTable<ResultPayload>;

const HEAD_CELL_WIDTH: usize = 12;
const BODY_CELL_WIDTH: usize = 40;

fn print_results_for_property<
    T,
    F: FnMut(&mut W, &Option<Edit>, &T) -> std::io::Result<Vec<ResultClassification>>,
    W: std::io::Write,
>(
    mut w: W,
    num_versions: usize,
    property_versions: &[Version],
    results: &ResultTable<T>,
    mut f: F,
) -> std::io::Result<()> {
    for (typ, results) in results.iter() {
        let mut false_negatives = Vec::with_capacity(num_versions);
        false_negatives.resize(num_versions, 0);
        let mut false_positives = Vec::with_capacity(num_versions);
        false_positives.resize(num_versions, 0);

        write!(w, " {:HEAD_CELL_WIDTH$} ", typ.to_string(),)?;
        write!(w, "| {:HEAD_CELL_WIDTH$} ", "expected")?;
        for (version, _) in property_versions.iter() {
            write!(w, "| {:BODY_CELL_WIDTH$} ", version)?
        }
        writeln!(w, "")?;
        write!(w, "-{:-<HEAD_CELL_WIDTH$}-+-{:-<HEAD_CELL_WIDTH$}-", "", "")?;
        for _ in 0..property_versions.len() {
            write!(w, "+-{:-<BODY_CELL_WIDTH$}-", "")?
        }
        writeln!(w, "")?;
        let mut edits = results.iter().collect::<Vec<_>>();
        edits.sort_by_key(|e| e.0);
        for (edit, versions) in edits {
            let edit_str = edit.as_ref().map_or("none".to_string(), Edit::to_string);
            write!(w, " {:HEAD_CELL_WIDTH$} ", edit_str)?;
            write!(
                w,
                "| {:^HEAD_CELL_WIDTH$} ",
                if let Some(edit) = &edit {
                    edit.severity.expected_emoji()
                } else {
                    "✅"
                }
            )?;
            for (i, (version, _)) in property_versions.iter().enumerate() {
                let (_, _, mutex) = versions.get(version).unwrap();
                for r in f(&mut w, edit, mutex)? {
                    match r {
                        ResultClassification::FalsePositive => false_positives[i] += 1,
                        ResultClassification::FalseNegative => false_negatives[i] += 1,
                        ResultClassification::Uninteresting => (),
                    }
                }
            }
            writeln!(w, "")?;
        }
        write!(w, "-{:-<HEAD_CELL_WIDTH$}-+-{:-<HEAD_CELL_WIDTH$}-", "", "")?;
        for _ in 0..property_versions.len() {
            write!(w, "+-{:-<BODY_CELL_WIDTH$}-", "")?
        }
        writeln!(w, "")?;

        write!(w, " {:HEAD_CELL_WIDTH$} ", "false neg")?;
        write!(w, "| {:^HEAD_CELL_WIDTH$} ", "-")?;
        for p in false_negatives {
            write!(w, "| {:^BODY_CELL_WIDTH$} ", p)?;
        }
        writeln!(w, "")?;
        write!(w, " {:HEAD_CELL_WIDTH$} ", "false pos")?;
        write!(w, "| {:^HEAD_CELL_WIDTH$} ", "-")?;
        for p in false_positives {
            write!(w, "| {:^BODY_CELL_WIDTH$} ", p)?;
        }
        writeln!(w, "")?;
    }
    Ok(())
}

enum ResultClassification {
    FalsePositive,
    FalseNegative,
    Uninteresting,
}

fn main() {
    let args = Box::leak::<'static>(Box::new(Args::parse()));
    std::env::set_current_dir(&args.directory).unwrap();
    assert!(args.parallelism > 0);
    if args.parallelism == 1 {
        main_seq(args);
    } else {
        // main_par(args);
    }
}

fn main_seq(args: &'static Args) {
    use std::io::Write;
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

    let progress = Box::leak::<'static>(Box::new(
        ProgressBar::new(num_configurations as u64).with_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg:11} {bar:40} {pos:>3}/{len:3}")
                .unwrap(),
        ),
    ));

    let mut w = std::io::stdout();
    let mut dir_builder = std::fs::DirBuilder::new();
    dir_builder.recursive(true);
    let mut results: SeqResultTable = configurations
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
                                    assert!(edit.as_ref().map_or(true, |e| e.property == typ));
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
                                    let compile_result = config.compile_edit();
                                    (version.0, (config, compile_result, (None, vec![])))
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
            for (config, compile_result, results) in e.values_mut() {
                assert!(results
                    .0
                    .replace(config.run_edit(compile_result).unwrap())
                    .is_none());
            }
        }
    }

    print_results_for_property(
        &mut w,
        num_versions,
        property_versions.as_slice(),
        &results,
        |w, edit, result| {
            let run_result = result.0.as_ref().unwrap();

            write!(w, "| {:^BODY_CELL_WIDTH$} ", run_result)?;

            match run_result {
                RunResult::CompilationError => Ok(vec![ResultClassification::Uninteresting]),
                RunResult::CheckResult(results) => Ok(results
                    .iter()
                    .map(|(_, check_result)| {
                        let was_expected = if let Some(edit) = edit {
                            edit.severity.expected_result(&check_result)
                        } else {
                            matches!(check_result, CheckResult::Success(_))
                        };

                        match check_result {
                            CheckResult::Error(_) if !was_expected => {
                                ResultClassification::FalsePositive
                            }
                            CheckResult::Success(_) if !was_expected => {
                                ResultClassification::FalseNegative
                            }
                            _ => ResultClassification::Uninteresting,
                        }
                    })
                    .collect::<Vec<_>>()),
            }
        },
    )
    .unwrap();
    writeln!(w, "Error message results:").unwrap();

    for t in results.values_mut() {
        for e in t.values_mut() {
            for (config, _, mutex) in e.values_mut() {
                let mut ran_emvs = false;
                if let RunResult::CheckResult(v) = mutex.0.as_ref().unwrap() {
                    for check_result in v {
                        if matches!(check_result, (PropType::Forge, CheckResult::Error(_))) {
                            for emv in error_message_versions.iter() {
                                let emvresult = config.run_error_msg(emv).unwrap();
                                progress.inc(1);
                                mutex.1.push((emv, emvresult));
                                ran_emvs = true;
                            }
                        }
                    }
                }

                if !ran_emvs {
                    progress.inc(error_message_versions.len() as u64);
                }
            }
        }
    }

    for type_results in results.values() {
        for edit_results in type_results.values() {
            for (config, _, result) in edit_results.values() {
                if let Some(RunResult::CheckResult(v)) = &result.0 {
                    for check_result in v {
                        if matches!(check_result, (PropType::Forge, CheckResult::Error(_))) {
                            for (emv, result) in result.1.iter() {
                                progress.suspend(|| {
                                    writeln!(w, "{}: {emv} {result}", config.describe()).unwrap();
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    progress.finish_and_clear();
}

// fn main_par(args: &'static Args) {
//     use std::io::Write;
//     let property_versions: Vec<_> = if args.property_versions.is_empty() {
//         println!("INFO: No specification variants to run given, running all known ones");
//         ALL_KNOWN_VARIANTS.to_vec()
//     } else {
//         ALL_KNOWN_VARIANTS
//             .iter()
//             .cloned()
//             .filter(|v| args.property_versions.iter().any(|e| e.as_str() == v.0))
//             .collect()
//     };

//     let error_message_versions: Vec<_> = if let Some(v) = args.error_message_versions.as_ref() {
//         let str_refs = v.iter().map(String::as_str).collect::<Vec<_>>();
//         if let ["none"] = str_refs.as_slice() {
//             vec![]
//         } else if let Some(e) = str_refs.iter().find(|r| !ERR_MSG_VERSIONS.contains(r)) {
//             panic!("Unknown error message version {e}");
//         } else {
//             str_refs
//         }
//     } else {
//         ERR_MSG_VERSIONS.to_vec()
//     };

//     let ref is_selected = {
//         let as_ref_v = args
//             .only
//             .as_ref()
//             .map(|v| v.iter().cloned().collect::<HashSet<Edit>>());
//         let args_ref = &args;
//         move |s: &Edit| !args_ref.no_edits && as_ref_v.as_ref().map_or(true, |v| v.contains(s))
//     };

//     let num_versions = property_versions.len();

//     let configurations: Vec<(_, Vec<_>)> = CONFIGURATIONS
//         .iter()
//         .filter(|conf| {
//             args.only_property
//                 .as_ref()
//                 .map_or(true, |p| p.contains(&conf.0))
//         })
//         .flat_map(|&(property, num_edits)| {
//             assert!(num_edits > 0);
//             let new_edits = (1..=num_edits)
//                 .flat_map(|articulation_point| {
//                     [Severity::Benign, Severity::Bug, Severity::Intentional]
//                         .into_iter()
//                         .map(move |severity| Edit {
//                             severity,
//                             articulation_point,
//                             property,
//                         })
//                         .filter(|e| is_selected(e))
//                 })
//                 .collect::<Vec<_>>();
//             (args.no_edits || !new_edits.is_empty()).then_some((property, new_edits))
//         })
//         .collect();

//     let num_configurations = configurations
//         .iter()
//         .map(
//             |(_, inner)| inner.len() + 1, // default (no edits)
//         )
//         .sum::<usize>()
//         * (2 * num_versions // compile + prop check
//        + num_versions * error_message_versions.len());

//     let mut progress = Box::leak::<'static>(Box::new(
//         ProgressBar::new(num_configurations as u64).with_style(
//             indicatif::ProgressStyle::default_bar()
//                 .template("{msg:11} {bar:40} {pos:>3}/{len:3}")
//                 .unwrap(),
//         ),
//     ));

//     let mut w = std::io::stdout();
//     let mut dir_builder = std::fs::DirBuilder::new();
//     dir_builder.recursive(true);
//     let results: ParResultTable = configurations
//         .into_iter()
//         .map(|(typ, edits)| {
//             (
//                 typ,
//                 edits
//                     .iter()
//                     .copied()
//                     .map(Some)
//                     .chain([None])
//                     .map(|edit| {
//                         progress.set_message(edit.map_or("default".to_string(), |e| e.to_string()));
//                         (
//                             edit,
//                             property_versions
//                                 .iter()
//                                 .map(|&version| {
//                                     assert!(edit.as_ref().map_or(true, |e| e.property == typ));
//                                     let config = RunConfiguration {
//                                         typ,
//                                         version,
//                                         edit,
//                                         progress,
//                                         args,
//                                     };
//                                     let outpath = config.outpath();
//                                     if !outpath.exists() {
//                                         dir_builder.create(outpath).unwrap();
//                                     }
//                                     let compile_result = config.compile_edit();
//                                     (
//                                         version.0,
//                                         (config, compile_result, Mutex::new((None, vec![]))),
//                                     )
//                                 })
//                                 .collect(),
//                         )
//                     })
//                     .collect(),
//             )
//         })
//         .collect();

//     std::thread::scope(|scope| {
//         let (send_work, receive_work) = channel();

//         for t in results.values() {
//             for e in t.values() {
//                 for descr in e.values() {
//                     send_work.send(descr).unwrap()
//                 }
//             }
//         }

//         let receive_work = Arc::new(Mutex::new(receive_work));

//         for _ in 0..args.parallelism {
//             let my_receive = receive_work.clone();
//             let my_results_ref = &results;
//             std::thread::Builder::new()
//                 .spawn_scoped(scope, move || {
//                     while let Some((config, compile_result, mutex)) =
//                         my_receive.lock().ok().and_then(|r| r.recv().ok())
//                     {
//                         let mut guard = mutex.try_lock().unwrap();
//                         assert!(guard
//                             .0
//                             .replace(config.run_edit(compile_result).unwrap())
//                             .is_none());
//                     }
//                 })
//                 .unwrap();
//         }
//     });

//     print_results_for_property(
//         &mut w,
//         num_versions,
//         property_versions.as_slice(),
//         &results,
//         |w, edit, mutex| {
//             let result = mutex.try_lock().unwrap();
//             let run_result = result.0.as_ref().unwrap();

//             write!(w, "| {:^BODY_CELL_WIDTH$} ", run_result)?;

//             match run_result {
//                 RunResult::CompilationError => Ok(vec![ResultClassification::Uninteresting]),
//                 RunResult::CheckResult(results) => Ok(results
//                     .iter()
//                     .map(|(_, check_result)| {
//                         let was_expected = if let Some(edit) = edit {
//                             edit.severity.expected_result(&check_result)
//                         } else {
//                             matches!(check_result, CheckResult::Success(_))
//                         };

//                         match check_result {
//                             CheckResult::Error(_) if !was_expected => {
//                                 ResultClassification::FalsePositive
//                             }
//                             CheckResult::Success(_) if !was_expected => {
//                                 ResultClassification::FalseNegative
//                             }
//                             _ => ResultClassification::Uninteresting,
//                         }
//                     })
//                     .collect::<Vec<_>>()),
//             }
//         },
//     )
//     .unwrap();
//     writeln!(w, "Error message results:").unwrap();
//     std::thread::scope(|scope| {
//         let (send_work, receive_work) = channel();

//         for t in results.values() {
//             for e in t.values() {
//                 for (config, compile_result, result_mutex) in e.values() {
//                     let mut ran_emvs = false;
//                     let mutex = result_mutex.try_lock().unwrap();
//                     if let RunResult::CheckResult(v) = mutex.0.as_ref().unwrap() {
//                         for check_result in v {
//                             if matches!(check_result, (PropType::Forge, CheckResult::Error(_))) {
//                                 for emv in error_message_versions.iter() {
//                                     let emvresult = config.run_error_msg(emv).unwrap();
//                                     progress.inc(1);
//                                     mutex.1.push((emv, emvresult));
//                                     ran_emvs = true;
//                                 }
//                             }
//                         }
//                     }

//                     if !ran_emvs {
//                         progress.inc(error_message_versions.len() as u64);
//                     }
//                 }
//             }
//         }

//         let receive_work = Arc::new(Mutex::new(receive_work));

//         for _ in 0..args.parallelism {
//             let my_receive = receive_work.clone();
//             let my_results_ref = &results;
//             let progress_ref = &progress;
//             std::thread::Builder::new()
//                 .spawn_scoped(scope, move || {
//                     while let Some((config, compile_result, mutex, emv)) =
//                         my_receive.lock().ok().and_then(|r| r.recv().ok())
//                     {
//                         let emvresult = config.run_error_msg(emv).unwrap();
//                         progress_ref.inc(1);
//                         mutex.lock().unwrap().1.push((emv, emvresult));
//                     }
//                 })
//                 .unwrap();
//         }
//     });

//     let mut csv = std::fs::OpenOptions::new()
//         .truncate(true)
//         .create(true)
//         .write(true)
//         .open("err-msg-stats.csv")
//         .unwrap();

//     writeln!(csv, "Property,Version,Articulation Point,Severity,Runtime,Sucess,Repair Type,Error Size,Graph Size,Labels Size").unwrap();

//     for type_results in results.values() {
//         for edit_results in type_results.values() {
//             for (config, compile_result, result) in edit_results.values() {
//                 let result = &result.try_lock().unwrap();
//                 let RunConfiguration {
//                     version: (version, ..),
//                     edit,
//                     ..
//                 } = config;
//                 if let Some(RunResult::CheckResult(v)) = &result.0 {
//                     for check_result in v {
//                         if matches!(check_result, (PropType::Forge, CheckResult::Error(_))) {
//                             for (emv, result) in result.1.iter() {
//                                 progress.suspend(|| {
//                                     writeln!(w, "{}: {emv} {result}", config.describe()).unwrap();
//                                 });
//                                 let Edit {
//                                     severity,
//                                     articulation_point,
//                                     property,
//                                 } = edit.expect("Must be edit");
//                                 let (success, runtime) = match result {
//                                     ErrMsgResult::Timeout => ("timeout", Duration::ZERO),
//                                     ErrMsgResult::Sat(t) => ("failed", *t),
//                                     ErrMsgResult::Success { runtime, .. } => ("yes", *runtime),
//                                 };
//                                 write!(csv, "{property},{version},{articulation_point},{severity},{},{success},{emv},", humantime::format_duration(runtime)).unwrap();
//                                 match result {
//                                     ErrMsgResult::Success {
//                                         payload: ErrMsgResultPayload::Markers(m),
//                                         ..
//                                     } => write!(csv, ",,{m}"),
//                                     ErrMsgResult::Success {
//                                         payload:
//                                             ErrMsgResultPayload::ErrGraph {
//                                                 regular_edges,
//                                                 error_edges,
//                                             },
//                                         ..
//                                     } => write!(csv, "{error_edges},{regular_edges},"),
//                                     _ => write!(csv, ",,,"),
//                                 }
//                                 .unwrap();
//                                 writeln!(csv).unwrap();
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     progress.finish_and_clear();
// }
