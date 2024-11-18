use std::fs;
use std::io::Read;
use std::collections::HashMap;

use std::fmt;

pub const MAJOR_VERSION: u8 = 0x37;
pub const MINOR_VERSION: u8 = 0x37;
pub const PATCH_NUM: u8 = 0x37;

pub const VAR_OFFSET: u16 = 0;
pub const CONST_OFFSET: u16 = 256;
pub const SPECIAL_VAR_OFFSET: u16 = 768;
pub const MAX_CONST_LEN: i32 = 256;
pub const CONST_HEADER_WIDTH: u16 = 2;
pub const HEADER_LEN: usize = 16;

pub const MAX_RECURSION_DEPTH: usize = 64;

#[derive(Debug)]
pub struct CError {
    msg: String,
}

impl std::fmt::Display for CError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg) // user-facing output
    }
}
impl CError {
    pub fn from_slice(st: &str) -> CError {
        CError {
            msg: st.to_string(),
        }
    }
}

impl From<String> for CError {
    fn from(st: String) -> Self {
        CError {
            msg: st.clone(),
        }
    }
}

impl From<&str> for CError {
    fn from(st: &str) -> Self {
        CError {
            msg: st.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for CError {
    fn from(err: std::num::ParseIntError) -> Self {
        CError {
            msg: err.to_string(),
        }
    }
}
impl From<std::string::FromUtf8Error> for CError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        CError {
            msg: err.to_string(),
        }
    }
}

impl From<std::io::Error> for CError {
    fn from(error: std::io::Error) -> Self {
        CError {
            msg: error.to_string(),
        }
    }
}


#[derive(Debug,Clone,PartialEq)]
pub enum TType {
    Var,
    Const,
    Load,
    Apply,
    Write,
    To,
    From,
    Operation(String),
    Transform(String,Vec<String>), //name, arguments
    Identifier(String),
    FileOp(String),
    Str(String),
    Directive(String),
    EndStatement,
    Nil,
}
#[derive(Debug,Clone)]
pub struct Token {
    pub ttype: TType,
    pub line: usize,
}
impl Token {
    pub fn new() -> Token {
        Token {
            ttype: TType::Nil,
            line: 0,
        }
    }
    pub fn new_val(t: TType, l: usize) -> Token {
        Token {
            ttype: t,
            line: l,
        }
    }
    pub fn is_var(&self) -> bool {
        if let TType::Var = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_const(&self) -> bool {
        if let TType::Const = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_from(&self) -> bool {
        if let TType::From = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_to(&self) -> bool {
        if let TType::To = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_load(&self) -> bool {
        if let TType::Load = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_apply(&self) -> bool {
        if let TType::Apply = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_write(&self) -> bool {
        if let TType::Write = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_ident(&self) -> bool {
        if let TType::Identifier(_) = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_transform(&self) -> bool {
        if let TType::Transform(_, _) = &self.ttype {
            true
        } else {
            false
        }
    }
    pub fn is_oper(&self) -> bool {
        if let TType::Operation(_) = &self.ttype {
            true
        } else {
            false
        }

    }
}

#[derive(Debug,Clone,PartialEq)]
pub enum PIType {
    Load,
    Apply,
    Operation,
    /*  Cipherelang v2 instructions
    For,
    Choose,
    End,
    Label,
    Branch,
    SKNE,
    SKE,
    */
    Nil,

}
#[derive(Debug,Clone)]
pub struct ProtoInstruction {
    pub pitype: PIType,
    pub source: String,
    pub second_source: String,
    pub target: String,
    pub value: String,
    pub args: Vec<String>,
    pub line: usize,
    pub arg_str: Vec<bool>,
}
impl ProtoInstruction {
    pub fn clear(&mut self) {
        self.pitype = PIType::Nil;
        self.source = String::new();
        self.second_source = String::new();
        self.target = String::new();
        self.value = String::new();
        self.args = Vec::new();
        self.line = 0;
        self.arg_str = Vec::new();
    }
    pub fn new() -> ProtoInstruction {
        ProtoInstruction {
            pitype: PIType::Nil,
            source: String::new(),
            second_source: String::new(),
            target: String::new(),
            value: String::new(),
            args: Vec::new(),
            line: 0,
            arg_str: Vec::new(),
        }
    }
    pub fn binarify(&self) -> Result<BinaryInstruction, String> {
      let number_base: u16 = 0x00;
      let negative_base: u16 = 0x01;
      let const_base: u16 = 0x10;
      let mut bi = BinaryInstruction::new();
      match self.pitype {
        PIType::Load => {
          bi.opcode = 0;
          bi.first = self.source.parse().unwrap();
          bi.second = self.target.parse().unwrap();
          bi.argc = 0;
          bi.third = 0;
        },
        PIType::Apply => {
          bi.opcode = 1;
          bi.first = self.target.parse().unwrap();
          bi.second = self.value.parse().unwrap();
          bi.third = 0;
          bi.argc = self.args.len() as u8;
          //convert args:
          for (i, x) in self.args.iter().enumerate() {
            if x.is_empty() {
              continue;
            }
            if self.arg_str[i] {
              bi.args.push( 
                ((const_base & 0x3f) << 0xA) as u16| (x.parse::<u16>().unwrap() & 0x3ff)
              );
            } else {
              let value: i16 = x.parse().unwrap();
              if value < 0 {
            
                bi.args.push(
                  ((negative_base & 0x3f) << 0xA) as u16 | (value.abs() as u16 & 0x3ff)
                );
              } else { //normal
                  bi.args.push( (
                    ((number_base & 0x3f) << 0xA) as i16 | (value & 0x3ff) )as u16 );
              }
            }

          }
        },
        PIType::Operation => {
          bi.opcode = 2;
          bi.first = self.source.parse().unwrap();
          bi.second = self.second_source.parse().unwrap();
          bi.third = self.value.parse().unwrap();
          bi.argc = 0;
        },
        PIType::Nil => {
          return Err(format!("Error: Nil instruction during binarification"));
        },
      }
      Ok(bi)
    }
}
#[derive(Debug)]
pub struct BinaryInstruction {
    pub opcode: u8,
    pub first: u16,
    pub second: u16,
    pub third: u16,
    pub argc: u8,
    pub args: Vec<u16>,
}
impl BinaryInstruction {

    pub fn binary(&self) -> Result<Vec<u8>, String> {
      let mut v: Vec<u8> = Vec::new();
      //this will need to be edited for ciplang v2
      v.resize(5 + (self.opcode == 1) as usize + self.args.len()*2 + 2*(self.opcode == 2) as usize, 0);

      v[0] = self.opcode;
      v[1] = ((self.first & 0xff00) >> 8) as u8;
      v[2] = (self.first & 0xff) as u8;
      v[3] = ((self.second & 0xff00) >> 8) as u8;
      v[4] = (self.second & 0xff) as u8;

      if self.opcode == 1 {
        v[5] = self.argc;
        for (i, x) in self.args.iter().enumerate() {
          v[2*i + 6] = ((x & 0xff00) >> 8) as u8;
          v[2*i + 7] = (x & 0xff) as u8;
        }
      } else if self.opcode == 2 {
        v[5] = ((self.third & 0xff00) >> 8) as u8;
        v[6] = (self.third & 0xff) as u8;
      } else if self.opcode == 0 {
        //do nothing
      } else {
        return Err(format!("Invalid opcode during text generation"));
      }
      

      Ok(v)
    }
    pub fn new() -> BinaryInstruction {
      BinaryInstruction {
        opcode: 0xff,
        first: 0,
        second: 0,
        third: 0,
        argc: 0,
        args: Vec::new(),
      }
    }
}
pub fn read_bin_file(infile: &String) -> Vec<u8> {
    let mut f = fs::File::open(&infile).expect("Could not open file.");
    let meta = fs::metadata(&infile).expect("Could not read metadata.");
    let mut buffer = vec![0; meta.len() as usize];
    f.read(&mut buffer).expect("Buffer overflow error");

    buffer
}

pub fn read_file(infile: &String) -> Option<String> {
    let r = fs::read_to_string(infile);
    match r {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

// Splits a string up to a vector by delim
pub fn parse(st: &str, delim: char) -> Vec<String> {
    let mut th: Vec<String> = Vec::new();
    let mut in_str: bool = false;
    let mut cw = String::new();
    
    for ch in st.chars() {
        if ch == delim {
            if in_str {
                cw.push(ch);
            } else {
                if !cw.is_empty() {
                    th.push(cw);
                    cw = String::new();
                }
            }
        }
        else if ch == '"' {
            cw.push('"');
            in_str = !in_str;

        } else {
            cw.push(ch);
        }

    }
    //the final piece won't be added.
    if !cw.is_empty() {
        th.push(cw);
    }
    th
}

pub fn value_in_str_map(map: &HashMap<String, String>, val: &String) -> Option<String> {
    map.iter().find_map(|(k, v)| if v == val {Some(k.to_string())} else {None})
}

pub fn index_of_vec_val(arr: &Vec<String>, val: &String) -> Option<usize> {
    arr.iter().position(|r| r == val)
}
