use coolor::*;
use ptfl_reader::Config;
use ptfl_reader::PtflParser;
use std::env;
use tiny_skia::Paint;
use tiny_skia::PathBuilder;
use tiny_skia::Pixmap;
use tiny_skia::Rect;
use tiny_skia::Stroke;
use tiny_skia::Transform;

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
    for i in point_files {
        let mut all_path_builder = PathBuilder::new();
        let mut non_zero_path_builder = PathBuilder::new();
        let mut points_path_builder = PathBuilder::new();

        all_path_builder.move_to(
            ((&i.1)[0].1 * (&i.1)[0].0.cos() + clip_pos) as f32,
            ((&i.1)[0].1 * (&i.1)[0].0.sin() + clip_pos) as f32,
        );
        let mut entry_iter = i.1.iter();
        if loop {
            let j = match entry_iter.next() {
                Some(some) => some,
                // iteration is finished
                None => break false,
            };

            let x: f32 = (j.1 * j.0.cos() + clip_pos) as f32;
            let y: f32 = (j.1 * j.0.sin() + clip_pos) as f32;

            // its possible to both move_to(x, y) and line_to(x, y),
            // but that's not a issue
            all_path_builder.line_to(x, y);
            if j.1 != 0.0 {
                // this might never be executed if all point is (angle, 0)
                // this is handled later by matching .finish()
                non_zero_path_builder.move_to(x, y);
                points_path_builder.move_to(x + 0.005, y + 0.005);
                points_path_builder.line_to(x - 0.005, y + 0.005);
                points_path_builder.line_to(x - 0.005, y - 0.005);
                points_path_builder.line_to(x + 0.005, y - 0.005);
                points_path_builder.line_to(x + 0.005, y + 0.005);
                points_path_builder.close();
                break true;
            }
        } {
            loop {
                let j = match entry_iter.next() {
                    Some(some) => some,
                    None => break,
                };

                let x: f32 = (j.1 * j.0.cos() + clip_pos) as f32;
                let y: f32 = (j.1 * j.0.sin() + clip_pos) as f32;
                all_path_builder.line_to(x, y);
                if j.1 != 0.0 {
                    non_zero_path_builder.line_to(x, y);
                    points_path_builder.move_to(x + 0.005, y + 0.005);
                    points_path_builder.line_to(x - 0.005, y + 0.005);
                    points_path_builder.line_to(x - 0.005, y - 0.005);
                    points_path_builder.line_to(x + 0.005, y - 0.005);
                    points_path_builder.line_to(x + 0.005, y + 0.005);
                    points_path_builder.close();
                }
            }
        }

        all_path_builder.close();
        non_zero_path_builder.close();
            

        let mut pixmap = Pixmap::new(
            (2.0 * clip_pos * scale) as u32,
            (2.0 * clip_pos * scale) as u32,
        )
        .unwrap();
        let mut paint = Paint::default();
        paint.anti_alias = true;

        paint.set_color_rgba8(0, 0, 0, 255);
        pixmap
            .fill_rect(
                Rect::from_xywh(0.0, 0.0, 2.0 * clip_pos as f32, 2.0 * clip_pos as f32).unwrap(),
                &paint,
                Transform::from_scale(scale as f32, scale as f32),
                None,
            )
            .unwrap();

        let rgba = Hsl::new(0.0, 0.4, 0.5).to_rgb();
        paint.set_color_rgba8(rgba.r, rgba.g, rgba.b, (1.0 * 255.0) as u8);
        let mut stroke = Stroke::default();
        stroke.width = 0.0005 as f32; // hairline
        pixmap.stroke_path(
            &all_path_builder.finish().unwrap(),
            &paint,
            &stroke,
            Transform::from_scale(scale as f32, scale as f32),
            None,
        );

        let rgba = Hsl::new(0.0, 0.7, 0.5).to_rgb();
        paint.set_color_rgba8(rgba.r, rgba.g, rgba.b, (1.0 * 255.0) as u8);
        let mut stroke = Stroke::default();
        stroke.width = 0.003 as f32;
        if let Some(non_zero_path) = non_zero_path_builder.finish() {
            pixmap.stroke_path(
                &non_zero_path,
                &paint,
                &stroke,
                Transform::from_scale(scale as f32, scale as f32),
                None,
            );
        }

        let rgba = Hsl::new(0.0, 1.0, 0.5).to_rgb();
        paint.set_color_rgba8(rgba.r, rgba.g, rgba.b, (1.0 * 255.0 as f64).ceil() as u8);
        let mut stroke = Stroke::default();
        stroke.width = 0.002 as f32;
        if let Some(points_path) = points_path_builder.finish() {
            pixmap.stroke_path(
                &points_path,
                &paint,
                &stroke,
                Transform::from_scale(scale as f32, scale as f32),
                None,
            );
        }

        pixmap.save_png(format!("./tests/{}.png", i.0)).unwrap();
    }
}

fn print_help() {
    println!("ptfl_reader [files] [--help]");
    println!("    files: one or multiple file path as input");
    println!("    help: print this message");
}
