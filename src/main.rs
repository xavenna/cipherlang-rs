
use std::env;
use std::fs;
use std::io::Read;
use cipherlang::*;

mod compile;
use crate::compile::*;

mod interpret;
use crate::interpret::*;

mod transform;

/* This is an attempt to rewrite parts of cipherlang using rust.
   Status:
   Bytecode generator: compliant with original
   Interpreter: mostly compliant with original
   Transform Library: Mostly compliant with original
   Dictionary: Functional
   External Transforms: Not implemented
 */

//For now, only the original featureset will be implemented, not the v2 additions
//type Statement = Vec<Token>;


fn main() {
    //parse command-line arguments:
    let c_args = env::args();
    let mut infile: String = String::new();
    let mut outfile: String = String::new();
    let mut method_file: String = String::new();
    let mut method: String = String::new();
    let mut args: String = String::new();
    let mut define: bool = false;
    let mut use_file: bool = false;
    let mut help: bool = false;
    let mut version: bool = false;

    if env::args().len() == 1 {
        println!("Error: No method specified\n");
        std::process::exit(1);
    }

    let c_args: Vec<String> = c_args.collect();
    let c_args = &c_args[1..];

    for x in c_args.iter() {
        if x.len() == 0 {
            continue;
        }
        if x.len() >= 3 && &x[0..2] == "--" {
            match &x[2..] {
                "version" => {version = true;},
                "help" => {help = true;},
                _ => {eprintln!("Error: invalid argument {}",x) },
            }
        } else if x.chars().next().unwrap() == '-' {
            if x.len() < 2 {
                eprintln!("Error: invalid argument");
                std::process::exit(1);
            }
            //since length is guaranteed to be nonzero, this is safe
            match x.chars().nth(1).unwrap() {
                'a' => {args = x[2..].to_string()},
                'd' => {define = true},
                'f' => {use_file = true; method_file = x[2..].to_string()},
                'h' => {help = true;},
                'i' => {infile = x[2..].to_string()},
                'o' => {outfile = x[2..].to_string()},
                _ => {eprintln!("Error: invalid argument"); std::process::exit(1);},
            }

        }
        else if method.is_empty() {
            method = x.to_string();
        }
        else {
            eprintln!("Error: Only one non-flag argument may be provided\n");
            std::process::exit(1);
        }
    }
    // post-processing
    if help {
        eprintln!("cipherlang v0, made by xavenna.");
        eprint!("Usage: ciplang");
        eprintln!(" [-f]method [-h] {{[-d] | [-i<infile>] -o<outfile>] [-a<args>]}}");
        eprintln!("if infile isn't specified, input is taken from stdin.");
        eprintln!("if outfile isn't specified, outtput is taken from stdout.");
        eprint!("if -f is set, method is presumed to be a local file. Otherwise, ");
        eprintln!("method file is presumed to be in '~/.ciplang/methods'");
        eprintln!("if -d is set, method is added to dictionary.");
        eprintln!("-a specified arguments. args should be a comma-delimited list.");
        eprintln!("Use -h to see this menu");
        std::process::exit(0);
    } else if version {
        eprintln!("cipherlang-rs v0.0");
        eprintln!("Copyright (C) 2024 xavenna <xavenna.v@gmail.com>");
        eprintln!("This piece of software is released under the MIT License. See LICENSE for details.");
        eprintln!("There is NO WARRANTY, to the extent permitted by law.");
        eprintln!("Written by xavenna");

    } else {
        if use_file {
            method = method_file;
        }
        transform_text(&infile, &outfile, &method, &args, use_file, define);
    }
}

fn transform_text(infile: &String, outfile: &String, method_name: &String, args: &String, local: bool, define: bool) -> bool {
    eprintln!("Transforming text");
    let method: Vec<u8>;
    if local {
        //check for local file
        if method_name.is_empty() {
            eprintln!("Error: null local filename - invalid call");
            return false;
        }
        if let Ok(_) = fs::metadata(&method_name) {
            //method exists

        } else {
            println!("Error: could not open local file.\nIf you were trying to use a global method, omit the '-f' flag.");
            return false
        }
        let mod_date = fs::metadata(&method_name).unwrap().modified().unwrap();
        let cache_name = ".".to_string() + method_name + ".cpth";
        let cache_exists: bool = match fs::metadata(&cache_name) {
            Ok(_) => true,
            Err(_) => false,
        };


        //
        if cache_exists && fs::metadata(&cache_name).unwrap().modified().unwrap() >= mod_date {
            eprintln!("Using cached bytecode file");
            method = read_bin_file(&cache_name);
        } else {
            //compile
            let script = fs::read_to_string(&method_name).expect("File read error");
            method = match convert_to_method(&script) {
                Ok(s) => s,
                Err(s) => {eprintln!("{}", s);return false},
            };
            match fs::write(&cache_name, &method) {
                Ok(_) => {},
                Err(s) => {eprintln!("Cache Write Error: {s}");},
            }

        }
    } else {
        //check dir for method

        let mut hdir = dirs::home_dir().expect("Could not find home dir");
        hdir.push(".ciplang/methods");

        //now, place the compiled method there
        hdir.push(method_name);
        hdir.set_extension("cpth");
        let path = match hdir.to_str() {
            Some(s) => s,
            None => {return false;}
        };
        method = read_bin_file(&path.to_string());
    }
    // add method file to dictionary
    if define {
        let mut hdir = match dirs::home_dir() {
            Some(s) => s,
            None => { eprintln!("Failed to get home directory");return false; },
        };
        hdir.push(".ciplang/methods");
        if fs::create_dir_all(&hdir).is_err() {
            eprintln!("Failed to open dictionary directory");
            return false;
        }

        //now, place the compiled method there
        hdir.push(method_name);
        hdir.set_extension("cpth");
        if fs::write(&hdir, &method).is_err() {
            eprintln!("Failed to write method to dictionary");
            return false;
        } else {
            eprintln!("Successfully wrote method to dictionary");
        }
    }
    //call interpreter using procured method

    let mut input = String::new();
    if infile.is_empty() {
        let mut stdin = std::io::stdin();
        if let Err(s) = stdin.read_to_string(&mut input) {
            eprintln!("File write error: {s}");
            return false;
        }

    } else {
        input = match fs::read_to_string(&infile) {
            Ok(s) => s,
            Err(s) => {eprintln!("{s}");return false;},
        };
    };

    let mut cargs: Vec<&str> = args.split(',').collect();
    cargs.retain(|x| !x.is_empty());


    //set up recursion tracker
    let depth: usize = 0;
    let output = match interpret(&method, &input, &cargs, depth) {
        Ok(s) => s,
        Err(s) => {eprintln!("{}",s);return false;},
    };
    
    //decide where to write `output`
    if outfile.is_empty() {
        //uses stdout, not stderr, to let it work for scripts
        println!("{output}");
    } else {
        //write to outfile
        match fs::write(&outfile, &output) {
            Ok(_) => {},
            Err(s) => {eprintln!("Could not write to output file: {s}");return false;},
        }
    }

    false
}


