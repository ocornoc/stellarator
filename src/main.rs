use clap::{Parser, Subcommand, Args};

mod stel;

#[derive(Args)]
struct Parse {
    #[clap(long, short = 'r')]
    recursive: bool,
    path: std::path::PathBuf,
}

impl Parse {
    fn print_file(path: &std::path::Path) {
        let bytes = std::fs::read(path).unwrap();
        let parsed = stel::StelData::parse(&bytes).unwrap();
        println!("Data for {}:", path.to_string_lossy());
        println!("{parsed}");
    }

    fn recursive(self) {
        use std::ffi::OsStr;

        let mut files = Vec::new();
        let mut directories = Vec::new();
        if self.path.is_dir() {
            directories.push(self.path.canonicalize().unwrap());
        } else if self.path.extension() == Some(OsStr::new("stel")) {
            files.push(self.path.canonicalize().unwrap());
        }
        while let Some(directory) = directories.pop() {
            // skip .git directories
            if directory.file_name() == Some(OsStr::new(".git")) {
                continue;
            }

            for path in std::fs::read_dir(directory).unwrap() {
                let path = path.unwrap().path().canonicalize().unwrap();
                if path.is_dir() {
                    directories.push(path);
                } else if path.extension() == Some(OsStr::new("stel")) {
                    files.push(path);
                }
            }
        }
        for file in files {
            Parse::print_file(&file);
        }
    }

    fn specific(self) {
        Parse::print_file(&self.path);
    }

    fn execute(self) {
        if self.recursive {
            self.recursive()
        } else {
            self.specific()
        }
    }
}

#[derive(Subcommand)]
enum Command {
    Parse(Parse),
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() {
    match Cli::parse().command {
        Command::Parse(parse) => parse.execute(),
    }
}
