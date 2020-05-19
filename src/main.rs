mod commands;
mod core;
mod languages;
mod utils;

#[cfg(test)]
mod test_utils;

use std::fmt;

use git2::Repository;
use semver::Version;
use structopt::StructOpt;

use commands::*;

#[derive(Debug, StructOpt)]
struct Semantic {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long)]
    write: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Bump {},
    Version {},
    Notes {},
    Changelog {},
    Release {},
}

#[derive(Debug)]
pub enum SemanticError {
    BumpError,
    VersionError,
    NotesError,
    ChangelogError,
    ReleaseError,
    IOError,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Bump {
    None,
    Prerelease,
    // Build
    Patch,
    Minor,
    Major,
}

impl fmt::Display for Bump {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub type SemanticResult = Result<(), SemanticError>;

trait ToTag {
    fn to_tag(&self) -> String;
}

impl ToTag for Version {
    fn to_tag(&self) -> String {
        format!("v{}", self)
    }
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
