mod structure;
mod japanese;
mod transcribe;
mod language;
mod stats;
mod report;

use structure::*;
use transcribe::*;
use language::*;
use stats::*;

use clap::Parser;

use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
    #[clap(short='m', long, value_enum, default_value_t=Mode::default())]
    mode: Mode,
    #[clap(short='o', long, value_enum, default_value_t=OutputMode::default())]
    outputmode: OutputMode,
    #[clap(short='l', long, default_value_t=true)]
    log: bool,
    #[clap(short='d', long, value_parser)]
    outputdir: Option<PathBuf>,
    #[clap(required = true)]
    inputfiles: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, clap::ValueEnum)]
enum Mode { #[default] Transcribe, Stats, Language }

#[derive(Debug, Default, Clone, Copy, PartialEq, clap::ValueEnum)]
enum OutputMode { #[default] Stdout, File }

fn main() {
    let args = Args::parse();
    if args.inputfiles.is_empty(){
        println!("No input files received!");
        return;
    }
    let mut log = String::new();
    let mut doc = String::new();

    let mut fileroot = args.inputfiles[0].clone();
    let mut chapters = args.inputfiles.into_iter()
        .map(|f| (get_chapter(&f), f))
        .filter(|(c, _)| c.is_some())
        .map(|(c, f)| (c.unwrap(), f))
        .collect::<Vec<_>>();
    chapters.sort_by(|a, b|
        a.0.volume.cmp(&b.0.volume)
        .then(a.0.chapter.cmp(&b.0.chapter))
        .then(a.0.subchapter.unwrap_or(0.0)
              .partial_cmp(&b.0.subchapter.unwrap_or(0.0)).unwrap())
    );

    match args.mode{
        Mode::Transcribe => {
            for (chapter, file) in chapters{
                doc.clear();
                write_transcription(chapter, &mut doc, &mut log);
                write_output(args.outputmode, &args.outputdir, file, &doc);
            }
        },
        Mode::Stats => {
            let mut stats = Stats::default();
            for (chapter, _) in chapters{
                accumulate_stats(chapter, &mut stats, &mut log);
            }
            fileroot.set_file_name("stats");
            stats_report(stats, &mut doc);
            write_output(args.outputmode, &args.outputdir, fileroot, &doc);
        },
        Mode::Language => {
            let mut stats = LangStats::default();
            for (chapter, _) in chapters{
                accumulate_lang_stats(chapter, &mut stats, &mut log);
            }
            fileroot.set_file_name("stats");
            lang_stats_report(stats, &mut doc);
            write_output(args.outputmode, &args.outputdir, fileroot, &doc);
        },
    }
    if args.log {
        println!("{}", log);
    }
}

fn get_chapter(file: &PathBuf) -> Option<Chapter>{
    let contents = match fs::read_to_string(&file){
        Ok(contents) => contents,
        Err(error) => {
            println!("Could not read file: \"{}\".\n\tError: {}", file.display(), error);
            return None;
        }
    };
    match toml::from_str::<Chapter>(&contents){
        Ok(chapter) => Some(chapter),
        Err(error) => panic!("{} (error position is an estimation!)", error),
    }
}

fn write_output(outputmode: OutputMode, outputdir: &Option<PathBuf>, mut file: PathBuf, doc: &str){
    if outputmode == OutputMode::File{
        if let Some(outdir) = outputdir{
            let filename = file.file_name().expect("rip").to_os_string();
            file.clear();
            file.push(outdir);
            file.push(filename);
        }
        file.set_extension("md");
        let mut outfile = match fs::File::create(&file){
            Ok(outfile) => outfile,
            Err(error) => {
                println!("Could not create file: \"{}\".\n\tError: {}", file.display(), error);
                return;
            }
        };
        if let Err(error) = write!(outfile, "{}", doc){
            println!("Could not write to file: \"{}\".\n\tError: {}", file.display(), error);
        }
    } else {
        println!("{}", doc);
    }
}

