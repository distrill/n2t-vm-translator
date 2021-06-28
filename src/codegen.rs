use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::parser::{
    Line,
    Segment,
    UnaryToken,
    BinaryToken,
    ComparisonToken,
    StackToken,
};

#[derive(Debug)]
pub struct CodeGen {
    jmps: u8,
    vs: u8,
    statics: HashMap<u16, String>,
}

impl CodeGen {
    pub fn new() -> CodeGen {
        CodeGen{ jmps: 0, vs: 0, statics: HashMap::new() }
    }

    fn get_jmp_token(&mut self) -> String {
        let jmp_id = self.jmps;
        self.jmps += 1;
        format!("JMP_{}", jmp_id)
    }

    fn get_variable(&mut self) -> String {
        let v_id = self.vs;
        self.vs += 1;
        format!("V_{}", v_id)
    }

    fn get_static_variable(&mut self, index: &u16) -> String {
        match self.statics.get(index) {
            Some(v) => v.to_string(),
            None => {
                let v = self.get_variable();
                &self.statics.insert(*index, v.to_string());
                v
            },
        }
    }

    fn get_address(&mut self, segment: &Segment, index: &u16) -> Result<String> {
        Ok(
            if let Segment::Static = segment {
                self.get_static_variable(index)
            } else {
                segment.to_address()?.to_string()
            }
        )
    }

    fn gen_stack_block(&mut self, token: &StackToken) -> Result<Vec<String>> {
        match token {
            StackToken::Push{segment, index} => {
                let mut asm = Vec::new();

                match segment {
                    Segment::Constant => {
                        // use index directly
                        asm.push(format!("@{}", index));
                        asm.push(format!("D=A"));
                    },
                    _ => {
                        let address = self.get_address(segment, index)?;

                        // offset segment by index
                        asm.push(format!("@{}", index));
                        asm.push(format!("D=A"));
                        asm.push(format!("@{}", &address));

                        // temp and pointers are fixed with no variables
                        // but they behave like the other virtual memories
                        match segment {
                            Segment::Temp | Segment::Pointer => {
                                asm.push(format!("A=D+A"));
                            },
                            _ => {
                                asm.push(format!("A=D+M"));
                            },

                        }
                        asm.push(format!("D=M"));
                    },
                };

                asm.push(format!("@SP"));
                asm.push(format!("A=M"));
                asm.push(format!("M=D"));
                asm.push(format!("@SP"));
                asm.push(format!("M=M+1"));

                Ok(asm)
            },
            StackToken::Pop{segment, index} => {
                match segment {
                    Segment::Constant => Err(anyhow!("cannot pop constant")),
                    _ => {
                        let mut asm = Vec::new();
                        let dest = self.get_variable();
                        let address = self.get_address(segment, index)?;

                        // get segment + index and load value into "dest"
                        asm.push(format!("@{}", index));
                        asm.push(format!("D=A"));
                        asm.push(format!("@{}", address));

                        // temp and pointers are fixed, there is no variable
                        // to look up and load value from 
                        match segment {
                            Segment::Temp | Segment::Pointer => {
                                //  wink
                            },
                            _ => {
                                asm.push(format!("A=M"));
                            },

                        }
                        asm.push(format!("D=D+A"));
                        asm.push(format!("@{}", dest));
                        asm.push(format!("M=D"));

                        // dec SP and load M into D
                        asm.push(format!("@SP"));
                        asm.push(format!("M=M-1"));
                        asm.push(format!("A=M"));
                        asm.push(format!("D=M"));

                        // set popped value to saved index
                        asm.push(format!("@{}", dest));
                        asm.push(format!("A=M"));
                        asm.push(format!("M=D"));

                        Ok(asm)
                    },
                }
            },
        }
    }

    fn gen_unary_block(&self, token: &UnaryToken) -> Result<Vec<String>> {
        let mut asm = Vec::new();
        if let UnaryToken::Neg = token {
            asm.push(format!("@0"));
            asm.push(format!("D=A"));
        }

        let operation = match token {
            UnaryToken::Neg => "M=D-M",
            UnaryToken::Not => "M=!M",
        };

        asm.push(format!("@SP"));
        asm.push(format!("A=M-1"));
        asm.push(format!("{}", operation));

        Ok(asm)
    }

    fn gen_binary_block(&self, token: &BinaryToken) -> Result<Vec<String>> {
        let operation = match token {
            BinaryToken::Add => "M=D+M",
            BinaryToken::Sub => "M=M-D",
            BinaryToken::And => "M=D&M",
            BinaryToken::Or => "M=D|M",
        };
        let mut asm = Vec::new();
        asm.push(format!("@SP"));
        asm.push(format!("M=M-1"));
        asm.push(format!("A=M"));
        asm.push(format!("D=M"));
        asm.push(format!("A=A-1"));
        asm.push(format!("{}", operation));
        Ok(asm)
    }

    fn gen_comparison_block(&mut self, token: &ComparisonToken) -> Result<Vec<String>> {
        let cnd_jmp = match token {
            ComparisonToken::Equal => "JEQ",
            ComparisonToken::GreaterThan => "JGT",
            ComparisonToken::LessThan => "JLT",
        };

        let if_match = self.get_jmp_token();
        let if_not_match = self.get_jmp_token();
        let done = self.get_jmp_token();

        let mut asm = Vec::new();
        // load 1st number into D
        asm.push(format!("@SP"));
        asm.push(format!("M=M-1"));
        asm.push(format!("A=M"));
        asm.push(format!("D=M"));

        // load comparison with second numer into D
        asm.push(format!("A=A-1"));
        asm.push(format!("D=M-D"));

        // branch from comparison outcome
        asm.push(format!("@{}", if_match));
        asm.push(format!("D; {}", cnd_jmp));
        asm.push(format!("@{}", if_not_match));
        asm.push(format!("0; JMP"));

        // set D=-1 if numbers were equal
        asm.push(format!("({})", if_match));
        asm.push(format!("    @0"));
        asm.push(format!("    D=A-1"));
        asm.push(format!("    @{}", done));
        asm.push(format!("    0; JMP"));

        // set D=0 if numbers were not equal
        asm.push(format!("({})", if_not_match));
        asm.push(format!("    @0"));
        asm.push(format!("    D=A"));

        // set @SP-1 = D
        asm.push(format!("({})", done));
        asm.push(format!("    @SP"));
        asm.push(format!("    A=M"));
        asm.push(format!("    A=A-1"));
        asm.push(format!("    M=D"));

        Ok(asm)
    }

    pub fn gen_block(&mut self, line: &Line) -> Result<Vec<String>> {
        match line {
            Line::Stack(token) => self.gen_stack_block(token),
            Line::Unary(token) => self.gen_unary_block(token),
            Line::Binary(token) => self.gen_binary_block(token),
            Line::Comparison(token) => self.gen_comparison_block(token),
        }
    }
}
