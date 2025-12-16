use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};
use emulator::Emulator;
use fuzzer::Fuzzer;
use modeling::input::InputFile;

use crate::cli;

#[derive(Debug)]
pub struct HoedurConfig {
    name: String,
    pub(crate) output_dir: PathBuf,
    seed: Option<u64>,
    pub(crate) prefix_input: Vec<PathBuf>,
    import_corpus: Vec<PathBuf>,
    snapshots: bool,
    statistics: bool,
}

impl HoedurConfig {
    pub fn new(
        name: String,
        output_dir: PathBuf,
        seed: Option<u64>,
        prefix_input: Vec<PathBuf>,
        import_corpus: Vec<PathBuf>,
        snapshots: bool,
        statistics: bool,
    ) -> Self {
        Self {
            name,
            output_dir,
            seed,
            prefix_input,
            import_corpus,
            snapshots,
            statistics,
        }
    }

    pub fn from_cli(name: String, args: &cli::HoedurArguments) -> Result<Self> {
        let output_dir = std::env::current_dir()?.join("hoedur-project");
        let seed = if let Some(seed_file) = args.seed.clone() {
            // random seed from file
            let mut seed = [0u8; 8];

            match File::open(&seed_file)
                .with_context(|| format!("Failed to open seed file {:?}", &seed_file))?
                .read_exact(&mut seed)
            {
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    log::warn!("seed too short, filling with zero");
                }
                Err(e) => {
                    Err(e).with_context(|| format!("Failed to load seed from {:?}", &seed_file))?
                }
            }

            Some(u64::from_be_bytes(seed))
        } else {
            None
        };

        Ok(Self::new(
            name,
            output_dir,
            seed,
            args.prefix.prefix_input.clone(),
            args.import_corpus.clone(),
            args.snapshots,
            args.statistics.statistics,
        ))
    }
}

pub(crate) fn run_fuzzer(emulator: Emulator<InputFile>, config: HoedurConfig) -> Result<()> {
    Fuzzer::new(
        config.name,
        config.seed,
        config.import_corpus,
        config.statistics,
        config.snapshots,
        config.output_dir,
        emulator,
    )?
    .run()
}
