use cipherlang::*;
use dirs;

use crate::interpret;

//this module contains all the built-in transforms and operations

// Helper functions

//returns true if character is an ascii special character
pub fn is_special(c: char) -> bool {
    (c >= '!' && c <= '/') || (c >= ':' && c <= '@') || (c >= '[' && c <= '`') ||
      (c >= '{' && c <= '~')
}

// Utilities:

pub fn transform_upper(input: &String) -> Result<String, CError> {
    Ok(input.to_ascii_uppercase())
}

pub fn transform_lower(input: &String) -> Result<String, CError> {
    Ok(input.to_ascii_lowercase())
}

pub fn transform_trim_special( input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| !is_special(x));
    Ok(t)
}

pub fn transform_trim_numeric(input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| !x.is_ascii_digit());
    Ok(t)
}

pub fn transform_trim_alpha(input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| !x.is_ascii_alphabetic());
    Ok(t)
}

pub fn transform_trim_whitespace(input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| !x.is_ascii_whitespace());
    Ok(t)
}

pub fn transform_prune(input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| x.is_ascii_alphabetic());
    Ok(t)
}

pub fn transform_prune_numeric(input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| x.is_ascii_digit());
    Ok(t)
}

pub fn transform_prune_ascii(input: &String) -> Result<String, CError> {
    let mut t = input.clone();
    t.retain(|x| x.is_ascii());
    Ok(t)
}


// Ciphers

pub fn transform_shift(input: &String, arg: i16) -> Result<String, CError> {
    if !input.is_ascii() {
        return Err(CError::from_slice("Error: non-ascii string"));
    }
    let mut output: Vec<u8> = Vec::new();
    for x in input.bytes() {
        if x > 64 && x <= 90 {
            output.push( (65 + ((x - 65) as i8 + arg as i8) % 26) as u8);
        } else if x > 96 && x <= 122 {
            output.push( ( 97 + ((x - 97) as i8 + arg as i8) % 26) as u8);
        } else {
            output.push(x);
        }
    }

    Ok(String::from_utf8(output)?)
}

pub fn transform_rc_encode(input: &String, arg: u16) -> Result<String, CError> {
    if arg == 1 || arg as usize > input.len(){
        return Ok(input.to_string());
    }
    let mut rails: Vec<String> = vec![String::new(); arg.into()];
    let mut up: bool = false; //is pointer moving up or down
    let mut pointer: usize = 0; //which rail is being incremented

    for x in input.chars() {
      rails[pointer].push(x);
      if pointer == 0 {
          up = false
      }
      if pointer == (arg-1).into() {
          up = true;
      }
      if up {pointer -= 1} else {pointer += 1}
    }
    Ok(rails.concat())
}

pub fn transform_rc_decode(input: &String, arg: u16) -> Result<String, CError> {
    let mut rails: Vec<String> = vec![String::new(); arg.into()];
    let mut lens: Vec<usize> = vec![0; arg as usize];
    let mut pointer: usize = 0;
    let mut up: bool = false;
    
    for _i in 0..input.len() {
        lens[pointer] += 1;
        if pointer == 0 {
            up = false
        }
        if pointer == (arg-1).into() {
            up = true;
        }
        if up {pointer -= 1} else {pointer += 1}
    }
    //place string in rails
    let mut offset = 0;
    for i in 0..rails.len() {
        rails[i] = input[offset..offset+lens[i]].to_string();
        offset += lens[i];
    }

    pointer = 0;
    up = false;
    let mut out = String::new();
    for _i in 0..input.len() {
        out.push(rails[pointer].chars().nth(0).unwrap());
        rails[pointer] = rails[pointer][1..].to_string();
        if pointer == 0 {
            up = false
        }
        if pointer == (arg-1).into() {
            up = true;
        }
        if up {pointer -= 1} else {pointer += 1}
    }
    Ok(out)
}

pub fn external_transform(input: &String, transform: &String, args: &Vec<&str>, depth: usize) -> Result<String, CError> {

    //search in dictionary for method with matching name. If so, load it in.
    //then, run interpreter on it.
    let mut hdir = dirs::home_dir().expect("Could not find home dir");
    hdir.push(".ciplang/methods");

    //now, place the compiled method there
    hdir.push(transform);
    hdir.set_extension("cpth");
    let path = match hdir.to_str() {
        Some(s) => s,
        None => {return Err(CError::from("Could not find transform"));}
    };
    let method = read_bin_file(&path.to_string());

    if depth > MAX_RECURSION_DEPTH {
        return Err(CError::from_slice("Maximum Recursion Depth exceeded"));
    }

    interpret(&method, input, args, depth+1)
}
