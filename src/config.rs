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
