use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Result};

use crate::parser::Parser;

#[derive(Debug)]
pub struct Translator {
    src: Vec<String>,
    parser: Parser,
}

impl Translator {
    pub fn new(filename: &impl AsRef<Path>) -> Result<Translator> {
	let file = File::open(filename)?;
	let buf = BufReader::new(file);
	let src = buf.lines()
	    .map(|l| l.expect("Could not parse line"))
	    .collect();

        let parser = Parser::new();

        Ok(Translator{ src, parser })
    }

    pub fn process(&mut self) -> Result<()> {
        for line in &self.src {
            &self.parser.process_line(line)?;
        }
        &self.parser.debug();
        Ok(())
    }

   
    pub fn write_bin(&self, binname: &String) -> Result<()> {
        let mut buf = "".to_string();

        buf.push_str("// Hack ASM (for nand2tetris book) generated from VM code\n");
        buf.push_str("// by Brent Hamilton <github.com/distrill/n2t-vm-translator>\n");
        for asm in &self.parser.asm {
            buf.push_str(format!("\n\n{}\n", &asm.src).as_str());
            for binline in &asm.bin {
                buf.push_str(format!("{}\n", binline).as_str());
            }
        }

        fs::write(binname, buf)?;
    
        Ok(())
    } 
}
