use std::env;
use std::fs;
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

    let mut point_files: Vec<(String, Vec<(f64, f64)>)> = Vec::new();
    for path in config.filenames {
        let file = match fs::read_to_string(&path) {
            Ok(file) => file,
            Err(_) => {
                println!("Error happened parsing file {path}: ");
                println!("\tFailed reading file");
                return;
            }
        };

        enum ParsingState {
            None,
            DuringReg(usize),
        }

        let file_entry: u32 = 0;
        let mut my_state = ParsingState::None;
        let mut current_reg: Vec<(f64, f64)> = Vec::new();
        for line in file.lines() {
            my_state = match my_state {
                ParsingState::None => {
                    if line.is_empty() {
                        ParsingState::None
                    } else {
                        let reg_length: usize = match line.parse() {
                            Ok(length) => length,
                            Err(_) => {
                                println!("Error happened parsing file {path}: ");
                                println!("\tExpected u32 or empty when None");
                                return;
                            }
                        };
                        current_reg.clear();
                        current_reg.reserve(reg_length);
                        ParsingState::DuringReg(reg_length)
                    }
                }
                ParsingState::DuringReg(remaining) => {
                    let next: usize = remaining - 1;
                    let reg: Vec<&str> = line.split(',').collect();
                    let reg: (f64, f64) = (reg[0].trim().parse().expect(""), reg[1].trim().parse().expect(""));
                    current_reg.push(reg);
                    if next == 0 {
                        point_files.push((format!("{path}-{file_entry}"), current_reg.clone()));
                        ParsingState::None
                    } else {
                        ParsingState::DuringReg(remaining - 1)
                    }
                }
            }
        }
        println!("Currently {} regs!", point_files.len());
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
        for i in &args[1..] {
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