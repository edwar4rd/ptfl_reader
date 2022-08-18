use ptfl_reader::Config;
use ptfl_reader::PNGOutput;
use ptfl_reader::PtflParser;
use std::env;

fn main() {
    let scale = 1000.0;
    let clip_pos = 2.0;
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
    let mut parser = PtflParser::new();
    for path in config.filenames {
        match parser.parse(path.as_str(), &mut point_files) {
            Ok(count) => {
                println!("Read {count} from {}.", path.as_str());
                println!("Currently {} regs!", point_files.len())
            }
            Err(err) => {
                println!("Error happened parsing file {path}: ");
                println!("\t{err}");
                parser.renew();
            }
        }
    }

    // print lidar points onto the image
    let mut all_output = PNGOutput::new();
    let mut this_hue = 0.0;
    for i in point_files {
        let mut output = PNGOutput::new();
        output.add_points(&i.1, clip_pos, scale, this_hue, 50);
        output
            .to_pixmap(clip_pos, scale)
            .save_png(format!("./tests/{}.png", i.0))
            .unwrap();
        all_output = PNGOutput::combine(all_output, output);
        this_hue = if this_hue + 7.0 > 360.0 {
            this_hue - 353.0
        } else {
            this_hue + 7.0
        };
    }
    all_output
        .to_pixmap(clip_pos, scale)
        .save_png(format!("./tests/all.png"))
        .unwrap();
}

fn print_help() {
    println!("ptfl_reader [files] [--help]");
    println!("    files: one or multiple file path as input");
    println!("    help: print this message");
}
