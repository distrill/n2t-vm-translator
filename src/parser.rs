use anyhow::{anyhow, Result};

use crate::codegen::CodeGen;

#[derive(Debug)]
pub enum Segment {
    Constant,
    Local,
    Argument,
    This,
    That,
    Temp,
    Pointer,
    Static,
}

impl Segment {
    fn new(raw: &str) -> Result<Segment> {
        match raw {
            "constant" => Ok(Segment::Constant),
            "local" => Ok(Segment::Local),
            "argument" => Ok(Segment::Argument),
            "this" => Ok(Segment::This),
            "that" => Ok(Segment::That),
            "temp" => Ok(Segment::Temp),
            "pointer" => Ok(Segment::Pointer),
            "static" => Ok(Segment::Static),
            _ => Err(anyhow!("unexpected dest: {}", raw)),
        }
    }

    pub fn to_address<'a>(&self) -> Result<&'a str> {
        match self {
            Segment::Local => Ok("LCL"),
            Segment::Argument => Ok("ARG"),
            Segment::This => Ok("THIS"),
            Segment::That => Ok("THAT"),
            Segment::Temp => Ok("5"),
            Segment::Pointer => Ok("3"),
            Segment::Static => Err(anyhow!("static address is contextual and only available in codegen")),
            Segment::Constant => Err(anyhow!("constant does not have an address")),
        }
    }
}

#[derive(Debug)]
pub enum BinaryToken {
    Add,
    Sub,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryToken {
    Neg,
    Not,
}

#[derive(Debug)]
pub enum ComparisonToken {
    Equal,
    LessThan,
    GreaterThan,
}

#[derive(Debug)]
pub enum StackToken {
    Push {
        segment: Segment,
        index: u16,
    },
    Pop {
        segment: Segment,
        index: u16,
    },
}

impl StackToken {
    fn new(raw: &str) -> Result<StackToken> {
        let ts = raw.split_whitespace();
        let tokens: Vec<&str> = ts.collect();

        let cmd = tokens.get(0).unwrap().trim();
        let segment = Segment::new(tokens.get(1).unwrap().trim())?;
        let index = tokens.get(2).unwrap().parse()?;

        match cmd {
            "push" => Ok(StackToken::Push{segment, index}),
            "pop" =>Ok(StackToken::Pop{segment, index}),
            _ => Err(anyhow!("unsupported stack cmd: {}", cmd)),
        }
    }
}

#[derive(Debug)]
pub enum Line {
    Stack(StackToken),
    Binary(BinaryToken),
    Unary(UnaryToken),
    Comparison(ComparisonToken),
}

impl Line {
    pub fn new(raw: &str) -> Result<Line> {
        match raw.split_whitespace().next() {
            Some(t) => {
                match t.trim() {
                    "push" | "pop" => Ok(Line::Stack(StackToken::new(raw)?)),
                    "neg" => Ok(Line::Unary(UnaryToken::Neg)),
                    "not" => Ok(Line::Unary(UnaryToken::Not)),
                    "add" => Ok(Line::Binary(BinaryToken::Add)),
                    "sub" => Ok(Line::Binary(BinaryToken::Sub)),
                    "and" => Ok(Line::Binary(BinaryToken::And)),
                    "or" => Ok(Line::Binary(BinaryToken::Or)),
                    "eq" => Ok(Line::Comparison(ComparisonToken::Equal)),
                    "lt" => Ok(Line::Comparison(ComparisonToken::LessThan)),
                    "gt" => Ok(Line::Comparison(ComparisonToken::GreaterThan)),
                    _ => Err(anyhow!("unexpected token: {}", t))
                }
            },
            None => Err(anyhow!("token cannot be null")),
        }
    }
}


#[derive(Debug)]
pub struct Asm {
    pub src: String,
    pub bin: Vec<String>,
}

#[derive(Debug)]
pub struct Parser {
    lines: Vec<Line>,
    pub asm: Vec<Asm>,
    cg: CodeGen,
    filename: String,
}

impl Parser {
    pub fn new(filename: String) -> Parser {
        Parser {
            lines: Vec::new(),
            asm: Vec::new(),
            cg: CodeGen::new(filename.clone()),
            filename,
        }
    }

    pub fn process_line(&mut self, raw: &str) -> Result<()> {
        if !raw.starts_with("//") && raw != "" {
            let line = Line::new(raw)?;
            let src = format!("// {}", raw);
            let bin = self.cg.gen_block(&line)?;
            &self.lines.push(line);
            &self.asm.push(Asm{ src, bin });
        }
        Ok(())
    }


    pub fn debug(&self) {
        println!("***  LINES  ***\n");
        for line in &self.lines {
            println!("{:?}", line);
        }
    }
}

