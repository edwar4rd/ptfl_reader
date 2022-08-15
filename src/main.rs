use ptfl_reader::Config;
use ptfl_reader::PtflParser;
use std::env;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::element::Rectangle;
use svg::Document;

fn main() {
    let scale = 250.0;
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

    for i in &point_files {
        let rect = Rectangle::new()
            .set("fill", "black")
            .set("width", "100%")
            .set("height", "100%");

        let all_path = Path::new()
            .set("fill", "none")
            .set("stroke", "navy")
            .set("stroke-width", scale * 0.0005)
            .set("d", all_pathdata(&i.1, clip_pos, scale));

        let nz_path = Path::new()
            .set("fill", "none")
            .set("stroke", "aqua")
            .set("stroke-width", scale * 0.002)
            .set("d", non_zero_pathdata(&i.1, clip_pos, scale));

        let points_path = Path::new()
            .set("fill", "none")
            .set("stroke", "yellow")
            .set("stroke-width", scale * 0.001)
            .set("d", non_zero_path_square_data(&i.1, clip_pos, scale, 0.006));

        let document = Document::new()
            .set(
                "viewBox",
                (0, 0, scale * clip_pos * 2.0, scale * clip_pos * 2.0),
            )
            .add(rect)
            .add(all_path)
            .add(nz_path)
            .add(points_path);

        svg::save(format!("./tests/{}.svg", i.0), &document).unwrap();
    }
}

fn print_help() {
    println!("ptfl_reader [files] [--help]");
    println!("    files: one or multiple file path as input");
    println!("    help: print this message");
}

fn all_pathdata(points: &Vec<(f64, f64)>, clip_pos: f64, scale: f64) -> Data {
    let mut data = Data::new().move_to((
        scale * (&points[0].1 * &points[0].0.cos() + clip_pos),
        scale * (&points[0].1 * &points[0].0.sin() + clip_pos),
    ));
    for j in points {
        data = data.line_to((
            scale * (j.1 * j.0.cos() + clip_pos),
            scale * (j.1 * j.0.sin() + clip_pos),
        ));
    }
    data.close()
}

fn non_zero_pathdata(points: &Vec<(f64, f64)>, clip_pos: f64, scale: f64) -> Data {
    let mut data = Data::new().move_to({
        let mut a = points.iter();
        loop {
            let j = match a.next() {
                Some(next) => next,
                None => break (0.0, 0.0),
            };
            if j.1 != 0.0 {
                break (
                    scale * (j.1 * (j.0.cos()) + clip_pos),
                    scale * (j.1 * (j.0.sin()) + clip_pos),
                );
            }
        }
    });
    for j in points {
        if j.1 != 0.0 {
            data = data.line_to((
                scale * (j.1 * (j.0.cos()) + clip_pos),
                scale * (j.1 * (j.0.sin()) + clip_pos),
            ));
        }
    }
    data.close()
}

fn non_zero_path_square_data(
    points: &Vec<(f64, f64)>,
    clip_pos: f64,
    scale: f64,
    square_size: f64,
) -> Data {
    let mut data = Data::new();
    for j in points {
        if j.1 != 0.0 {
            data = data.move_to((
                scale * (j.1 * (j.0.cos()) + clip_pos + square_size / 2.0),
                scale * (j.1 * (j.0.sin()) + clip_pos + square_size / 2.0),
            ));
            data = data.line_by((scale * -square_size, 0));
            data = data.line_by((0, scale * -square_size));
            data = data.line_by((scale * square_size, 0));
            data = data.close();
        }
    }
    data.close()
}
