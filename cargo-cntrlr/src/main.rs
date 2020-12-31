// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2020 Branan Riley <me@branan.info>
//
// Portions of the argument parsing code are derived from Cargo, under
// the MIT license:
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

use anyhow::{anyhow, bail, Result};
use cargo::{
    core::{
        compiler::{BuildConfig, CompileMode, MessageFormat},
        features::maybe_allow_nightly_features,
        Workspace,
    },
    ops::{compile, init, new, CompileFilter, CompileOptions, NewOptions, Packages},
    util::{
        config::Config, important_paths::find_root_manifest_for_wd, interning::InternedString,
        paths::resolve_executable,
    },
};
use clap::{App, AppSettings, Arg, SubCommand};
use cntrlr_build::{Board, Flash};
use std::{
    env::current_dir,
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
};
use subprocess::{Exec, ExitStatus};
use tempfile::NamedTempFile;

const MAIN: &'static str = "#![no_std]
#![no_main]

use cntrlr::prelude::*;
use core::future::pending;

#[entry]
async fn main() -> ! {
    serial_1().enable(9600).unwrap();
    writeln!(serial_1(), \"Hello, World\").await.unwrap();

    // Hang forever once we've sent our message
    pending().await
}
";

const DEPS: &'static str = "[dependencies]
cntrlr = \"0.1.0\"

[build-dependencies]
cntrlr-build = \"0.1.0\"
";

const BUILD: &'static str = "
use cntrlr_build::configure_board;

fn main() {
    configure_board();
}
";

fn build_command(name: &'static str) -> App<'static, 'static> {
    SubCommand::with_name(name)
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::with_name("package")
                .short("p")
                .long("package")
                .takes_value(true)
                .multiple(true)
                .value_name("SPEC")
                .help("Package to build (see `cargo help pkgid`)"),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .help("Alias for --workspace (deprecated)"),
        )
        .arg(
            Arg::with_name("workspace")
                .long("workspace")
                .help("Build all packages in the workspace"),
        )
        .arg(
            Arg::with_name("exclude")
                .long("exclude")
                .takes_value(true)
                .multiple(true)
                .value_name("SPEC")
                .help("Exclude packages from the build"),
        )
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .long("jobs")
                .takes_value(true)
                .value_name("N")
                .help("Number of parallel jobs, defaults to # of CPUs"),
        )
        .arg(
            Arg::with_name("lib")
                .long("lib")
                .help("Build only the specified binary"),
        )
        .arg(
            Arg::with_name("bin")
                .long("bin")
                .takes_value(true)
                .multiple(true)
                .value_name("NAME")
                .help("Build only the specified binary"),
        )
        .arg(
            Arg::with_name("bins")
                .long("bins")
                .help("Build all binaries"),
        )
        .arg(
            Arg::with_name("example")
                .long("example")
                .takes_value(true)
                .multiple(true)
                .value_name("NAME")
                .help("Build only the specified example"),
        )
        .arg(
            Arg::with_name("examples")
                .long("examples")
                .help("Build all examples"),
        )
        .arg(
            Arg::with_name("test")
                .long("test")
                .takes_value(true)
                .multiple(true)
                .value_name("NAME")
                .help("Build only the specified test target"),
        )
        .arg(
            Arg::with_name("tests")
                .long("tests")
                .help("Build all tests"),
        )
        .arg(
            Arg::with_name("bench")
                .long("bench")
                .takes_value(true)
                .multiple(true)
                .value_name("NAME")
                .help("Build only the specified bench target"),
        )
        .arg(
            Arg::with_name("benches")
                .long("benches")
                .help("Build all benches"),
        )
        .arg(
            Arg::with_name("all-targets")
                .long("all-targets")
                .help("Build all targets"),
        )
        .arg(
            Arg::with_name("release")
                .long("release")
                .help("Build artifacts in release mode, with optimizations"),
        )
        .arg(
            Arg::with_name("profile")
                .long("profile")
                .takes_value(true)
                .value_name("PROFILE-NAME")
                .help("Build artifacts with the specified profile"),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .takes_value(true)
                .multiple(true)
                .value_name("FEATURE")
                .help("Space-separated list of features to activate"),
        )
        .arg(
            Arg::with_name("all-features")
                .long("all-features")
                .help("Activate all available features"),
        )
        .arg(
            Arg::with_name("no-default-features")
                .long("no-default-features")
                .help("Do not activate the `default` feature"),
        )
        .arg(
            Arg::with_name("target")
                .long("target")
                .takes_value(true)
                .multiple(true)
                .value_name("TRIPLE")
                .help("Build for the target triple"),
        )
        .arg(
            Arg::with_name("target-dir")
                .long("target-dir")
                .takes_value(true)
                .value_name("DIRECTORY")
                .help("Directory for all generated artifacts"),
        )
        .arg(
            Arg::with_name("manifest-path")
                .long("manifest-path")
                .takes_value(true)
                .value_name("PATH")
                .help("Path to Cargo.toml"),
        )
        .arg(
            Arg::with_name("message-format")
                .long("message-format")
                .takes_value(true)
                .value_name("FMT")
                .help("Error format"),
        )
        .arg(
            Arg::with_name("build-plan")
                .long("build-plan")
                .help("Output the build plan in JSON (unstable)"),
        )
        .arg(
            Arg::with_name("board")
                .long("board")
                .takes_value(true)
                .value_name("BOARD")
                .required(true)
                .help("Build for the target board. 'help' For the list of supported boards."),
        )
}

fn new_command(name: &'static str) -> App<'static, 'static> {
    SubCommand::with_name(name)
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(Arg::with_name("registry")
             .long("registry")
             .takes_value(true)
             .value_name("REGISTRY")
             .help("Registry to use"))
        .arg(Arg::with_name("vcs")
             .long("vcs")
             .takes_value(true)
             .possible_values(&["git", "hg", "pijul", "fossil", "none"])
             .value_name("VCS")
             .help("Initialize a new repository for the given version control system (git, hg, pijul, or fossil) or do not initialize any version control at all (none), overriding a global configuration."))
        .arg(Arg::with_name("bin")
             .long("bin")
             .help("Use a binary (application) template [default]"))
        .arg(Arg::with_name("lib")
             .long("lib")
             .help("Use a library template"))
        .arg(Arg::with_name("edition")
             .long("edition")
             .takes_value(true)
             .possible_values(&["2015", "2018"])
             .value_name("EDITION")
             .help("Edition to set for the crate generated [possible values: 2015, 2018]")
        )
        .arg(Arg::with_name("name")
             .long("name")
             .takes_value(true)
             .value_name("NAME")
             .help("Set the resulting package name, defaults to the directory name"))
        .arg(Arg::with_name("path")
             .takes_value(true)
             .value_name("PATH")
             .required(true))
}

fn main() -> Result<()> {
    maybe_allow_nightly_features();

    // Cleanup our executable name depending on how we were invoked
    let args = std::env::args().collect::<Vec<_>>();
    let args = if args[1] == "cntrlr" {
        &args[2..]
    } else {
        &args[1..]
    };

    let matches = App::new("cargo-cntrlr")
        .version("0.1.0")
        .author("Branan Riley <me@branan.info>")
        .about("Cargo subcommands for Cntrlr")
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::NoBinaryName)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .global(true)
                .help("Use verbose output (-vv very verbose/build.rs output)"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .global(true)
                .help("No output printed to stdout"),
        )
        .arg(
            Arg::with_name("color")
                .long("color")
                .takes_value(true)
                .possible_values(&["auto", "always", "never"])
                .value_name("WHEN")
                .global(true)
                .help("Coloring"),
        )
        .arg(
            Arg::with_name("frozen")
                .long("frozen")
                .global(true)
                .help("Require Cargo.lock and cache are up to date"),
        )
        .arg(
            Arg::with_name("locked")
                .long("locked")
                .global(true)
                .help("Require Cargo.lock is up to date"),
        )
        .arg(
            Arg::with_name("offline")
                .long("offline")
                .global(true)
                .help("Run without accessing the network"),
        )
        .arg(
            Arg::with_name("unstable_flags")
                .short("Z")
                .takes_value(true)
                .multiple(true)
                .value_name("FLAG")
                .global(true)
                .help("Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details"),
        )
        .subcommand(build_command("build").about("Compile the current package"))
        .subcommand(
            build_command("flash")
                .about("Flash a binary to a target board")
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .takes_value(true)
                        .value_name("PORT")
                        .help("The serial port the programmer is connected at, if needed"),
                ),
        )
        .subcommand(new_command("new").about("Create a new cntrlr package"))
        .subcommand(
            new_command("init").about("Create a new cntrlr package in an existing directory"),
        )
        .get_matches_from(args.iter());

    let (command, command_matches) = matches.subcommand();
    let command_matches = command_matches.ok_or(anyhow!("A subcommand is required"))?;

    let mut config = Config::default()?;
    let verbosity =
        matches.occurrences_of("verbose") as u32 + command_matches.occurrences_of("verbose") as u32;
    let quiet = matches.is_present("quiet") || command_matches.is_present("quiet");
    let color = command_matches
        .value_of("color")
        .or(matches.value_of("color"));
    let frozen = matches.is_present("frozen") || command_matches.is_present("frozen");
    let locked = matches.is_present("locked") || command_matches.is_present("locked");
    let offline = matches.is_present("offline") || command_matches.is_present("offline");
    let target_dir = command_matches.value_of("target-dir").map(PathBuf::from);
    let unstable_flags = matches
        .values_of("unstable_flags")
        .into_iter()
        .chain(command_matches.values_of("unstable_flags").into_iter())
        .flatten()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    config.configure(
        verbosity,
        quiet,
        color,
        frozen,
        locked,
        offline,
        &target_dir,
        &unstable_flags,
        &[],
    )?;

    if command == "new" || command == "init" {
        let version_control = command_matches
            .value_of("vcs")
            .map(str::parse)
            .transpose()?;
        let bin = command_matches.is_present("bin");
        let lib = command_matches.is_present("lib");
        let mut path: PathBuf = command_matches.value_of("path").unwrap().into();
        let name = command_matches.value_of("name").map(ToOwned::to_owned);
        let edition = command_matches.value_of("edition").map(ToOwned::to_owned);
        let registry = command_matches.value_of("registry").map(ToOwned::to_owned);

        if !path.is_absolute() {
            path = current_dir()?.join(path);
        }

        let opts = NewOptions::new(
            version_control,
            bin,
            lib,
            path.clone(),
            name,
            edition,
            registry,
        )?;
        if command == "new" {
            new(&opts, &config)?;
        } else {
            init(&opts, &config)?;
        }

        let manifest_path = path.join("cargo.toml");
        let mut manifest = String::new();
        OpenOptions::new()
            .read(true)
            .open(&manifest_path)?
            .read_to_string(&mut manifest)?;
        manifest = manifest.replace("[dependencies]", DEPS);
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&manifest_path)?
            .write_all(manifest.as_bytes())?;

        let build_path = path.join("build.rs");
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&build_path)?
            .write_all(BUILD.as_bytes())?;

        if !lib {
            let main_path = path.join("src").join("main.rs");
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&main_path)?
                .write_all(MAIN.as_bytes())?;
        }
        return Ok(());
    }

    let manifest_path = if let Some(path) = command_matches.value_of("manifest_path") {
        PathBuf::from(path)
    } else {
        find_root_manifest_for_wd(&current_dir()?)?
    };

    let workspace = Workspace::new(&manifest_path, &config)?;

    let all_features = command_matches.is_present("all-features");
    let no_default_features = command_matches.is_present("no-default-features");
    let all = command_matches.is_present("all") || command_matches.is_present("workspace");
    let exclude = command_matches
        .values_of("exclude")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let package = command_matches
        .values_of("package")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let mut compile_options = CompileOptions::new(&config, CompileMode::Build)?;
    compile_options.features = command_matches
        .values_of("features")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let lib_only = command_matches.is_present("lib");
    let bins = command_matches
        .values_of("bin")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let all_bins = command_matches.is_present("bins");
    let tsts = command_matches
        .values_of("test")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let all_tsts = command_matches.is_present("tests");
    let exms = command_matches
        .values_of("example")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let all_exms = command_matches.is_present("examples");
    let bens = command_matches
        .values_of("bench")
        .into_iter()
        .flatten()
        .map(ToOwned::to_owned)
        .collect();
    let all_bens = command_matches.is_present("benches");
    let all_targets = command_matches.is_present("all-targets");
    compile_options.all_features = all_features;
    compile_options.no_default_features = no_default_features;
    compile_options.spec = Packages::from_flags(all, exclude, package)?;
    compile_options.filter = CompileFilter::from_raw_arguments(
        lib_only,
        bins,
        all_bins,
        tsts,
        all_tsts,
        exms,
        all_exms,
        bens,
        all_bens,
        all_targets,
    );

    let board_name = command_matches
        .value_of("board")
        .ok_or(anyhow!("Board not specified"))?;

    if board_name == "help" {
        println!("arduino_uno");
        println!("teensy_30");
        println!("teensy_32");
        println!("teensy_35");
        println!("teensy_36");
        println!("teensy_40");
        println!("teensy_41");
        println!("teensy_lc");
        println!("red_v");
        return Ok(());
    }

    let board: Board = board_name
        .parse()
        .map_err(|_| anyhow!("Invalid board specified"))?;
    let requested_targets = command_matches
        .values_of("target")
        .map(|targets| {
            targets
                .into_iter()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![board.targets[0].to_owned()]);

    let jobs = command_matches
        .value_of("jobs")
        .map(str::parse)
        .transpose()?;

    let mut build_config = BuildConfig::new(&config, jobs, &requested_targets, CompileMode::Build)?;
    let mut message_format = None;
    let default_json = MessageFormat::Json {
        short: false,
        ansi: false,
        render_diagnostics: false,
    };
    for fmt in command_matches
        .values_of("message_format")
        .into_iter()
        .flatten()
    {
        for fmt in fmt.split(',') {
            let fmt = fmt.to_ascii_lowercase();
            match fmt.as_str() {
                "json" => {
                    if message_format.is_some() {
                        bail!("cannot specify two kinds of `message-format` arguments");
                    }
                    message_format = Some(default_json);
                }
                "human" => {
                    if message_format.is_some() {
                        bail!("cannot specify two kinds of `message-format` arguments");
                    }
                    message_format = Some(MessageFormat::Human);
                }
                "short" => {
                    if message_format.is_some() {
                        bail!("cannot specify two kinds of `message-format` arguments");
                    }
                    message_format = Some(MessageFormat::Short);
                }
                "json-render-diagnostics" => {
                    if message_format.is_none() {
                        message_format = Some(default_json);
                    }
                    match &mut message_format {
                        Some(MessageFormat::Json {
                            render_diagnostics, ..
                        }) => *render_diagnostics = true,
                        _ => bail!("cannot specify two kinds of `message-format` arguments"),
                    }
                }
                "json-diagnostic-short" => {
                    if message_format.is_none() {
                        message_format = Some(default_json);
                    }
                    match &mut message_format {
                        Some(MessageFormat::Json { short, .. }) => *short = true,
                        _ => bail!("cannot specify two kinds of `message-format` arguments"),
                    }
                }
                "json-diagnostic-rendered-ansi" => {
                    if message_format.is_none() {
                        message_format = Some(default_json);
                    }
                    match &mut message_format {
                        Some(MessageFormat::Json { ansi, .. }) => *ansi = true,
                        _ => bail!("cannot specify two kinds of `message-format` arguments"),
                    }
                }
                s => bail!("invalid message format specifier: `{}`", s),
            }
        }
    }
    build_config.message_format = message_format.unwrap_or(MessageFormat::Human);
    build_config.build_plan = command_matches.is_present("build-plan");
    build_config.requested_profile = if let Some(profile) = command_matches.value_of("profile") {
        InternedString::new(profile)
    } else if command_matches.is_present("release") {
        InternedString::new("release")
    } else {
        InternedString::new("dev")
    };
    compile_options.build_config = build_config;

    std::env::set_var("CNTRLR_BOARD", board_name);
    let out = compile(&workspace, &compile_options)?;

    if command == "flash" {
        if out.binaries.len() != 1 {
            bail!("A single binary must be built in order to flash to a board");
        }
        let binary = out.binaries[0]
            .1
            .to_str()
            .ok_or(anyhow!("Binary path is not UTF-8"))?;
        match board.flash {
            Flash::AvrDude(programmer) => {
                let avrdude = resolve_executable(&PathBuf::from("avrdude"))?;
                let flash = format!("-Uflash:w:{}", binary);
                let port = command_matches
                    .value_of("port")
                    .ok_or(anyhow!("--port is required to program this board"))?;
                let status = Exec::cmd(avrdude)
                    .arg("-p")
                    .arg(board.mcu)
                    .arg("-c")
                    .arg(programmer)
                    .arg("-P")
                    .arg(port)
                    .arg(flash)
                    .join()?;
                if status != ExitStatus::Exited(0) {
                    bail!("avrdude error");
                }
            }
            Flash::TeensyLoader => {
                let objcopy = resolve_executable(&PathBuf::from("arm-none-eabi-objcopy"))?;
                let teensyloader = resolve_executable(&PathBuf::from("teensy_loader_cli"))
                    .or_else(|_| resolve_executable(&PathBuf::from("teensy-loader-cli")))?;
                let mcu = format!("--mcu={}", board.mcu);
                let hex = format!("{}.hex", binary);
                let status = Exec::cmd(objcopy)
                    .arg("-O")
                    .arg("ihex")
                    .arg(&binary)
                    .arg(&hex)
                    .join()?;
                if status != ExitStatus::Exited(0) {
                    bail!("objcopy error");
                }

                let status = Exec::cmd(teensyloader)
                    .arg("-w")
                    .arg(&hex)
                    .arg(&mcu)
                    .join()?;
                if status != ExitStatus::Exited(0) {
                    bail!("teensy-loader-cli error");
                }
            }
            Flash::OpenOcd(cfg) => {
                let openocd = resolve_executable(&PathBuf::from("openocd"))?;
                let mut cfg_file = NamedTempFile::new()?;
                writeln!(
                    cfg_file,
                    "source [find {}]
                     flash write_image erase {}
                     reset run
                     shutdown",
                    cfg, binary
                )?;
                let status = Exec::cmd(openocd).arg("-f").arg(cfg_file.path()).join()?;
                if status != ExitStatus::Exited(0) {
                    bail!("OpenOCD error");
                }
            }
        }
    }
    Ok(())
}
