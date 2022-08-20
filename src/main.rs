use indexmap::IndexMap;
use ptfl_reader::Config;
use ptfl_reader::PtflParser;
use std::cmp::Ordering;
use std::env;
use std::io;
use std::io::Write;

fn main() {
    // process command line arguments
    let args: Vec<String> = env::args().collect();
    let config = match Config::new(&args) {
        Ok(config) => config,
        Err(err) => {
            print_args_help();
            println!("\nError happened parsing args:\n\t{err}");
            return;
        }
    };

    if config.help {
        print_args_help();
        return;
    }

    // parse and load files specified in command line
    let mut point_files: IndexMap<(String, u32), Vec<(f64, f64)>> = IndexMap::new();
    let mut parser = PtflParser::new();
    for path in config.filenames {
        match parser.parse(path.as_str(), &mut point_files) {
            Ok(count) => {
                println!("Read {count} from {}.", path.as_str());
                println!("Currently {} regs!", point_files.len())
            }
            Err(err) => {
                // skip the file error occur
                // and reset the state of parser
                println!("Error happened parsing file {path}: ");
                println!("\t{err}");
                parser.renew();
            }
        }
    }

    tui_loop(point_files);
}

fn print_args_help() {
    println!("ptfl_reader [files] [--help]");
    println!("    files: one or multiple file path as input");
    println!("    help: print this message");
}

// the main loop of tui interface
fn tui_loop(mut point_files: IndexMap<(String, u32), Vec<(f64, f64)>>) {
    loop {
        // prompt the user to input something
        print!("> ");
        io::stdout().flush().unwrap();

        // read in the command
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Reading line from terminal failed");

        // break down the input
        let input: Vec<&str> = input.trim().split(' ').collect();
        let command = input[0];

        match command.len() {
            0 => continue,
            4 => {
                if command == "exit" {
                    break;
                } else if command == "list" {
                    println!("Listing entries:");
                    for i in &point_files {
                        println!("\t{}-{:04}: {:8} points", i.0 .0, i.0 .1, i.1.len());
                    }
                    println!("");
                } else if command == "load" {
                    fn prompt() {
                        println!("load filename1 [filename2] [filename3] ...");
                        println!("");
                    }
                    if input.len() < 2 {
                        prompt();
                    } else {
                        let mut parser = PtflParser::new();
                        for filename in &input[1..] {
                            match parser.parse(&filename, &mut point_files) {
                                Ok(count) => {
                                    println!("Read {count} from {}.", filename);
                                    println!("Currently {} regs!", point_files.len())
                                }
                                Err(err) => {
                                    // skip the file error occur
                                    // and reset the state of parser
                                    println!("Error happened parsing file {filename}: ");
                                    println!("\t{err}");
                                    parser.renew();
                                }
                            }
                        }
                    }
                } else if command == "show" {
                    fn prompt() {
                        println!("show entry_name entry_num");
                        println!("");
                    }
                    if input.len() != 3 {
                        prompt();
                    } else {
                        let key = (
                            input[1].to_string(),
                            match input[2].parse::<u32>() {
                                Ok(entry_num) => entry_num,
                                Err(err) => {
                                    prompt();
                                    println!(
                                        "Error happened parsing entry_num: \n\t{}",
                                        err.to_string()
                                    );
                                    continue;
                                }
                            },
                        );

                        match point_files.get(&key) {
                            Some(entry) => {
                                println!("Yes, {}-{:04} has {} points", &key.0, &key.1, entry.len())
                            }
                            None => println!("No, {}-{:04} not found", &key.0, &key.1),
                        }
                    }
                } else {
                    print_tui_help();
                }
            }
            7 => {
                if command == "combine" {
                    fn prompt() {
                        println!("combine target_name target_num");
                        println!("\tentry_name entry_num");
                        println!("\tentry_name entry_num");
                        println!("\t...");
                        println!("");
                    }
                    if input.len() != 3 {
                        prompt()
                    } else {
                        let key = (
                            input[1].to_string(),
                            match input[2].parse::<u32>() {
                                Ok(target_num) => target_num,
                                Err(err) => {
                                    prompt();
                                    println!(
                                        "Error happened parsing target_num: \n\t{}",
                                        err.to_string()
                                    );
                                    continue;
                                }
                            },
                        );

                        if point_files.contains_key(&key) {
                            prompt();
                            println!("Entry {}-{:04} already exist!", input[1], input[2]);
                            continue;
                        }

                        let mut combined_entry: Vec<(f64, f64)> = Vec::new();
                        for key in tui_get_entry_keys(&point_files, prompt) {
                            // get entry names ensure the keys are valid so we can safely unwrap
                            combined_entry.append(&mut (point_files.get(&key).unwrap().clone()));
                        }
                        combined_entry.sort_by(|a, b| {
                            if a.0 > b.0 {
                                Ordering::Greater
                            } else if a.0 == b.0 {
                                if a.1 > b.1 {
                                    Ordering::Greater
                                } else {
                                    if a.1 == b.1 {
                                        Ordering::Equal
                                    } else {
                                        Ordering::Less
                                    }
                                }
                            } else {
                                Ordering::Less
                            }
                        });
                        point_files.insert(key, combined_entry);
                    }
                } else {
                    print_tui_help();
                }
            }
            _ => {
                print_tui_help();
            }
        }
    }
}

fn print_tui_help() {
    println!("combine:\tcombine multiple entry into a new entry");
    println!("exit:\t\texit the program");
    println!("help:\t\tprint this message");
    println!("list:\t\tlist all entries with ammount of contained points");
    println!("load:\t\tread and parse a file to pointfiles");
    println!("show:\t\tcheck if a entry exists");
}

fn tui_get_entry_keys(
    point_files: &IndexMap<(String, u32), Vec<(f64, f64)>>,
    prompt: fn(),
) -> Vec<(String, u32)> {
    let mut entry_keys: Vec<(String, u32)> = Vec::new();
    loop {
        // prompt the user they are entering entries for combination
        print!("\t");
        io::stdout().flush().unwrap();

        // read in the entry name
        let mut entry_input = String::new();
        io::stdin()
            .read_line(&mut entry_input)
            .expect("Reading line from terminal failed");

        // break down the input
        let entry_input: Vec<&str> = entry_input.trim().split(' ').collect();
        if entry_input.len() == 1 && entry_input[0].len() == 0 {
            break;
        }

        if entry_input.len() != 2 {
            prompt();
            continue;
        }

        // turn the input into a key
        let entry_key = (
            entry_input[0].to_string(),
            match entry_input[1].parse::<u32>() {
                Ok(entry_num) => entry_num,
                Err(err) => {
                    prompt();
                    println!("Error happened parsing entry_num: \n\t{}", err.to_string());
                    continue;
                }
            },
        );

        // push if the entry exist in point_files
        if point_files.contains_key(&entry_key) {
            entry_keys.push(entry_key);
        } else {
            prompt();
            println!("Entry {}-{:04} didn't exist!", entry_key.0, entry_key.1);
            continue;
        }
    }
    entry_keys
}
