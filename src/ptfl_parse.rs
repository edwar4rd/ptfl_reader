use indexmap::IndexMap;
use std::fs;
use std::path::Path;

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
        target: &mut IndexMap<(String, u32), Vec<(f64, f64)>>,
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
                            Ok(length) => {
                                if length > 0 {
                                    length
                                } else {
                                    return Err(
                                        "Expected non-zero u32 or empty when None, got Zero"
                                            .to_string(),
                                    );
                                }
                            }
                            Err(err) => {
                                return Err(format!(
                                    "Expected non-zero u32 or empty when None, {}",
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
                        target.insert(
                            (
                                Path::new(&path)
                                    .file_name()
                                    .expect("Given path isn't a file")
                                    .to_str()
                                    .expect("Invalid unicode in filename")
                                    .to_string(),
                                file_entry_num,
                            ),
                            current_reg.clone(),
                        );
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
