use anyhow::bail;
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use log::{LevelFilter, trace};
use rand::{
    Rng, SeedableRng,
    distr::{Distribution, Uniform},
    rngs::SmallRng,
    seq::IndexedRandom,
};

#[derive(Parser)]
struct Cli {
    /// The file to write the generated GFA graph to.
    /// If not provided or set to '-', the graph will be written to stdout.
    #[clap(short = 'o', long, default_value = "-")]
    output_file: PathBuf,

    /// The number of nodes in the generated GFA graph.
    #[clap(short = 'n', long)]
    node_count: usize,

    /// The number of edges in the generated GFA graph.
    #[clap(short = 'e', long)]
    edge_count: usize,

    /// If set, the graph will be strongly connected.
    /// If not set, it may still be strongly connected by chance.
    #[clap(short = 'c', long)]
    ensure_strongly_connected: bool,

    /// The seed for the random number generator. If not provided, a random seed will be used.
    #[clap(short = 's', long)]
    seed: Option<u64>,

    /// The log level to use. Can be one of "error", "warn", "info", "debug", or "trace". Default is "info".
    #[clap(short = 'l', long, default_value = "info")]
    log_level: LevelFilter,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    initialise_logger(cli.log_level);

    if cli.ensure_strongly_connected && cli.edge_count < cli.node_count {
        bail!("Cannot ensure strong connectivity with fewer edges than nodes.");
    }

    let mut output = BufWriter::new(if &cli.output_file == "-" {
        Box::new(io::stdout()) as Box<dyn Write>
    } else {
        Box::new(File::create(cli.output_file)?) as Box<dyn Write>
    });
    let mut rng = SmallRng::seed_from_u64(cli.seed.unwrap_or_else(rand::random));

    trace!("Writing header");
    writeln!(output, "H\tVN:Z:1.0")?;

    trace!("Writing nodes");
    for node_id in 1..=cli.node_count {
        let sequence = random_dna_string(&mut rng, Uniform::new_inclusive(5, 15).unwrap());
        writeln!(output, "S\t{}\t{}", node_id, sequence)?;
    }

    trace!("Writing edges");
    let edge_count = if cli.ensure_strongly_connected {
        trace!("Ensuring strong connectivity by creating a cycle through all nodes");
        for i in 1..cli.node_count {
            writeln!(output, "L\t{}\t+\t{}\t+\t0M", i, i + 1)?;
        }
        writeln!(output, "L\t{}\t+\t1\t+\t0M", cli.node_count)?;

        trace!("Writing remaining edges");
        cli.edge_count - cli.node_count
    } else {
        cli.edge_count
    };

    let random_node = Uniform::new_inclusive(1, cli.node_count).unwrap();
    let signs = ['+', '-'];
    for _ in 0..edge_count {
        let from = random_node.sample(&mut rng);
        let to = random_node.sample(&mut rng);
        let from_sign = signs.choose(&mut rng).unwrap();
        let to_sign = signs.choose(&mut rng).unwrap();
        writeln!(
            output,
            "L\t{}\t{}\t{}\t{}\t0M",
            from, from_sign, to, to_sign
        )?;
    }

    Ok(())
}

fn random_dna_string(rng: &mut impl Rng, length_distribution: Uniform<usize>) -> String {
    let length = length_distribution.sample(rng);

    let nucleotides = ['A', 'C', 'G', 'T'];
    nucleotides.choose_iter(rng).unwrap().take(length).collect()
}

fn initialise_logger(log_level: LevelFilter) {
    simplelog::TermLogger::init(
        log_level,
        Default::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
}
