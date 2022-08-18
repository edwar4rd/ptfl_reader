use coolor::*;
use std::fs;
use std::path::Path;
use svg::node::element::path::Data as SVGData;
use svg::node::element::Path as SVGPath;
use svg::node::element::Rectangle as SVGRectangle;
use svg::Document as SVGDocument;
use tiny_skia::Paint;
use tiny_skia::PathBuilder;
use tiny_skia::Pixmap;
use tiny_skia::Rect;
use tiny_skia::Stroke;
use tiny_skia::Transform;

pub struct Config {
    pub filenames: Vec<String>,
    pub help: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, String> {
        let mut filenames: Vec<String> = Vec::new();
        for i in &args[1..] {
            if !i.starts_with("--") {
                if !Path::new(i).exists() {
                    return Result::Err(format!("Given filepath {} do not exist", i));
                }
                filenames.push(i.to_string());
            } else {
                if i == "--help" {
                    return Result::Ok(Config {
                        filenames,
                        help: true,
                    });
                }
            }
        }

        Result::Ok(Config {
            filenames,
            help: false,
        })
    }
}

enum ParsingState {
    None,
    DuringReg(usize),
}

pub struct PtflParser {
    state: ParsingState,
}

impl PtflParser {
    pub fn new() -> PtflParser {
        return PtflParser {
            state: ParsingState::None,
        };
    }

    pub fn renew(&mut self) {
        self.state = ParsingState::None;
    }

    pub fn parse(
        &mut self,
        path: &str,
        target: &mut Vec<(String, Vec<(f64, f64)>)>,
    ) -> Result<u32, String> {
        let file = match fs::read_to_string(&path) {
            Ok(file) => file,
            Err(err) => {
                return Err(format!("Failed reading file: \n\t\t{}", err.to_string()));
            }
        };

        let mut current_reg: Vec<(f64, f64)> = Vec::new();
        let mut file_entry_num: u32 = 0;

        for line in file.lines() {
            self.state = match self.state {
                ParsingState::None => {
                    if line.is_empty() {
                        ParsingState::None
                    } else {
                        let reg_length: usize = match line.parse() {
                            Ok(length) => length,
                            Err(err) => {
                                return Err(format!(
                                    "Expected u32 or empty when None, {}",
                                    err.to_string()
                                ))
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
                    if reg.len() != 2 {
                        return Err(format!(
                            "Expected 2 comma separated when DuringReg, get {}",
                            reg.len()
                        ));
                    }

                    let reg: (f64, f64) = (
                        match reg[0].trim().parse() {
                            Ok(angle) => angle,
                            Err(err) => {
                                return Err(format!(
                                    "Expected float when DuringReg, {}",
                                    err.to_string()
                                ))
                            }
                        },
                        match reg[1].trim().parse() {
                            Ok(range) => range,
                            Err(err) => {
                                return Err(format!(
                                    "Expected float when DuringReg, {}",
                                    err.to_string()
                                ))
                            }
                        },
                    );
                    current_reg.push(reg);
                    if next == 0 {
                        target.push((
                            format!(
                                "{}-{:04}",
                                Path::new(&path)
                                    .file_name()
                                    .expect("Given path isn't a file")
                                    .to_str()
                                    .expect("Invalid unicode in filename"),
                                file_entry_num
                            ),
                            current_reg.clone(),
                        ));
                        file_entry_num += 1;
                        ParsingState::None
                    } else {
                        ParsingState::DuringReg(remaining - 1)
                    }
                }
            }
        }
        Ok(file_entry_num)
    }
}

pub struct SVGOutput {
    all_paths: Vec<SVGPath>,
    non_zero_paths: Vec<SVGPath>,
    points_paths: Vec<SVGPath>,
}

impl SVGOutput {
    pub fn new() -> SVGOutput {
        SVGOutput {
            all_paths: Vec::new(),
            non_zero_paths: Vec::new(),
            points_paths: Vec::new(),
        }
    }

    pub fn add_points(
        &mut self,
        points: &Vec<(f64, f64)>,
        clip_pos: f64,
        scale: f64,
        hue: f64,
        brightness: u32,
    ) {
        self.all_paths.push(
            SVGPath::new()
                .set("fill", "none")
                .set("stroke", format!("hsla({hue},40%,{brightness}%, 0.3)"))
                .set("stroke-width", scale * 0.0005)
                .set("d", all_path_svgdata(&points, clip_pos, scale)),
        );

        self.non_zero_paths.push(
            SVGPath::new()
                .set("fill", "none")
                .set("stroke", format!("hsla({hue},70%,{brightness}%, 0.6)"))
                .set("stroke-width", scale * 0.003)
                .set("d", non_zero_path_svgdata(&points, clip_pos, scale)),
        );

        self.points_paths.push(
            SVGPath::new()
                .set("fill", "none")
                .set("stroke", format!("hsla({hue},100%,{brightness}%, 0.8)"))
                .set("stroke-width", scale * 0.002)
                .set(
                    "d",
                    non_zero_path_square_svgdata(&points, clip_pos, scale, 0.01),
                ),
        );
    }

    pub fn combine(mut a: SVGOutput, mut b: SVGOutput) -> SVGOutput {
        a.all_paths.append(&mut b.all_paths);
        a.non_zero_paths.append(&mut b.non_zero_paths);
        a.points_paths.append(&mut b.points_paths);
        SVGOutput {
            all_paths: a.all_paths,
            non_zero_paths: a.non_zero_paths,
            points_paths: a.points_paths,
        }
    }

    pub fn output_to_empty_document(&self, scale: f64, clip_pos: f64) -> SVGDocument {
        let mut document = svg_empty_document(scale, clip_pos);
        for path in &self.all_paths {
            document = document.add(path.clone());
        }

        for path in &self.non_zero_paths {
            document = document.add(path.clone());
        }

        for path in &self.points_paths {
            document = document.add(path.clone());
        }

        document
    }

    pub fn output_to_document(
        &self,
        mut document: SVGDocument,
    ) -> SVGDocument {
        for path in &self.all_paths {
            document = document.add(path.clone());
        }

        for path in &self.non_zero_paths {
            document = document.add(path.clone());
        }

        for path in &self.points_paths {
            document = document.add(path.clone());
        }

        document
    }
}

fn svg_empty_document(scale: f64, clip_pos: f64) -> SVGDocument {
    svg::Document::new()
        .set("width", format!("{}px", (scale * clip_pos * 2.0) as u32))
        .set("height", format!("{}px", (scale * clip_pos * 2.0) as u32))
        .set(
            "viewBox",
            (0, 0, scale * clip_pos * 2.0, scale * clip_pos * 2.0),
        )
        .add(
            SVGRectangle::new()
                .set("fill", "black")
                .set("width", "100%")
                .set("height", "100%"),
        )
}

fn all_path_svgdata(points: &Vec<(f64, f64)>, clip_pos: f64, scale: f64) -> SVGData {
    let mut data = SVGData::new().move_to((
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

fn non_zero_path_svgdata(points: &Vec<(f64, f64)>, clip_pos: f64, scale: f64) -> SVGData {
    let mut data = SVGData::new().move_to({
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

fn non_zero_path_square_svgdata(
    points: &Vec<(f64, f64)>,
    clip_pos: f64,
    scale: f64,
    square_size: f64,
) -> SVGData {
    let mut data = SVGData::new();
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

pub struct PNGOutput {
    all_paths: Vec<(tiny_skia::Path, f64, u32)>,
    non_zero_paths: Vec<(tiny_skia::Path, f64, u32)>,
    points_paths: Vec<(tiny_skia::Path, f64, u32)>,
}

impl PNGOutput {
    pub fn new() -> PNGOutput {
        PNGOutput {
            all_paths: Vec::new(),
            non_zero_paths: Vec::new(),
            points_paths: Vec::new(),
        }
    }

    pub fn add_points(
        &mut self,
        points: &Vec<(f64, f64)>,
        clip_pos: f64,
        scale: f64,
        hue: f64,
        brightness: u32,
    ) {
        let mut all_path_builder = PathBuilder::new();
        let mut non_zero_path_builder = PathBuilder::new();
        let mut points_path_builder = PathBuilder::new();

        all_path_builder.move_to(
            (scale * ((&points)[0].1 * (&points)[0].0.cos() + clip_pos)) as f32,
            (scale * ((&points)[0].1 * (&points)[0].0.sin() + clip_pos)) as f32,
        );
        let mut entry_iter = points.iter();
        if loop {
            let j = match entry_iter.next() {
                Some(some) => some,
                // iteration is finished
                None => break false,
            };

            let x = scale * (j.1 * j.0.cos() + clip_pos);
            let y = scale * (j.1 * j.0.sin() + clip_pos);

            // its possible to both move_to(x, y) and line_to(x, y),
            // but that's not a issue
            all_path_builder.line_to(x as f32, y as f32);
            if j.1 != 0.0 {
                // this might never be executed if all point is (angle, 0)
                // this is handled later by matching .finish()
                non_zero_path_builder.move_to(x as f32, y as f32);
                points_path_builder.move_to((x + scale * 0.005) as f32, (y + scale * 0.005) as f32);
                points_path_builder.line_to((x - scale * 0.005) as f32, (y + scale * 0.005) as f32);
                points_path_builder.line_to((x - scale * 0.005) as f32, (y - scale * 0.005) as f32);
                points_path_builder.line_to((x + scale * 0.005) as f32, (y - scale * 0.005) as f32);
                points_path_builder.line_to((x + scale * 0.005) as f32, (y + scale * 0.005) as f32);
                points_path_builder.close();
                break true;
            }
        } {
            loop {
                let j = match entry_iter.next() {
                    Some(some) => some,
                    None => break,
                };

                let x = scale * (j.1 * j.0.cos() + clip_pos);
                let y = scale * (j.1 * j.0.sin() + clip_pos);
                all_path_builder.line_to(x as f32, y as f32);
                if j.1 != 0.0 {
                    non_zero_path_builder.line_to(x as f32, y as f32);
                    points_path_builder
                        .move_to((x + scale * 0.005) as f32, (y + scale * 0.005) as f32);
                    points_path_builder
                        .line_to((x - scale * 0.005) as f32, (y + scale * 0.005) as f32);
                    points_path_builder
                        .line_to((x - scale * 0.005) as f32, (y - scale * 0.005) as f32);
                    points_path_builder
                        .line_to((x + scale * 0.005) as f32, (y - scale * 0.005) as f32);
                    points_path_builder
                        .line_to((x + scale * 0.005) as f32, (y + scale * 0.005) as f32);
                    points_path_builder.close();
                }
            }

            all_path_builder.close();
            non_zero_path_builder.close();

            if let Some(all_path) = all_path_builder.finish() {
                self.all_paths.push((all_path, hue, brightness));
            }

            if let Some(non_zero_path) = non_zero_path_builder.finish() {
                self.non_zero_paths.push((non_zero_path, hue, brightness));
            }

            if let Some(points_path) = points_path_builder.finish() {
                self.points_paths.push((points_path, hue, brightness));
            }
        }
    }

    pub fn combine(mut a: PNGOutput, mut b: PNGOutput) -> PNGOutput {
        a.all_paths.append(&mut b.all_paths);
        a.non_zero_paths.append(&mut b.non_zero_paths);
        a.points_paths.append(&mut b.points_paths);
        PNGOutput {
            all_paths: a.all_paths,
            non_zero_paths: a.non_zero_paths,
            points_paths: a.points_paths,
        }
    }

    pub fn to_pixmap(&self, clip_pos: f64, scale: f64) -> Pixmap {
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
                Rect::from_xywh(
                    0.0,
                    0.0,
                    (scale * 2.0 * clip_pos) as f32,
                    (scale * 2.0 * clip_pos) as f32,
                )
                .unwrap(),
                &paint,
                Transform::identity(),
                None,
            )
            .unwrap();

        let mut stroke = Stroke::default();
        stroke.width = (0.0005 * scale) as f32;
        for i in &self.all_paths {
            let rgba = Hsl::new(i.1 as f32, 0.4, (i.2 as f64 / 100.0) as f32).to_rgb();
            paint.set_color_rgba8(rgba.r, rgba.g, rgba.b, (0.3 * 255.0) as u8);
            pixmap.stroke_path(&i.0, &paint, &stroke, Transform::identity(), None);
        }

        let mut stroke = Stroke::default();
        stroke.width = (0.003 * scale) as f32;
        for i in &self.non_zero_paths {
            let rgba = Hsl::new(i.1 as f32, 0.7, (i.2 as f64 / 100.0) as f32).to_rgb();
            paint.set_color_rgba8(rgba.r, rgba.g, rgba.b, (0.6 * 255.0) as u8);
            pixmap.stroke_path(&i.0, &paint, &stroke, Transform::identity(), None);
        }

        let mut stroke = Stroke::default();
        stroke.width = (0.002 * scale) as f32;
        for i in &self.points_paths {
            let rgba = Hsl::new(i.1 as f32, 1.0, (i.2 as f64 / 100.0) as f32).to_rgb();
            paint.set_color_rgba8(rgba.r, rgba.g, rgba.b, (0.8 * 255.0) as u8);
            pixmap.stroke_path(&i.0, &paint, &stroke, Transform::identity(), None);
        }

        pixmap
    }
}
