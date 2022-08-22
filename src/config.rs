use std::path::Path;

pub struct Config {
    pub filenames: Vec<String>,
    pub help: bool,
    pub no_prompt: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, String> {
        let mut filenames: Vec<String> = Vec::new();
        let mut no_prompt = false;
        for i in &args[1..] {
            if !i.starts_with("--") {
                if !Path::new(i).exists() {
                    return Result::Err(format!("Given filepath {} do not exist", i));
                }
                filenames.push(i.to_string());
            } else if i == "--help" {
                return Result::Ok(Config {
                    filenames,
                    help: true,
                    no_prompt,
                });
            } else if i == "--no-prompt" {
                no_prompt = true;
            }
        }

        Result::Ok(Config {
            filenames,
            help: false,
            no_prompt,
        })
    }
}
