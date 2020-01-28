extern crate config;

use structopt::StructOpt;
use std::collections::HashMap;
use config::Value;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    #[structopt(default_value = "foobar", long)]
    pattern: String,
    /// The path to the file to read
    #[structopt(parse(from_os_str), default_value = "config", long)]
//    #[structopt(default_value = "foobar", String)]
    path: std::path::PathBuf,
}



fn main() {
    let args = Cli::from_args();
    println!("Hello, world!");
    println!("Pattern {}", &args.pattern);
    println!("Path: {:?}", &args.path.display());

    let mut settings = config::Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings")).unwrap();

    // Print out our settings (as a HashMap)
    println!("{:?}",
             settings.try_into::<HashMap<String, Vec<Value>>>().unwrap());
}

