extern crate image;

use ptfl_reader::Config;
use ptfl_reader::PtflParser;
use std::env;

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
    for i in point_files {
        let imgx = 800;
        let imgy = 800;

        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);
        for j in i.1 {
            if j.0 > 4.0_f64.sqrt() {
                continue;
            }
            let x = j.1 / 4.0 * ((j.0).cos());
            let y = j.1 / 4.0 * ((j.0).sin());
            let x = if (x * 800.0 + 400.0) < (imgx as f64) {
                (x * 800.0 + 400.0) as u32
            } else {
                continue
            };
            let y = if (y * 800.0 + 400.0) < (imgy as f64) {
                (y * 800.0 + 400.0) as u32
            } else {
                continue;
            };
            let pixel = imgbuf.get_pixel_mut(x, y);
            *pixel = image::Rgb([5 as u8, 255 as u8, 255 as u8]);
        }
        imgbuf.save(format!("./tests/{}.png", i.0)).unwrap();
    }
}

fn print_help() {
    println!("ptfl_reader [files] [--help]");
    println!("    files: one or multiple file path as input");
    println!("    help: print this message");
}
