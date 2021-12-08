use std::env;

use anyhow::{anyhow, Result};

mod codegen;
mod parser;
mod translator;

use translator::Translator;

#[derive(Debug)]
struct Config {
    srcname: String,
    binname: String,
}

impl Config {
    fn parse(args: Vec<String>) -> Result<Config> {
        if args.len() < 2 {
            return Err(anyhow!("not enough arguments"));
        }
        let srcname = args[1].clone();
        if !srcname.ends_with(".vm") {
            return Err(anyhow!("file must be vm file. (provided: {})", srcname,));
        }

        let binname = srcname.replace(".vm", ".asm");
        Ok(Config { srcname, binname })
    }
}

fn main() -> Result<()> {
    let config = Config::parse(env::args().collect())?;
    let mut translator = Translator::new(&config.srcname)?;

    println!("translating {}", &config.srcname);
    translator.process()?;
    translator.write_bin(&config.binname)?;
    println!("written to {}", &config.binname);

    Ok(())
}
