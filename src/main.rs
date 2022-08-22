use indexmap::IndexMap;
use ptfl_reader::Config;
use ptfl_reader::PNGOutput;
use ptfl_reader::PtflParser;
use ptfl_reader::SVGOutput;
use ptfl_reader::TevWrappedClient;
use rayon::prelude::*;
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
    let mut tev: TevWrappedClient = TevWrappedClient::new();
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

        // match and execute the given command
        match command.len() {
            0 => continue,
            1 => {
                if command == "#" {
                    continue;
                } else {
                    print_tui_help();
                }
            }
            2 => {
                if command == "//" {
                    continue;
                } else {
                    print_tui_help();
                }
            }
            3 => {
                if command == "tev" {
                    fn prompt() {
                        println!("tev entry_name entry_num");
                        println!("");
                    }

                    match tev.start_client() {
                        Ok(_) => {}
                        Err(err) => {
                            println!("Failed starting tev:\n\t{}", err);
                        }
                    }

                    if input.len() == 3 {
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
                                let mut png_output = PNGOutput::new();
                                png_output.add_points(&entry, 2.0, 500.0, 222.0, 50);
                                match png_output
                                    .to_pixmap(2.0, 500.0)
                                    .save_png(format!("/tmp/{}-{:04}.png", key.0, key.1))
                                {
                                    Ok(_) => {
                                        println!(
                                            "Saved {}-{:04}.png with {} entries.",
                                            key.0,
                                            key.1,
                                            entry.len()
                                        );
                                    }
                                    Err(err) => {
                                        println!(
                                            "Failed saving to file {}-{:04}.png:\n\t{}",
                                            key.0,
                                            key.1,
                                            err.to_string()
                                        );
                                    }
                                }

                                match tev.open_image(format!("/tmp/{}-{:04}.png", key.0, key.1)) {
                                    Ok(_) => {
                                        println!("Opened image: /tmp/{}-{:04}.png", key.0, key.1);
                                    }
                                    Err(err) => {
                                        println!("Failed opening image:\n\t{}", err);
                                    }
                                }
                            }
                            None => {
                                prompt();
                                println!("Entry {}-{:04} didn't exist!", key.0, key.1);
                            }
                        }
                    } else {
                        prompt();
                    }
                }
            }
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
            6 => {
                if command == "output" {
                    fn prompt_multi_entry() {
                        println!("output [options] file_name");
                        println!("\tentry_name entry_num [hue]");
                        println!("\tentry_name entry_num [hue]");
                        println!("\t...");
                    }

                    fn prompt() {
                        println!("output [options] entry_name entry_num [hue]");
                        prompt_multi_entry();
                        println!("");
                    }

                    fn prompt_options() {
                        println!("options:");
                        println!("\t--png:\t\t(DEFAULT)output in PNG format");
                        println!("\t--svg:\t\toutput in SVG(Scalable Vector Graphics) format");
                        println!("\t--scale SCALE:\t(DEFAULT=1000)how much pixel for a meter");
                        println!("\t--clip POS:\t(DEFAULT=2)how far to include in the output");
                        println!("\t--help:\t\tprint this message");
                    }

                    enum OutputType {
                        SVG,
                        PNG,
                    }
                    struct OutputOption {
                        output_type: OutputType,
                        scale: f64,
                        clip_pos: f64,
                        help: bool,
                    }

                    fn parse_options(input: &Vec<&str>) -> Result<(OutputOption, usize), String> {
                        let mut option: OutputOption = OutputOption {
                            output_type: OutputType::PNG,
                            scale: 1000.0,
                            clip_pos: 2.0,
                            help: false,
                        };

                        let mut next: usize = 1;
                        loop {
                            // check if input have enough arguments to parse
                            if input.len() <= next {
                                return Ok((option, next));
                            }
                            if input[next] == "--" {
                                return Ok((option, next + 1));
                            } else if input[next] == "--png" {
                                option.output_type = OutputType::PNG;
                                next += 1;
                            } else if input[next] == "--svg" {
                                option.output_type = OutputType::SVG;
                                next += 1;
                            } else if input[next] == "--scale" {
                                if input.len() <= next + 1 {
                                    return Err(
                                        "Expect f64 after --scale, getting None".to_string()
                                    );
                                } else {
                                    option.scale = match input[next + 1].parse() {
                                        Ok(scale) => {
                                            if scale > 0.0 {
                                                scale
                                            } else {
                                                return Err(
                                                    "Expect positive f64 for SCALE".to_string()
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            return Err(format!(
                                                "Expect f64 after --scale, {}",
                                                err.to_string()
                                            ))
                                        }
                                    };
                                    next += 2;
                                }
                            } else if input[next] == "--clip" {
                                if input.len() <= next + 1 {
                                    return Err("Expect f64 after --clip, getting None".to_string());
                                } else {
                                    option.clip_pos = match input[next + 1].parse() {
                                        Ok(clip_pos) => {
                                            if clip_pos > 0.0 {
                                                clip_pos
                                            } else {
                                                return Err(
                                                    "Expect positive f64 for POS".to_string()
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            return Err(format!(
                                                "Expect f64 after --clip, {}",
                                                err.to_string()
                                            ))
                                        }
                                    };
                                    next += 2;
                                }
                            } else if input[next] == "--help" {
                                option.help = true;
                                next += 1;
                            } else {
                                // left any unknown command as filename/entry name
                                return Ok((option, next));
                            }
                        }
                    }

                    let (option, next) = match parse_options(&input) {
                        Ok(result) => result,
                        Err(err) => {
                            prompt();
                            prompt_options();
                            println!("Error happened parsing options:\t\n{}", err);
                            continue;
                        }
                    };

                    if option.help {
                        prompt();
                        prompt_options();
                        continue;
                    }

                    if input.len() <= next {
                        prompt();
                        println!(
                            "Expect filename or entry_name and entry_num after command and options"
                        );
                        continue;
                    }

                    if input.len() - 1 == next {
                        fn tui_get_entry_keys_and_hue(
                            point_files: &IndexMap<(String, u32), Vec<(f64, f64)>>,
                            prompt: fn(),
                        ) -> Vec<((String, u32), f64)> {
                            let mut entry_keys: Vec<((String, u32), f64)> = Vec::new();
                            let mut no_hue: u32 = 0;
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
                                let entry_input: Vec<&str> =
                                    entry_input.trim().split(' ').collect();
                                if entry_input.len() == 1 && entry_input[0].len() == 0 {
                                    break;
                                }

                                if entry_input.len() != 2 && entry_input.len() != 3 {
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
                                            println!(
                                                "Error happened parsing entry_num: \n\t{}",
                                                err.to_string()
                                            );
                                            continue;
                                        }
                                    },
                                );

                                // reading hue from input or uses the default
                                let hue = if entry_input.len() == 3 {
                                    match entry_input[2].parse::<f64>() {
                                        Ok(hue) => {
                                            if 0.0 <= hue && hue <= 360.0 {
                                                hue
                                            } else {
                                                no_hue += 1;
                                                -1.0
                                            }
                                        }
                                        Err(err) => {
                                            prompt();
                                            println!("Expect f64 for hue, {}", err.to_string());
                                            continue;
                                        }
                                    }
                                } else {
                                    no_hue += 1;
                                    -1.0
                                };

                                // push if the entry exist in point_files
                                if point_files.contains_key(&entry_key) {
                                    entry_keys.push((entry_key, hue));
                                } else {
                                    prompt();
                                    println!(
                                        "Entry {}-{:04} didn't exist!",
                                        entry_key.0, entry_key.1
                                    );
                                    continue;
                                }
                            }
                            let mut current_hue = 0.0;
                            if no_hue > 0 {
                                let step_hue = 360.0 / no_hue as f64;
                                for (_, hue) in &mut entry_keys {
                                    if *hue < 0.0 {
                                        *hue = current_hue;
                                        current_hue += step_hue;
                                    }
                                }
                                assert!(current_hue - 360.0 < 0.01);
                            }
                            entry_keys
                        }
                        // output [options] filename
                        // let pngoutput = PNGOutput::new();
                        // let svgoutput = SVGOutput::new();
                        //
                        // for key in tui_get_entry_keys(&point_files, prompt) {
                        //     // get entry names ensure the keys are valid so we can safely unwrap
                        //     combined_entry.append(&mut (point_files.get(&key).unwrap().clone()));
                        // }

                        let keys_and_hues =
                            tui_get_entry_keys_and_hue(&point_files, prompt_multi_entry);
                        match option.output_type {
                            OutputType::PNG => {
                                let mut png_output = PNGOutput::new();
                                for (entry, hue) in keys_and_hues {
                                    png_output.add_points(
                                        &point_files.get(&entry).unwrap(),
                                        option.clip_pos,
                                        option.scale,
                                        hue,
                                        50,
                                    );
                                }

                                match png_output
                                    .to_pixmap(option.clip_pos, option.scale)
                                    .save_png(input[next])
                                {
                                    Ok(_) => {
                                        println!("Saved {}", input[next]);
                                    }
                                    Err(err) => {
                                        println!(
                                            "Failed saving to file {}:\t\n{}",
                                            input[next],
                                            err.to_string()
                                        );
                                    }
                                }
                            }
                            OutputType::SVG => {
                                let mut svg_output = SVGOutput::new();
                                for (entry, hue) in keys_and_hues {
                                    svg_output.add_points(
                                        &point_files.get(&entry).unwrap(),
                                        option.clip_pos,
                                        option.scale,
                                        hue,
                                        50,
                                    );
                                }

                                match svg::save(
                                    input[next],
                                    &svg_output
                                        .output_to_empty_document(option.scale, option.clip_pos),
                                ) {
                                    Ok(_) => {
                                        println!("Saved {}", input[next]);
                                    }
                                    Err(err) => {
                                        println!(
                                            "Failed saving to file {}:\t\n{}",
                                            input[next],
                                            err.to_string()
                                        );
                                    }
                                }
                            }
                        }
                    } else if input.len() - 2 == next || input.len() - 3 == next {
                        // output [options] entry_name entry_num [hue]
                        let hue = if input.len() - 3 == next {
                            match input[next + 2].parse::<f64>() {
                                Ok(hue) => hue,
                                Err(err) => {
                                    prompt();
                                    println!("Expect f64 for hue, {}", err.to_string());
                                    continue;
                                }
                            }
                        } else {
                            0.0
                        };

                        if input[next + 1] == "*" && {
                            let mut iter = point_files.iter();
                            loop {
                                match iter.next() {
                                    Some((key, _)) => {
                                        if key.0 == input[next] {
                                            break true;
                                        }
                                    }
                                    None => {
                                        break false;
                                    }
                                }
                            }
                        } {
                            match option.output_type {
                                OutputType::PNG => {
                                    fn png_output_entry(
                                        key: &(String, u32),
                                        entry: &Vec<(f64, f64)>,
                                        input: &Vec<&str>,
                                        next: usize,
                                        hue: f64,
                                        option: &OutputOption,
                                    ) {
                                        let mut png_output = PNGOutput::new();
                                        if key.0 == input[next] {
                                            png_output.add_points(
                                                &entry,
                                                option.clip_pos,
                                                option.scale,
                                                hue,
                                                50,
                                            );
                                            match png_output
                                                .to_pixmap(option.clip_pos, option.scale)
                                                .save_png(format!("{}-{:04}.png", key.0, key.1))
                                            {
                                                Ok(_) => {
                                                    println!(
                                                        "Saved {}-{:04}.png with {} entries.",
                                                        key.0,
                                                        key.1,
                                                        entry.len()
                                                    );
                                                }
                                                Err(err) => {
                                                    println!(
                                                        "Failed saving to file {}-{:04}.png:\t\n{}",
                                                        key.0,
                                                        key.1,
                                                        err.to_string()
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    point_files.par_iter().for_each(|x| {
                                        png_output_entry(x.0, x.1, &input, next, hue, &option)
                                    });
                                }
                                OutputType::SVG => {
                                    fn svg_output_entry(
                                        key: &(String, u32),
                                        entry: &Vec<(f64, f64)>,
                                        input: &Vec<&str>,
                                        next: usize,
                                        hue: f64,
                                        option: &OutputOption,
                                    ) {
                                        let mut svg_output = SVGOutput::new();
                                        if key.0 == input[next] {
                                            svg_output.add_points(
                                                &entry,
                                                option.clip_pos,
                                                option.scale,
                                                hue,
                                                50,
                                            );
                                            match svg::save(
                                                format!(
                                                    "{}.svg",
                                                    format!("{}-{:04}", key.0, key.1)
                                                ),
                                                &svg_output.output_to_empty_document(
                                                    option.scale,
                                                    option.clip_pos,
                                                ),
                                            ) {
                                                Ok(_) => {
                                                    println!(
                                                        "Saved {}-{:04}.svg with {} entries.",
                                                        key.0,
                                                        key.1,
                                                        entry.len()
                                                    );
                                                }
                                                Err(err) => {
                                                    println!(
                                                        "Failed saving to file {}-{:04}.svg:\t\n{}",
                                                        key.0,
                                                        key.1,
                                                        err.to_string()
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    point_files.par_iter().for_each(|x| {
                                        svg_output_entry(x.0, x.1, &input, next, hue, &option)
                                    });
                                }
                            }
                        } else {
                            let key = (
                                input[next].to_string(),
                                match input[next + 1].parse::<u32>() {
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
                                Some(entry) => match option.output_type {
                                    OutputType::PNG => {
                                        let mut png_output = PNGOutput::new();
                                        png_output.add_points(
                                            &entry,
                                            option.clip_pos,
                                            option.scale,
                                            hue,
                                            50,
                                        );
                                        match png_output
                                            .to_pixmap(option.clip_pos, option.scale)
                                            .save_png(format!("{}-{:04}.png", key.0, key.1))
                                        {
                                            Ok(_) => {
                                                println!(
                                                    "Saved {}-{:04}.png with {} entries.",
                                                    key.0,
                                                    key.1,
                                                    entry.len()
                                                );
                                            }
                                            Err(err) => {
                                                println!(
                                                    "Failed saving to file {}-{:04}.png:\t\n{}",
                                                    key.0,
                                                    key.1,
                                                    err.to_string()
                                                );
                                            }
                                        }
                                    }
                                    OutputType::SVG => {
                                        let mut svg_output = SVGOutput::new();
                                        svg_output.add_points(
                                            &entry,
                                            option.clip_pos,
                                            option.scale,
                                            hue,
                                            50,
                                        );
                                        match svg::save(
                                            format!("{}.svg", format!("{}-{:04}", key.0, key.1)),
                                            &svg_output.output_to_empty_document(
                                                option.scale,
                                                option.clip_pos,
                                            ),
                                        ) {
                                            Ok(_) => {
                                                println!(
                                                    "Saved {}-{:04}.svg with {} entries.",
                                                    key.0,
                                                    key.1,
                                                    entry.len()
                                                );
                                            }
                                            Err(err) => {
                                                println!(
                                                    "Failed saving to file {}-{:04}.png:\t\n{}",
                                                    key.0,
                                                    key.1,
                                                    err.to_string()
                                                );
                                            }
                                        }
                                    }
                                },
                                None => {
                                    prompt();
                                    println!("Entry {}-{:04} didn't exist!", key.0, key.1);
                                    continue;
                                }
                            }
                        }
                    } else {
                        prompt();
                        println!("Too many arguments!");
                        continue;
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
    println!("output:\t\toutput entry(es) into file");
    println!("show:\t\tcheck if a entry exists");
    println!("tev:\t\tpreview a entry on tev");
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
