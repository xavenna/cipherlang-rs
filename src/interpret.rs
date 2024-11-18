use cipherlang::*;

use crate::transform::*;

pub fn get_const_offset(method: &Vec<u8>, offset: u16, num: u16) -> Result<u16,CError> {
    if method[0xb] as u16 <= num {
        return Err(CError::from_slice("Error: out-of-bounds const read"));
    }
    Ok(
    offset + ( ((method[ (offset + (CONST_HEADER_WIDTH*num) ) as usize] as u16) << 8) |
            (method[( offset + (CONST_HEADER_WIDTH*num) + 1 ) as usize] as u16) )
    )
}

pub fn get_const(method: &Vec<u8>, offset: u16, num: u16) -> Result<String, CError> {
    let t = get_const_offset(&method, offset, num)? as usize;
    let mut end = t;
    while method[end] != 0 {
        end += 1;
    }
    match String::from_utf8(method[t..end].to_vec()) {
        Ok(s) => Ok(s),
        Err(_) => Err(CError::from_slice("Error: Could not get constant")),
    }
}

pub fn interpret(method: &Vec<u8>, input: &String, args: &Vec<&str>, depth: usize) -> Result<String, CError> {
    dbg!(&args);
    let len = method.len();
    if len < (16 as usize) {
        return Err(CError::from(format!("Error: Invalid method file")));
    }
    if method[0] != 'C' as u8  ||  method[1] != 'P' as u8 || method[2] != 'T' as u8 || method[3] != 'H' as u8 || method[7] != 0x0 {
        return Err(CError::from(format!("Error: Invalid header")));
    }
    let const_offset: usize = ((method[0xc] as usize) << 8) | (method[0xd] as usize);
    let argmin = method[0x8];
    let argmax = method[0x9];
    let const_wid = method[0xa];
    let num_consts = method[0xb];
    let num_vars = method[0xe];

    if args.len() > argmax.into() || args.len() < argmin.into() {
        return Err(CError::from(format!("Error: incorrect argument number {}: max: {}, min: {}", args.len(), argmax, argmin)));
    }
    let mut last_tr = String::new(); //result of last transform
    let mut last_op = String::new(); //result of last operation

    let mut output = String::new(); //output string
    let mut vars: Vec<String> = vec![String::new(); num_vars.into()]; //variables
    let mut count: usize = 0x10; //???

    let mut in_ptr = 0;
    loop {
        let inst: Vec<u8> = match method[count] {
            0 => {
                method[count..count+5].to_vec()
            },
            1 => {
                method[count..(count+6 + (2 * method[count+5]) as usize ) ].to_vec()
            },
            2 => {
                method[count..count+7].to_vec()
            },
            _ => {return Err(CError::from(format!("Error: unrecognized opcode '0x{:X}'", method[count])));},
        };
        eprintln!("Running inst {:?}", inst);

        //execute the instruction
        let first: u16 = ((method[count+1] as u16) << 8) | method[count+2] as u16;
        let second: u16 = ((method[count+3] as u16) << 8) | method[count+4] as u16;


        //this is where the instruction logic goes
        match inst[0] {
            0 => { //load
                let tempvar: String;
                if first < 0x100 { //standard variable
                    if vars.len() >= first as usize {
                        tempvar = vars[first as usize].clone();
                    } else {
                        return Err(CError::from(format!("Error: out-of-range var read")));
                    }
                } else if first < 0x300 { //constant
                    if first - 0x100 >= num_consts as u16 {
                        return Err(CError::from(format!("Error: out-of-bounds const read")));
                    }
                    let t = get_const_offset(&method, const_offset as u16, first-0x100)? as usize;
                    let mut end = t;
                    while method[end] != 0 {
                        end += 1;
                    }
                    tempvar = String::from_utf8(method[t..end].to_vec())?;
                } else if first < 0x400 { //special var
                    tempvar = read_special_var(first - 0x300, &last_tr, &last_op, &input, &mut in_ptr, &args)?;

                } else {
                    return Err(CError::from(format!("Error: invalid identifier number")));
                }
                eprintln!("Writing '{}'",&tempvar);

                //destination
                if second < 0x100 { //normal variable
                    if vars.len() <= second as usize {
                        return Err(CError::from(format!("Error: out-of-bounds variable write")));
                    } 
                    vars[second as usize] = tempvar;
                } else if second < 0x300 {
                    return Err(CError::from(format!("Error: constant writes are prohibited")));
                } else if second < 0x400 {
                    write_special_var(second - 0x300, &tempvar[..], &mut output)?;
                } else {
                    return Err(CError::from(format!("Error: invalid identifier number")));
                }

                //find dest
            },
            1 => { //apply
                //load tempVar with target
                let tempvar: String;
                let varnum: usize; //which varnum to write to
                if first < 0x100 { //variable
                    if vars.len() >= first as usize {
                        tempvar = vars[first as usize].clone();
                        varnum = first as usize;
                    } else {
                        return Err(CError::from(format!("Error: out-of-bounds variable read")));
                    }
                } else if first < 0x300 {
                    //const
                    return Err(CError::from(format!("Error: Cannot transform constant")));
                } else if first < 0x400 {
                    //special var
                    return Err(CError::from(format!("Error: Cannot transform special var")));
                } else {
                    return Err(CError::from(format!("Error: out-of-bounds identifier")));
                }

                //load transform name into a variable
                let transform: String;
                if second < 0x100 { //variable
                    return Err(CError::from(format!("Error: cannot use variable for transform name")));
                } else if second < 0x300 { //constant
                    if second - 0x100 >= num_consts as u16 {
                        return Err(CError::from(format!("Error: out-of-bounds const read")));
                    }
                    transform = match get_const(&method, const_offset as u16, second - 0x100) {
                        Ok(s) => s,
                        Err(_) => {return Err(CError::from(format!("Error: could not read const")));},
                    };
                } else if second < 0x400 { //special var
                    return Err(CError::from(format!("Error: cannot use special var for transform name")));
                } else {
                    return Err(CError::from(format!("Error: invalid identifier")));
                }

                //figure out args:
                let argc = method[count+5] as usize;
                let mut args: Vec<String> = Vec::new();
                for i in 0..argc {
                    let arg: u16 = ((method[count+6 + 2*i] as u16) << 8) |
                        (method[count+7+2*i] as u16);
                    let form: u16 = (arg & 0xfc00) >> 10;
                    let value: u16 = arg & 0x3ff;
                    if form == 0x10 {
                        //const argument
                        let v = get_const(&method, const_offset as u16, value-0x100)?;
                        args.push(v);
                    } else if form == 0x00 {
                        // positive num argument
                        args.push(value.to_string());
                    } else if form == 0x01 {
                        args.push("-".to_owned() + &value.to_string());
                    } else {
                        return Err(CError::from(format!("Error: invalid argument type {:X}", form)));
                    }

                }
                eprintln!("Performing transform {} with args: {:?}", &transform, &args);

                
                //perform transform, write back to specified var
                // change args from Vec<String> to Vec<&str>
                let argstr: Vec<&str> = args.iter().map(|x| &x[..]).collect();
                let tempvar = apply_transform(&tempvar, &transform, &argstr, depth)?;

                //write tempvar to a place
                last_tr = tempvar.clone();
                vars[varnum] = tempvar;
            },
            2 => { //operation
                let tempvar: String;
                let secondvar: String;
                if first < 0x100 { //variable
                    if first as usize >= vars.len() {
                        return Err(CError::from(format!("Error: Invalid variable read")));
                    }
                    tempvar = vars[first as usize].clone();

                } else if first < 0x300 { //const
                    if first - 0x100 >= num_consts.into() {
                        return Err(CError::from(format!("Error: out-of-bounds const read")));
                    }
                    tempvar = match get_const(&method, const_offset as u16, first - 0x100) {
                        Ok(s) => s,
                        Err(_) => {return Err(CError::from(format!("Error: could not read const")));},
                    };
                } else if first < 0x400 { //special var
                    tempvar = read_special_var(first - 0x300, &last_tr, &last_op, &input, &mut in_ptr, &args)?;
                } else {
                    return Err(CError::from(format!("Invalid identifier")));
                }


                if second < 0x100 { //variable
                    if second as usize >= vars.len() {
                        return Err(CError::from(format!("Error: Invalid variable read")));
                    }
                    secondvar = vars[second as usize].clone();

                } else if second < 0x300 { //const
                    if second - 0x100 >= num_consts.into() {
                        return Err(CError::from(format!("Error: out-of-bounds const read")));
                    }
                    secondvar = match get_const(&method, const_offset as u16, second - 0x100) {
                        Ok(s) => s,
                        Err(_) => {return Err(CError::from(format!("Error: could not read const")));},
                    };
                } else if second < 0x400 { //special var
                    secondvar = read_special_var(second - 0x300, &last_tr, &last_op, &input, &mut in_ptr, &args)?;
                } else {
                    return Err(CError::from(format!("Invalid identifier")));
                }

                let third: u16 = ((method[count+5] as u16) << 8) | (method[count+6] as u16);
                let operation: String;
                if third < 0x100 || third >= 0x300 {
                    return Err(CError::from(format!("Error: attempt to use var as operation name")));
                } else {
                    if third - 0x100 >= num_consts as u16 {
                        return Err(CError::from(format!("Error: out-of-bounds const read")));
                    }
                    operation = match get_const(&method, const_offset as u16, third - 0x100) {
                        Ok(s) => s,
                        Err(_) => {return Err(CError::from(format!("Error: const read failure")));},
                    };
                }
                eprintln!("Applying oper {} on {} and {}", operation, tempvar, secondvar);
                let result = apply_operation(&tempvar, &secondvar, &operation)?;
                last_op = result;


            },

            _ => {return Err(CError::from(format!("Error: unrecognized opcode '0x{:X}'", method[count])));},
        }

        count += inst.len();

        if count >= const_offset {
            eprintln!("Interpretation successful\n=====================\n");
            return Ok(output);
        }
    }
}

fn read_special_var(num: u16, last_tr: &String, last_op: &String, input: &String, in_ptr: &mut usize, args: &Vec<&str>) -> Result<String, CError> {
    match num {
        0 => { // "_"
            Ok(last_tr.clone())
        },
        1 => { // "_o"
            Ok(last_op.clone())
        },
        8 => { // "_stdin"
            if *in_ptr >= input.len() {
                Err(CError::from(format!("Error: exceeded input text: ptr {}, len {}", *in_ptr, input.len())))
            } else {
                Ok(
                match input[*in_ptr..].find('\n') {
                    Some(s) => {let t = *in_ptr; *in_ptr += 1 + s;
                        input[t..s].to_string()},
                    None => {let t=input[*in_ptr..].to_string();*in_ptr = input.len();t},
                }
                )
            }
        }
        9 => { // "_stdout"
            Err(CError::from(format!("Error: cannot read from stdout")))
        },
        0xa => { // "_null"
            Ok(String::new())
        }
        0xb => { // "_c" : returns an arbitrary character
            Ok(String::from(" "))
        },
        0x40..=0x5f => { //arg #-0x40
            if args.len() <= num as usize - 0x40 {
                Ok(String::new())
            } else {
                Ok(args[num as usize - 0x40].to_string())
            }
        },
        _ => Err(CError::from(format!("That operation is not supported yet"))),
    }
}

fn write_special_var(num: u16, value: &str, output: &mut String) -> Result<(), CError> {
    match num {
        9 => {
            if !value.is_empty() {
                *output += value;
                output.push('\n');
            }
            Ok(())
        },
        _ => {return Err(CError::from(format!("Error: cannot write to specified special var")));},
    }
}

fn apply_transform(input: &String, transform: &String, args: &Vec<&str>, depth: usize) -> Result<String, CError> {
    //if type is a built-in transform, execute it:
    match &transform[..] {
        "upper" => {
            transform_upper(input)
        },
        "lower" => {
            transform_lower(input)
        },
        "trim_numeric" => {
            transform_trim_numeric(input)
        },
        "trim_alpha" => {
            transform_trim_alpha(input)
        },
        "trim_special" => {
            transform_trim_special(input)
        },
        "trim_whitespace" => {
            transform_trim_whitespace(input)
        },
        "prune" => {
            transform_prune(input)
        },
        "prune_numeric" => {
            transform_prune_numeric(input)
        },
        "prune_ascii" => {
            transform_prune_ascii(input)
        },
        "shift" => {
            if args.len() < 1 || args[0].parse::<i16>().is_err() {
                return Err(CError::from(format!("Shift requires a numeric argument")));
            }
            //args[0] needs to be coerced to a i16
            transform_shift(input, args[0].parse::<i16>().unwrap()) //unsafe
        },
        "rc" => {
            if args.len() != 1 || args[0].parse::<u16>().is_err() {
                return Err(CError::from(format!("rc encode requires a numeric argument")));
            }
            transform_rc_encode(input, args[0].parse().unwrap())
                
        },
        "rc_dec" => {
            if args.len() != 1 || args[0].parse::<u16>().is_err() {
                return Err(CError::from(format!("rc decode requires a numeric argument")));
            }
            transform_rc_decode(input, args[0].parse().unwrap())
        },
        _ => { //not a built-in transform
               //check for transforms in the dictionary
            external_transform(input, transform, args, depth)
        },
    }
}

fn apply_operation(in1: &String, in2: &String, op: &String) -> Result<String, String> {
    match &op[..] {
        "cat" => {
            let t = in1.to_string() + in2;
            Ok(t)
        },
        "eq" => {
            Ok(if in1 == in2 { String::new() } else {"false".to_string()})
        },
        "repeat" => {
            if let Ok(s) = in2.parse::<usize>() {
                Ok(in1.repeat(s))
            } else {
                Err(format!("Error: Invalid argument to $repeat"))
            }
        },
        _ => {
            return Err(format!("Error: invalid operation '{}'",op));
        },
    }
}
