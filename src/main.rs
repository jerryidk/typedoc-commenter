/*
 * Document all decorator
 * with typedoc like syntax
 *
 *  TODO:
 *
 *  Add autogenerate comment and prevent double commenting
 *  Able to change @... rather than inserting
 *
 *  New Features:
 *
 *  Flag --update
 */
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs::rename;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::vec::Vec;
use walkdir::WalkDir;

lazy_static! {
    static ref DECORATOR: Regex = Regex::new(r"^[@].+$").unwrap();
    static ref VARIABLE: Regex = Regex::new(r"^[a-zA-Z?]+[:].+$").unwrap();
    static ref CLASS: Regex = Regex::new(r"^class\s[a-zA-Z{}]+$").unwrap();
    static ref COMMENT_DECORATOR: Regex = Regex::new(r"^*\s[@].+$").unwrap();
    static ref EXPORT: Regex =
        Regex::new(r#"^export\s[*a-zA-Z"]+\sfrom\s"([a-zA-Z./]+)";$"#).unwrap();
}

/**
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
                let files = WalkDir::new(arg).into_iter().filter_map(|file| file.ok());

                for file in files {
                    if file.metadata().unwrap().is_file() {
                        let src = file.path().to_str().unwrap().to_owned();
                        let dst = format!("{}-new", src);
                        addcomment(&src, &dst)?;
                        rename(dst, src)?;
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
 * 
 * How LOCK and UNLOCK works:
 * 
 * Surround decorator @... with //LOCK and //UNLOCK to prevent being in the comment
 * e.g. 
 * 
 *     //LOCK
 *     @crazyDecorator()
 *     //UNLOCK
 *     private crazyVariable;
 *  
 * This prevents @crazyDecorator to be generated in the comment.
 * 
 * TODO: 
 * 
 * Keep track of the comment string.
 * Add another list keeps track of "* @...", and compare it to decorators list,
 * every time upon write out to decide if comment string or out_string should be written out.
 *  
 */
fn addcomment(src: &String, dst: &String) -> Result<(), Error> {
    let input = File::open(src)?;
    let mut output = File::create(dst)?;
    let buffered = BufReader::new(input);

    let mut out_string: String;
    let mut decorators: Vec<String> = Vec::new();

    let mut locked: bool = false;

    for line in buffered.lines() {
        let copy: String = line?.to_owned();
        let trim: String = String::from(copy.trim());

        //Check if the decorator is locked.
        if trim == "//LOCK" {
            locked = true;
        } else if trim == "//UNLOCK" {
            locked = false;
        }

        if DECORATOR.is_match(&trim) && !locked {
            //accumulate all decorators associate with the variable and class
            //don't add to the list if it is locked.
            decorators.push(copy);
        } else if (VARIABLE.is_match(&trim) || CLASS.is_match(&trim)) && !decorators.is_empty() {
            //writing out comments
            out_string = String::from("/**\n * \n * Decorator Usage:\n");
            for d in &decorators {
                let d = format!(" * {} \n", d.trim());
                out_string.push_str(&d);
            }
            out_string.push_str(" */\n");

            //writing out decorators
            out_string.push_str("//LOCK\n");
            for d in &decorators {
                let d = format!("{}\n", d);
                out_string.push_str(&d);
            }
            out_string.push_str("//UNLOCK\n");

            //writing out variable or class name
            let var = format!("{}\n", copy);
            out_string.push_str(&var);

            //clearing out the buffer
            decorators = Vec::new(); 
            write!(output, "{}", &out_string)?;
        } else {
            //write out whatever the line is
            out_string = format!("{}\n", copy);
            write!(output, "{}", &out_string)?;
        }
    }

    Ok(())
}
