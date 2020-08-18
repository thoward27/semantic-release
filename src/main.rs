use semantic_release::*;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Semantic {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long)]
    write: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Compute and display the suggested version bump.
    Bump {},

    /// Set the project version to the next suggested.
    Version {},

    /// Create release notes.
    Notes {},

    /// Generate a changelog.
    Changelog {},

    /// Build an entire release.
    Release {},
}

fn main() {
    log::debug!("start");
    let args = Semantic::from_args();
    simple_logger::init_with_level(if args.debug {
        log::Level::Debug
    } else {
        log::Level::Error
    })
    .unwrap();
    let repo = Repository::open(".").unwrap();
    let result = match args.cmd {
        Command::Bump {} => bump(repo),
        Command::Version {} => version(repo),
        Command::Notes {} => notes(repo),
        Command::Changelog {} => changelog(repo),
        Command::Release {} => release(repo),
    };
    match result {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    }
}
