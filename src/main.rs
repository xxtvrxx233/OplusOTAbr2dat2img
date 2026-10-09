//! Command line front end for the `br2img` library.
//!
//! Built with `--features cli`; without it the crate ships as a library only.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use br2img::{Job, NewData, Options, Progress, Report, SizeCheck, TransferList};

/// Turn an Android block-OTA payload into a raw partition image.
///
/// `br2img system.new.dat.br` decompresses the payload, lays its blocks out the
/// way `system.transfer.list` describes and writes `system.img` next to it.
#[derive(Debug, Parser)]
#[command(name = "br2img", version, about, long_about = None)]
struct Cli {
    /// Payload to convert: `<name>.new.dat.br`, or an already decompressed
    /// `<name>.new.dat`.
    dat: PathBuf,

    /// Transfer list to apply. Defaults to `<name>.transfer.list` next to DAT.
    #[arg(short, long, value_name = "PATH")]
    transfer_list: Option<PathBuf>,

    /// Image to write. Defaults to `<name>.img` next to DAT.
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Overwrite the image if it is already there.
    #[arg(short, long)]
    force: bool,

    /// Skip the consistency checks between header, commands and payload.
    #[arg(long)]
    no_verify: bool,

    /// A `dynamic_partitions_op_list` to cross-check the image size against.
    // Oplus device only option.
    #[arg(long, value_name = "PATH")]
    op_list: Option<PathBuf>,

    /// Only print the summary, no progress bar.
    #[arg(short, long)]
    quiet: bool,

    /// Print what the transfer list says before converting.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match convert(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("br2img: error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn convert(cli: &Cli) -> br2img::Result<()> {
    let mut job = Job::from_dat(&cli.dat)?;
    if let Some(path) = cli.transfer_list.clone() {
        job.transfer_list = path;
    }
    if let Some(path) = cli.output.clone() {
        job.output = path;
    }

    let options = Options {
        verify: !cli.no_verify,
        overwrite: cli.force,
        op_list: cli.op_list.clone(),
    };

    if cli.verbose {
        describe(&job, &options)?;
    }

    let mut reporter = Reporter::new(cli.quiet);
    let report = match job.run(&options, &mut reporter) {
        Ok(report) => report,
        Err(error) => {
            reporter.abandon();
            return Err(error);
        }
    };

    summarize(&job, &report);
    Ok(())
}

/// Print what the transfer list says, without writing anything.
fn describe(job: &Job, options: &Options) -> br2img::Result<()> {
    let list = TransferList::from_path(&job.transfer_list)?;
    let payload = NewData::new(&job.dat);
    let zeroed = list.covered_blocks() - list.payload_blocks();

    println!(
        "transfer list : {} (version {}, {})",
        job.transfer_list.display(),
        list.version(),
        list.android_release()
    );
    println!(
        "payload       : {}{}",
        job.dat.display(),
        if payload.is_compressed() {
            ", brotli compressed"
        } else {
            ", uncompressed"
        }
    );
    println!("output        : {}", job.output.display());
    println!(
        "commands      : {} ({} blocks copied, {} blocks zeroed)",
        list.commands().len(),
        list.payload_blocks(),
        zeroed
    );
    println!(
        "image         : {} blocks, {}",
        list.image_blocks(),
        human_bytes(list.image_bytes())
    );
    println!(
        "checks        : {}",
        if options.verify { "on" } else { "off" }
    );
    Ok(())
}

/// Report what the conversion produced.
fn summarize(job: &Job, report: &Report) {
    println!(
        "{}: {} blocks ({}) in {:.2}s",
        job.output.display(),
        report.image_blocks,
        human_bytes(report.image_bytes),
        report.elapsed.as_secs_f64()
    );
    println!(
        "  {} blocks copied from {} ({} bytes on disk), {} blocks left zero",
        report.blocks_copied,
        file_name(&job.dat),
        report.dat_bytes,
        report.blocks_zeroed
    );

    match &report.size_check {
        Some(SizeCheck::Match { partition, size }) => {
            println!("  size agrees with `{partition}` in the op list ({size})");
        }
        Some(SizeCheck::Mismatch {
            partition,
            expected,
            actual,
        }) => eprintln!(
            "br2img: warning: the op list has `{partition}` at {expected} but the image is {actual} \
             (op list sizes are bytes on most devices and 512-byte sectors on a few)"
        ),
        Some(SizeCheck::Unknown) => eprintln!(
            "br2img: warning: the op list has no `resize` for `{}`",
            job.name().unwrap_or_else(|| file_name(&job.dat))
        ),
        None => {}
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// e.g. `1536` to `1.50 KiB`.
fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

/// Draws a progress bar on stderr, unless the user asked for quiet output.
struct Reporter {
    bar: ProgressBar,
    quiet: bool,
}

impl Reporter {
    fn new(quiet: bool) -> Self {
        let bar = ProgressBar::new(0);
        if let Ok(style) = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} blocks ({eta})",
        ) {
            bar.set_style(style.progress_chars("=> "));
        }
        Self { bar, quiet }
    }

    /// Take the bar off the screen; used when the conversion failed.
    fn abandon(&self) {
        self.bar.finish_and_clear();
    }
}

impl Progress for Reporter {
    fn begin(&mut self, total_blocks: u64) {
        self.bar.set_length(total_blocks);
        if !self.quiet {
            self.bar.enable_steady_tick(Duration::from_millis(120));
        }
    }

    fn update(&mut self, done: u64) {
        self.bar.set_position(done);
    }

    fn finish(&mut self) {
        self.bar.finish_and_clear();
    }
}
