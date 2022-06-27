/*
 * Document all decorator
 * with typedoc like syntax
 *
 * It will override the original comment in format /** ... */
 * Bug:
 * Don't support multiline
 */
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs::rename;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::vec::Vec;
use walkdir::{DirEntry, WalkDir};

lazy_static! {
    static ref DECORATOR: Regex = Regex::new(r"^[@].+$").unwrap();
    static ref MLDECORATOR: Regex = Regex::new(r"^[@][a-zA-Z0-9]+\([^\)]*$").unwrap();
    static ref MLDECORATOREND: Regex = Regex::new(r"^.*\)$").unwrap();
    static ref VARIABLE: Regex = Regex::new(r"^[a-zA-Z?]+[:].+$").unwrap();
    static ref CLASS: Regex = Regex::new(r"^class\s[a-zA-Z{}]+$").unwrap();
    static ref COMMENT: Regex = Regex::new(r"^[\*].*$").unwrap();
}

/*
 *
 * Command line args:
 *
 * 1. gives a Dir ---> Autogenerate comment for all files under the Directory (recursively)
 * 2. gives a File ---> Autogenerate comment for the given file
 * 3. gives a Src File and Dst File ---> Autogenerate comment for the Dst file
 *
 */
fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let arg = &args[1];

            if &arg[arg.len() - 1..] != "/" {
                let src = args[1].to_owned();
                let dst = format!("{}-new", src);
                addcomment(&src, &dst)?;
                rename(dst, src)?;

                print!("{}", "transformaing a file...");
            } else {
                fn is_hidden(entry: &DirEntry) -> bool {
                    entry
                        .file_name()
                        .to_str()
                        .map(|s| s.starts_with("."))
                        .unwrap_or(false)
                }

                let files = WalkDir::new(arg)
                    .into_iter()
                    .filter_entry(|e| !is_hidden(e))
                    .filter_map(|file| file.ok());

                for file in files {
                    if file.metadata().unwrap().is_file() {
                        let src = file.path().to_str().unwrap().to_owned();

                        if src.ends_with(".dto.ts") {
                            let dst = format!("{}-new", src);
                            addcomment(&src, &dst)?;
                            rename(dst, src)?;
                        }
                    }
                }
                print!("{}", "transformaing a dir...");
            }
        }

        3 => {
            let src = &args[1];
            let dst: &String = &args[2];
            addcomment(src, dst)?;

            print!("{}", "transformaing a file to another file...");
        }

        _ => {
            panic!("ops, unsupported command line args length")
        }
    }

    Ok(())
}

/**
 * Add comment for all decorators for given src file and output to dst
 *
 * How the method works:
 *
 * It looks for decorators @... symbol line by line and add them to a list.
 * When it sees variable or class symbol, it will writes out comments and original code.
 * If it doesn't see any of the above, it just copy the file content on that line.
 */
fn addcomment(src: &String, dst: &String) -> Result<(), Error> {
    let input = File::open(src)?;
    let mut output = File::create(dst)?;
    let buffered = BufReader::new(input);

    let mut out_string: String;
    let mut decorators: Vec<String> = Vec::new();

    let mut ml_decorator: String = String::from("");
    let mut ml: bool = false;
    let mut in_comment: bool = false;
    
    for line in buffered.lines() {
        let copy: String = line?.to_owned();
        let trim: String = String::from(copy.trim());

        //check if we are in comment or out of comment
        if trim == "/**" {
            in_comment = true;
            continue;
        }
        if trim == "*/"{
            in_comment = false;
            continue;
        }
        
        if trim == "\n"{
            continue;
        }

        if in_comment {
            continue;
        }
        else if ml {
            if MLDECORATOREND.is_match(&trim) {
                ml_decorator.push_str(&copy);
                let copy = ml_decorator.clone();
                decorators.push(copy);
                ml = false;
            }else
            {
                let f = format!("{}\n", &copy); 
                ml_decorator.push_str(&f);
            }
        } 
        else {
            if MLDECORATOR.is_match(&trim) {
                ml = true;
                ml_decorator = String::from("");
                let f = format!("{}\n", &copy);
                ml_decorator.push_str(&f);
            } else if DECORATOR.is_match(&trim) {
                //accumulate all decorators associate with the variable and class
                //don't add to the list if it is locked.
                decorators.push(copy);
            } else if (VARIABLE.is_match(&trim) || CLASS.is_match(&trim)) && !decorators.is_empty()
            {
                //writing out comments
                out_string = String::from("/**\n * Decorator Usage:\n * ```\n");
                for d in &decorators {
                    let d = format!(" * {} \n", d.trim());
                    out_string.push_str(&d);
                }
                out_string.push_str(" * ```\n */\n");

                //writing out decorators
                for d in &decorators {
                    let d = format!("{}\n", d);
                    out_string.push_str(&d);
                }

                //writing out variable or class name
                let var = format!("{}\n", copy);
                out_string.push_str(&var);

                //clearing out the buffer
                decorators = Vec::new();
                write!(output, "{}", &out_string)?;
            } else {
                out_string = format!("{}\n", copy);
                write!(output, "{}", &out_string)?;
            }
        }
    }

    Ok(())
}
