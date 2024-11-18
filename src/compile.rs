use std::collections::HashMap;
use snailquote::unescape;
use cipherlang::*;
type Statement = Vec<Token>;

pub fn convert_to_method(inp: &String) -> Result<Vec<u8>, CError> {
    //tokenize
    let lines: Vec<String> = inp.lines().map(|x| x.to_string()).collect();
    let mut tokens: Vec<Token> = tokenize(&lines)?;
    //compile to bytecode
    compile(&mut tokens)
}



pub fn tokenize(script: &Vec<String>) -> Result<Vec<Token>, CError> {
    //tokenize line by line
    let mut line_counter: usize = 1; //starts with 1 because code starts with line 1;
    let mut tokens: Vec<Token> = Vec::new();

    for line in script {
        if line.len() == 0 || line.chars().nth(0) == Some('#') {
            line_counter += 1;
            continue;
        }
        //tokenize line
        let mut k = tokenize_line(line, line_counter)?;
        tokens.append(&mut k);
        line_counter += 1;
    }

    //dbg!(&tokens);
    Ok(tokens)
}

pub fn tokenize_line(line: &String, count: usize) -> Result<Vec<Token>, CError> {

    let mut tokens: Vec<Token> = Vec::new();
    //run a state machine to split this into varieties
    let segments = parse(line, ' ');
    for s in segments.iter() {
        let mut t = Token::new();
        if s.len() < 1 {
            continue;
        }
        let c1 = s.chars().nth(0).unwrap();
        let cback = s.chars().nth_back(0).unwrap();
        if c1 == '_' {
            //special variable: Identifier
            t.ttype = TType::Identifier(s.to_string());
        } else if c1 == '$' {
            //operation
            if s.len() < 2 {
                return Err(CError::from(format!("Line {}: Null operations are not permitted", count)));
            }
            t.ttype = TType::Operation(s[1..].to_string());
        } else if c1 == '"' {
            //string constant
            //strparse value to handle escape characters
            if s.len() < 2 {
                return Err(CError::from(format!("Line {}: Invalid string constant", count)));
            }
            if let Ok(r) = unescape(s) {
                t.ttype = TType::Str(r);
            } else {
                return Err(CError::from(format!("Line {}: Invalid string escape code", count)));
            }
        } else if c1 == '!' {
            //file operation -- not implemented?
            return Err(CError::from(format!("Line {}: File operation -- File operations are not implemented", count)));
        } else if c1 == '%' {
            //directive
            if s.len() < 2 {
                return Err(CError::from(format!("Line {}: Empty directive line", count)));
            }
            t.ttype = TType::Directive(s[1..].to_string());
        } else if cback == '>' {
            //transform: extract transform name and arguments
            if let Some(n) = s.find('<') {
                // I don't know if this actually works... (rust strings are weird)
                let args = &s[n+1..s.len()-1];
                let argvec: Vec<String> = parse(args, ',');

                if !valid_transform_args(&argvec) {
                    return Err(CError::from(format!("Line {}: Invalid transform arguments", count)));
                }

                t.ttype = TType::Transform(s[0..n].to_string(), argvec);

            } else {
                return Err(CError::from(format!("Line {}: Invalid transform token",count)));
            }
        } else if s == "var" {
            t.ttype = TType::Var;
        } else if s == "const" {
            t.ttype = TType::Const;
        } else if s == "load" {
            t.ttype = TType::Load;
        } else if s == "write" {
            t.ttype = TType::Write;
        } else if s == "apply" {
            t.ttype = TType::Apply;
        } else if s == "to" {
            t.ttype = TType::To;
        } else if s == "from" {
            t.ttype = TType::From;
        } else {
            //attempt to determine type of token based on text contents
            //most likely identifier
            t.ttype = TType::Identifier(s.to_string());
        }
        t.line = count;
        tokens.push(t);

    }
    if segments.len() != 0 {
        let mut k = Token::new();
        k.ttype = TType::EndStatement;
        tokens.push(k);
    }
    Ok(tokens)
}

//this splits the tokens into a series of 
pub fn organize_tokens(tlist: &mut Vec<Token>) -> Result<Vec<Statement>, CError> {
    let mut statements: Vec<Statement> = Vec::new();
    
    let mut current: Statement = Vec::new();
    for x in tlist.iter() {
        match x.ttype {
            TType::EndStatement => {statements.push(current);current = Vec::new()},
            _ => {current.push(x.clone());},
        }
    }
    Ok(statements)
}

pub fn locate_vars(statements: &Vec<Statement>) -> Result<(Vec<String>, Vec<String>, HashMap<String, String> ), CError> {
    //in each statement, check for var/const declarations/usage
    //when var/const is declared, check if has been used yet. If so, error.
    //Else, add to appropriate table
    //
    //TODO: Add handling for special vars

    let mut vars: Vec<String> = Vec::new();
    let mut consts: Vec<String> = Vec::new();
    let mut cval: HashMap<String, String> = HashMap::new();
    for x in statements.iter() {
        if x.len() == 0 {
            //disregard empty statements
            continue;
        }
        if let TType::Var = x[0].ttype {
            //get declaration information
            if x.len() != 2 {
                return Err(CError::from(format!("Line {}: Malformed variable declaration statement", x[0].line)));
            }
            if let TType::Identifier(s) = &x[1].ttype {
                //search for identifier in var and const tables
                if s.len() > 0 && s.chars().nth(0).unwrap() == '_' {
                    return Err(CError::from(format!("Line {}: The '_' prefix for variable names is reserved", x[0].line)));
                }
                if vars.contains(s) {
                    return Err(CError::from(format!("Line {}: Error: Redeclaration of variable {}", x[0].line, s)));
                } else if consts.contains(s) {
                    return Err(CError::from(format!("Line {}: Error: Redeclaration of constant {}", x[0].line, s)));
                } else {
                    vars.push(s.clone());
                }
            } else {
                return Err(CError::from(format!("Line {}: Malformed variable declaration statement", x[0].line)));
            }
        } else if let TType::Const = x[0].ttype {
            //get declaration information
            if x.len() != 3 {
                return Err(CError::from(format!("Line {}: Malformed constant declaration statement", x[0].line)));
            }
            if let TType::Identifier(s) = &x[1].ttype {
                //search for identifier in var and const tables
                if s.len() > 0 && s.chars().nth(0).unwrap() == '_' {
                    return Err(CError::from(format!("Line {}: The '_' prefix for variable names is reserved", x[0].line)));
                }
                if vars.contains(s) {
                    return Err(CError::from(format!("Line {}: Error: Redeclaration of variable {}", x[0].line, s)));
                } else if consts.contains(s) {
                    return Err(CError::from(format!("Line {}: Error: Redeclaration of constant {}", x[0].line, s)));
                } else if let TType::Str(t) = &x[2].ttype {
                    consts.push(s.clone());
                    cval.insert(s.clone(), t.clone());
                }
            } else {
                return Err(CError::from(format!("Line {}: Malformed constant declaration statement", x[0].line)));
            }
        } else {
            //check if variables are used
            for tok in x {
                if let TType::Identifier(s) = &tok.ttype {
                    //check if s has been declared yet
                    if !consts.contains(&s) && !vars.contains(&s) {
                        if !is_valid_special_var(&s) {
                            return Err(CError::from(format!("Line {}: Identifier '{}' has not been declared", tok.line, s)));
                        }
                    }


                } else if let TType::Transform(n, a) = &tok.ttype {
                    if !value_in_str_map(&cval, &n).is_some() {
                        let name = format!("__cpth_cGenConst`{}", consts.len());
                        consts.push(name.clone());
                        cval.insert(name, n.to_string());
                    } // if this constant already exists,

                    //handle args
                    for i in 0..a.len() {
                        let t = &a[i];
                        //is t a constant or a number?
                        if let Ok(_) = t.trim().parse::<i16>() {

                        } else {
                            //t is a constant:
                            let name = format!("__cpth_cGenConst`{}", consts.len());
                            consts.push(name.clone());
                            cval.insert(name.clone(), t.to_string());

                            //replace the constant with the constant's name
                        }
                    }

                } else if let TType::Operation(s) = &tok.ttype {
                    if !value_in_str_map(&cval, &s).is_some() {
                        let name = format!("__cpth_cGenConst`{}", consts.len());
                        consts.push(name.clone());
                        cval.insert(name, s.to_string());
                    }
                }
                //otherwise, no constants to extract


            }
        }


    }
    //dbg!(&cval);
    Ok( (vars, consts, cval) )
}

pub fn consolidate(statements: &Vec<Statement>) -> Result<Vec<ProtoInstruction>, CError> {
    let mut proto: Vec<ProtoInstruction> = Vec::new();
    //for each statement:
    for st in statements {
        let mut x = st.clone();
        if x.len() == 0 { //ignore empty statements
            continue;
        }
        let mut p = ProtoInstruction::new();

        match &x[0].ttype {
            TType::Load => { //should be [load] [ident:...] [from] [source] where source can be an
                      //operation statement
                if x.len() < 4 || x.len() & 1 == 1 { // statement must have an even number of tokens
                                                   // to be a valid use of operations
                    return Err(CError::from(format!("Line {}: Error: Malformed load statement", x[0].line)));
                }

                if !x[1].is_ident() {
                    return Err(CError::from(format!("Line {}: Error: Malformed load statement - Missing first ident", x[0].line))); 
                }

                if !x[2].is_from() {
                    return Err(CError::from(format!("Line {}: Error: Malformed load statement - Missing from", x[0].line))); 
                }
                //this uses a weird version of a for loop, so implementing using while
                let mut i=x.len()-1;
                while i > 3 {
                    let mut pt = ProtoInstruction::new();
                    if !x[i].is_ident() {
                        return Err(CError::from(format!("Line {}: Error: Malformed load statement - missing final ident", x[0].line))); 
                    }
                    pt.pitype = PIType::Operation;
                    pt.source = match &x[i-2].ttype{
                        TType::Identifier(s) => s.to_string(),
                        _ => { return Err(CError::from(format!("Line {}: Error: not ident", x[0].line))); }
                    };
                    pt.second_source = match &x[i].ttype {
                        TType::Identifier(s) => s.to_string(),
                        _ => { return Err(CError::from(format!("Line {}: Error not ident", x[0].line)));}
                    };
                    pt.value = match &x[i-1].ttype {
                        TType::Operation(s) => s.to_string(),
                        _ => { return Err(CError::from(format!("Line {}: Error not op", x[0].line)));}
                    };
                    
                    x[i-2] = Token::new_val(TType::Identifier("_o".to_string()), x[i-2].line); //set s to "_o";
                    proto.push(pt);
                    i = i-2;
                }
                p.pitype = PIType::Load;
                p.target = match &x[1].ttype {
                    TType::Identifier(s) => s.to_string(),
                    _ => { return Err(CError::from(format!("Line {}: Error not ident", x[0].line)));},
                };
                p.source = match &x[3].ttype {
                    TType::Identifier(s) => s.to_string(),
                    _ => { return Err(CError::from(format!("Line {}: Error", x[0].line)));},
                };
                proto.push(p);
            },
            TType::Write => {
         // statement must have an even number of tokens to be a valid use of operations
                if x.len() < 4 || x.len() & 1 == 1 {
                    return Err(CError::from(format!("Line {}: Error: Malformed load statement", x[0].line)));
                }

                let mut i=x.len()-3;
                while i > 1 {
                    //create things
                    let mut pt = ProtoInstruction::new();
                    if !x[i].is_ident() || !x[i-1].is_oper() || !x[i-2].is_ident() {
                        return Err(CError::from(format!("Line {}: Error: Malformed write statement", x[0].line))); 
                    }
                    pt.pitype = PIType::Operation;
                    pt.source = match &x[i-2].ttype{
                        TType::Identifier(s) => s.to_string(),
                        _ => { return Err(CError::from(format!("Line {}: Error: not ident", x[0].line))); }
                    };
                    pt.second_source = match &x[i].ttype {
                        TType::Identifier(s) => s.to_string(),
                        _ => { return Err(CError::from(format!("Line {}: Error not ident", x[0].line)));}
                    };
                    pt.value = match &x[i-1].ttype {
                        TType::Operation(s) => s.to_string(),
                        _ => { return Err(CError::from(format!("Line {}: Error not op", x[0].line)));}
                    };
                    
                    x[i-2] = Token::new_val(TType::Identifier("_o".to_string()), x[i-2].line); //set s to "_o";
                    proto.push(pt);
                    i = i-2;
                }

                if !x[1].is_ident() {
                    return Err(CError::from(format!("Line {}: Error: Malformed write statement - Missing first ident", x[0].line))); 
                }

                if !x[x.len()-2].is_to() {
                    return Err(CError::from(format!("Line {}: Error: Malformed write statement - Missing to", x[0].line))); 
                }

                p.pitype = PIType::Load;
                p.target = match &x.last().unwrap().ttype {
                    TType::Identifier(s) => s.to_string(),
                    _ => { return Err(CError::from(format!("Line {}: Error not ident", x[0].line)));},
                };
                p.source = match &x[1].ttype {
                    TType::Identifier(s) => s.to_string(),
                    _ => { return Err(CError::from(format!("Line {}: Error", x[0].line)));},
                };
                proto.push(p);
            },
            TType::Apply => {
                if x.len() != 4 {
                    return Err(CError::from(format!("Line {}: Malformed apply statement", x[0].line)));
                }
                if !x[1].is_transform() || !x[2].is_to() || !x[3].is_ident() {
                    return Err(CError::from(format!("Line {}: Malformed apply statement", x[0].line)));
                }
                p.pitype = PIType::Apply;
                p.value = match &x[1].ttype {
                    TType::Transform(s, _) => s.to_string(),
                    _ => { return Err(CError::from(format!("Line {}: Error", x[0].line)));},
                };
                p.args = match &x[1].ttype {
                    TType::Transform(_, t) => t.to_vec(),
                    _ => { return Err(CError::from(format!("Line {}: Error", x[0].line)));},
                };
                p.arg_str = p.args.iter().map(|x| !x.parse::<f64>().is_ok()).collect();
                p.target = match &x[3].ttype {
                    TType::Identifier(s) => s.to_string(),
                    _ => { return Err(CError::from(format!("Line {}: Error", x[0].line)));},
                };
                proto.push(p);
            },

            //cipherlang v2 will add several new instructions here, but that's for later

            _ => {}, //var and const aren't part of bytecode

        }
    }
    //dbg!(&proto);
    Ok(proto)
}

pub fn resolve_references(proto: &Vec<ProtoInstruction>, variables: &Vec<String>, constants: &Vec<String>, constvals: &HashMap<String, String>) -> Result<Vec<ProtoInstruction>, String> {
    //go through each instruction, and create a new one
    let mut out: Vec<ProtoInstruction> = Vec::new();

    // search for identifiers
    for x in proto {
        let mut y = x.clone();
        match x.pitype {
            PIType::Load => {
                //source, target have idents
                let vn = get_reference_num(variables, &x.source);
                let cn = get_reference_num(constants, &x.source);
                let sn = get_special_var_num(&x.source);
                if vn == None {
                    if cn == None {
                        if sn == None {
                            return Err(format!("Invalid reference {}", &x.source));
                        } else {
                            y.source = (SPECIAL_VAR_OFFSET + sn.unwrap()).to_string();
                        }
                    } else {
                        y.source = (CONST_OFFSET + cn.unwrap()).to_string();
                    }
                } else {
                    y.source = (VAR_OFFSET + vn.unwrap()).to_string();
                }

                let vn = get_reference_num(variables, &x.target);
                let cn = get_reference_num(constants, &x.target);
                let sn = get_special_var_num(&x.target);
                if vn == None {
                    if cn == None {
                        if sn == None {
                            return Err(format!("Invalid reference (somehow) uh oh"));
                        } else {
                            y.target = (SPECIAL_VAR_OFFSET + sn.unwrap()).to_string();
                        }
                    } else {
                        return Err(format!("Error: Illegal write to constant"));
                    }
                } else {
                    y.target = (VAR_OFFSET + vn.unwrap()).to_string();
                }

            },
            PIType::Apply => {
                //target has identifier, args may have identifier
                let vn = get_reference_num(variables, &x.target);
                let cn = get_reference_num(constants, &x.target);
                let sn = get_special_var_num(&x.target);
                if vn == None {
                    if cn == None {
                        if sn == None {
                            return Err(format!("Invalid reference (somehow) uh oh"));
                        } else {
                            y.target = (SPECIAL_VAR_OFFSET + sn.unwrap()).to_string();
                        }
                    } else {
                        return Err(format!("Error: Illegal write to constant"));
                    }
                } else {
                    y.target = (VAR_OFFSET + vn.unwrap()).to_string();
                }

                // val should be in const table, find it
                let cname = match value_in_str_map(&constvals, &x.value) {
                    Some(s) => s,
                    None => {return Err(format!("Invalid constant uh oh"));},
                };
                if let Some(s) = index_of_vec_val(&constants, &cname) {
                    y.value = (CONST_OFFSET as usize + s).to_string();
                } else {
                    return Err(format!("Error: invalid constant ..."));
                }

                // each arg should be in const table, find it
                for (i, a) in x.args.iter().enumerate() {
                    //check if arg is an int
                    if x.arg_str[i] {
                        let cname = match value_in_str_map(&constvals, &a) {
                            Some(s) => s,
                            None => {return Err(format!("Invalid constant {}", &a));},
                        };
                        if let Some(s) = index_of_vec_val(&constants, &cname) {
                            y.args[i] = (CONST_OFFSET as usize + s).to_string();
                        } else {
                            return Err(format!("Error: invalid constant ..."));
                        }
                    }
                }
            },
            PIType::Operation => {
                //source & secondSource have identifiers, name is a constant
                let vn = get_reference_num(variables, &x.source);
                let cn = get_reference_num(constants, &x.source);
                let sn = get_special_var_num(&x.source);
                if vn == None {
                    if cn == None {
                        if sn == None {
                            return Err(format!("Invalid reference (somehow) uh oh"));
                        } else {
                            y.source = (SPECIAL_VAR_OFFSET + sn.unwrap()).to_string();
                        }
                    } else {
                        y.source = (CONST_OFFSET + cn.unwrap()).to_string();
                    }
                } else {
                    y.source = (VAR_OFFSET + vn.unwrap()).to_string();
                }

                let vn = get_reference_num(variables, &x.second_source);
                let cn = get_reference_num(constants, &x.second_source);
                let sn = get_special_var_num(&x.second_source);
                if vn == None {
                    if cn == None {
                        if sn == None {
                            return Err(format!("Invalid reference (somehow) uh oh"));
                        } else {
                            y.second_source = (SPECIAL_VAR_OFFSET + sn.unwrap()).to_string();
                        }
                    } else {
                        y.second_source = (CONST_OFFSET + cn.unwrap()).to_string();
                    }
                } else {
                    y.second_source = (VAR_OFFSET + vn.unwrap()).to_string();
                }

                // val should be in const table, find it
                let cname = match value_in_str_map(&constvals, &x.value) {
                    Some(s) => s,
                    None => {return Err(format!("Invalid constant uh oh"));},
                };
                if let Some(s) = index_of_vec_val(&constants, &cname) {
                    y.value = (CONST_OFFSET as usize + s).to_string();
                } else {
                    return Err(format!("Error: invalid constant ..."));
                }

            },
            // cipherlang v2 instructions go here
            _ => {}, //if nothing to change, keep unchanged
        }
        out.push(y);

    }
    //dbg!(&out);
    Ok(out)
}

pub fn generate_text(bin: &Vec<BinaryInstruction>) -> Result<Vec<u8>, String> {
    let mut text: Vec<u8> = Vec::new();
    for x in bin {
        let mut p = match x.binary() {
            Ok(s) => s,
            Err(s) => {return Err(s);},
        };
        text.append(&mut p);
    }
    Ok(text)
}

pub fn generate_const(values: &HashMap<String, String>, constants: &Vec<String>) -> Result<Vec<u8>, String> {
    let total_char_len: usize = values.iter().map(|(_, v)| v.len()+1).sum();

    let mut table: Vec<u8> = Vec::with_capacity(total_char_len + CONST_HEADER_WIDTH as usize* values.len());
    let mut pos: u16 = 0;
    for x in constants {
        if !values.contains_key(x) {
            return Err(format!("Error: undeclared constant {}", x));
        }
        let y = &values[x];
        if y.len() > MAX_CONST_LEN.try_into().unwrap() {
            return Err(format!("Error: Constant {} exceeds max constant length", x));
        }
        table.push( (((pos + CONST_HEADER_WIDTH * values.len() as u16) & 0xff00) >> 8) as u8);
        table.push( ((pos + CONST_HEADER_WIDTH * values.len() as u16) & 0xff) as u8);
        pos += (y.len()+1) as u16;

    }
    //now place string constants in table
    for x in constants {
        let y = &values[x];
        for z in y.bytes() {
            table.push(z);
        }
        table.push(0);
    }
    Ok(table)
}

pub fn create_header(argmin: i32, argmax: i32, num_consts: usize, text_len: usize, var_num: usize) -> Result<Vec<u8>, String> {
    let mut header: Vec<u8> = Vec::with_capacity(16);
    header.resize(16, 0);
    header[0] = 'C' as u8;
    header[1] = 'P' as u8;
    header[2] = 'T' as u8;
    header[3] = 'H' as u8;
    header[4] = MAJOR_VERSION;
    header[5] = MINOR_VERSION;
    header[6] = PATCH_NUM;
    header[7] = 0x0;
    header[8] = argmin as u8;
    header[9] = argmax as u8;
    header[0xa] = CONST_HEADER_WIDTH as u8;
    header[0xb] = num_consts as u8;
    header[0xc] = (((HEADER_LEN + text_len) & 0xff00) >> 8) as u8;
    header[0xd] = ((HEADER_LEN + text_len) & 0xff) as u8;
    header[0xe] = var_num as u8;
    header[0xf] = 0x0;
    Ok(header)
}
pub fn assemble(header: &Vec<u8>, text: &Vec<u8>, data: &Vec<u8>) -> Result<Vec<u8>, CError>{
    let total = header.len() + text.len() + data.len();
    eprintln!("header : 0x{:x};  text : 0x{:x};  data : 0x{:x};  total : 0x{:x}", header.len(), text.len(), data.len(), total);
    let mut table: Vec<u8> = Vec::new();
    table.extend(header.iter());
    table.extend(text.iter());
    table.extend(data.iter());
    Ok(table)
}

//takes an identifier, and replaces it with a numerical reference
pub fn get_reference_num(ident_list: &Vec<String>, ident: &String) -> Option<u16> {
    for (i, x) in ident_list.iter().enumerate() {
        if x == ident {
            return Some((i) as u16);
        }
    }
    None
}

//special var space map:
//0x300 - 0x33f: general use variables
//0x340 - 0x35f: arguments
//this should use Option<u16> or something like that
pub fn get_special_var_num(name: &str) -> Option<u16> {
    //values contains any constant values. Some, like _# have a non-constant name
    let values = vec!["_", "_o", "_randU", "_randL", "_randE", "_randN", "_randA",
    "_argc", "_stdin", "_stdout", "_null", "_c", "_k", "_cs", "_cc", "_loc"];
    for (i, _) in values.iter().enumerate() {
        if values[i] == name {
            return Some(i.try_into().unwrap());
        }
    }
    if &name[0..1] == "_" {
        if let Ok(s) = name[1..].parse::<u16>() {
            if s < 0x20 {
                return Some(s + 0x40);
            }
        }
        //other dynamic special vars go here
        
    }
    None
}

pub fn is_valid_special_var(name: &str) -> bool {
    if name.len() == 0 {
        return false;
    }
    let values = vec!["_", "_o", "_randU", "_randL", "_randE", "_randN", "_randA",
    "_argc", "_stdin", "_stdout", "_null", "_c", "_k", "_cs", "_cc", "_loc", "__len#", "_#"];
    if values.contains(&name) {
        return true;
    }
    if &name[0..1] == "_" {
        if let Ok(s) = name[1..].parse::<u16>() {
            if s < 0x20 {
                return true;
            }
        }
    }
    false
}

pub fn valid_transform_args(args: &Vec<String>) -> bool {

    for _x in args.iter() {
        //should be a number or a string constant. I'm not sure how to validate this, but
        //it's here

    }
    true
}

// Compiles a list of tokens to a method (bytecode)
pub fn compile(tlist: &mut Vec<Token>) -> Result<Vec<u8>, CError> {
    eprintln!("Compiling Method");

    //check for directives, verify req'd args
    let mut argmax: i32=-1;
    let mut argmin: i32=-1;
    let mut directives: Vec<String> = Vec::new();
    for x in tlist.iter() {
        if let TType::Directive(s) = &x.ttype {
            let mut k = parse(&s, ',');
            directives.append(&mut k);     
        }
    }
    
    //now, remove all directive tokens from tlist
    tlist.retain(|v| if let TType::Directive(_) = v.ttype {false}else{true});


    //handle directives
    for x in directives.iter() {
        if x.len() < 3 {
            return Err(CError::from_slice("Directive is of insufficient size")); 
        }
        match x.find('=') {
            Some(s) => {
                if s == 0 {
                    return Err(CError::from_slice("Directive is missing a name"));
                } else if s == directives.len() {
                    return Err(CError::from_slice("Directive is missing a key"));
                }
                else {
                    let name = &x[0..s];
                    let value = &x[s+1..];
                    if name == "argmin" {
                        argmin = value.parse()?
                    } else if name == "argmax" {
                        argmax = value.parse()?
                    } else {
                        return Err(CError::from(format!("Invalid directive key {}", name)));
                    }
                }
            },
            None => {return Err(CError::from_slice("Directive is missing an equal sign"))},
        }
    }
    if argmin < 0 || argmax < 0 {
        return Err(CError::from_slice("argmax and argmin must be specified"));
    }

    //organize tokens into expressions
    
    let statements = organize_tokens(tlist)?;
    //create an index of variables and consts

    let (variables, constants, constvals) = locate_vars(&statements)?;

    //convert statements into proto-instructions
    // instructions is mutable to it can be edited by resolveReferences

    let instructions = consolidate(&statements)?;

    //resolve references to vars and consts
    let instructions = resolve_references(&instructions, &variables, &constants, &constvals)?;

    //check for redundant proto-instructions

    //convert proto-instructions to binary instructions
    let binary_ins: Vec<BinaryInstruction> = instructions.iter().map(
        |x| x.binarify().unwrap()
    ).collect();

    //generate text section of method
    let text = generate_text(&binary_ins)?;
    
    //generate constants table of method
    let data = generate_const(&constvals, &constants)?;

    //generate method header
    let header = create_header(argmin, argmax, constants.len(), text.len(), variables.len())?;

    //assemble into a complete method file
    assemble(&header, &text, &data)
}
