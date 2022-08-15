use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::new(&args) {
        Ok(config) => config,
        Err(err) => {
            print_help();
            println!("\nError happened parsing args:\n\t{err}");
            return;
        } 
    };

    if config.help {
        print_help();
        return;
    }

    for i in config.filenames {
        print!("{i} ");
    }
    println!("");

}

struct Config {
    filenames: Vec<String>,
    help: bool
}

impl Config {
    fn new(args: &[String]) -> Result<Config, String> {
        let mut filenames: Vec<String> = Vec::new();
        for i in args {
            if !i.starts_with("--") {
                if !Path::new(i).exists() {
                    return Result::Err(format!("Given filepath {} do not exist", i))
                }
                filenames.push(i.to_string());
            }

            else {
                if i=="--help" {
                    return Result::Ok(Config { filenames, help: true })
                }
            }
        }

        Result::Ok(Config { filenames, help: false })
    }
}

fn print_help() {
    println!("ptfl_reader [files] [--help]");
    println!("    files: one or multiple file path as input");
    println!("    help: print this message");
}