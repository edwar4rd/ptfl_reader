use std::fs;
use std::path::Path;
use svg::node::element::path::Data as SVGData;
use svg::node::element::Path as SVGPath;
use svg::node::element::Rectangle as SVGRectangle;
use svg::Document as SVGDocument;

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
                            Ok(angle) => angle,
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

pub fn svg_empty_document(scale: f64, clip_pos: f64) -> SVGDocument {
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

pub fn svg_output_to_document(
    document: SVGDocument,
    points: &Vec<(f64, f64)>,
    clip_pos: f64,
    scale: f64,
    hue: f64,
    brightness: u32,
) -> SVGDocument {
    let all_path = SVGPath::new()
        .set("fill", "none")
        .set("stroke", format!("hsla({hue},40%,{brightness}%, 0.3)"))
        .set("stroke-width", scale * 0.0005)
        .set("d", all_path_svgdata(&points, clip_pos, scale));

    let nz_path = SVGPath::new()
        .set("fill", "none")
        .set("stroke", format!("hsla({hue},70%,{brightness}%, 0.6)"))
        .set("stroke-width", scale * 0.003)
        .set("d", non_zero_path_svgdata(&points, clip_pos, scale));

    let points_path = SVGPath::new()
        .set("fill", "none")
        .set("stroke", format!("hsla({hue},100%,{brightness}%, 0.8)"))
        .set("stroke-width", scale * 0.002)
        .set(
            "d",
            non_zero_path_square_svgdata(&points, clip_pos, scale, 0.01),
        );

    document.add(all_path).add(nz_path).add(points_path)
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
