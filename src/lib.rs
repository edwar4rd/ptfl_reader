use std::fs;
use std::path::Path;

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
                            Err(err) => return Err(format!("Expected u32 or empty when None, {}", err.to_string())),
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
                                "{}-{file_entry_num}",
                                Path::new(&path).file_name().expect("Given path isn't a file").to_str().expect("Invalid unicode in filename")
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
