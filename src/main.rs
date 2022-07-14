/*
 * Document all decorator
 * with typedoc like syntax
 *
 */
use diffy::create_patch;
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs::{metadata, remove_file, rename, File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::vec::Vec;
use walkdir::{DirEntry, WalkDir};

lazy_static! {
    static ref DECORATOR: Regex = Regex::new(r"^[@].+$").unwrap();
    static ref MLDECORATOR: Regex = Regex::new(r"^[@][a-zA-Z0-9]+\([^\)]*$").unwrap();
    static ref MLDECORATOREND: Regex = Regex::new(r"^.*\)$").unwrap();
    static ref VARIABLE: Regex =
        Regex::new(r"^(public|private|readonly)?[\s]*[a-zA-Z?_\-0-9]+[\s]*:[\s]*.+;$").unwrap();
    static ref CLASS: Regex =
        Regex::new(r"^(export)?[\s]*(class)[\s]+[a-zA-Z?_\-0-9\{\}\s;:]+$").unwrap();
    static ref DELETION: Regex = Regex::new(r"^(-)[^-]+$").unwrap();
}

fn path_exists(path: &str) -> bool {
    metadata(path).is_ok()
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    println!(
        "Options:     
            commenter <filename>  
            commenter <dirname>/  
            commenter <dirname>/ -ends <file extension>\n"
    );

    match args.len() {
        2 => {
            let arg = &args[1];

            if &arg[arg.len() - 1..] != "/" {
                transform_file(arg)?;
            } else {
                transform_dir(arg, ".ts")?;
            }
        }

        4 => {
            if &args[2] == "-ends" {
                let arg = &args[1];
                let ends = &args[3].to_owned();
                transform_dir(arg, ends)?;
            } else {
                return Err(Error::new(ErrorKind::Other, "unsupport cmd flags"));
            }
        }

        _ => {
            return Err(Error::new(ErrorKind::Other, "unsupport cmd args"));
        }
    }

    Ok(())
}

fn transform_dir(src: &String, ends_with: &str) -> Result<(), Error> {
    fn is_hidden(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
    }

    let files = WalkDir::new(src)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|file| file.ok());

    for file in files {
        if file.metadata().unwrap().is_file() {
            let src = file.path().to_str().unwrap().to_owned();

            if src.ends_with(ends_with) {
                transform_file(&src)?;
            }
        }
    }
    Ok(())
}

/**
 * Comment a file,
 * if success, a file with src name should exist with comments.
 * if fail, a file with src name should exist with no modification.
 *
 * No file other than src name should exist after this method terminates.
 *
 */
fn transform_file(src: &String) -> Result<(), Error> {
    let dst = format!("{}-new", src);
    if path_exists(&dst) {
        println!("dst file: {} names exists in current proj", &dst);
        return Err(Error::new(ErrorKind::Other, "repeatitive name"));
    }

    match addcomment(&src, &dst) {
        Ok(_) => {
            println!("SUCCESS to add comment to file: {} ", src);
            rename(&dst, &src)?;
        }
        Err(e) => {
            remove_file(&dst)?;
            return Err(e);
        }
    }

    Ok(())
}

/**
 * perform typedoc comment adding, take src file and write out to dst file.
 *
 * This method also create a patch file between src and dst to looks for any - (deletion)
 * to ensure no line of src is deleted.
 *
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

    //keep a String representation of src and dst
    let mut original: String = String::from("");
    let mut modified: String = String::from("");

    for line in buffered.lines() {
        let copy: String = line?.to_owned();
        let trim: String = String::from(copy.trim());

        original.push_str(&format!("{}\n", copy));

        //check if we are in comment or out of comment
        if trim == "//AUTOCOMMENT" {
            in_comment = true;
            continue;
        }
        if trim == "//ENDCOMMENT" {
            in_comment = false;
            continue;
        }

        //ignore white line...
        if trim == "\n" {
            continue;
        }

        //if we are in the comment area, ignore...
        if in_comment {
            continue;
        } else if ml {
            if MLDECORATOREND.is_match(&trim) {
                ml_decorator.push_str(&copy);
                let copy = ml_decorator.clone();
                decorators.push(copy);
                ml = false;
            } else {
                let f = format!("{}\n", &copy);
                ml_decorator.push_str(&f);
            }
        } else {
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
                out_string = String::from("//AUTOCOMMENT\n/**\n * Decorator Usage:\n * ```\n");
                for d in &decorators {
                    let d = format!(" * {} \n", d.trim());
                    out_string.push_str(&d);
                }
                out_string.push_str(" * ```\n */\n//ENDCOMMENT\n");

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
                //write!(output, "{}", &out_string)?;
                modified.push_str(&out_string);
            } else {
                out_string = format!("{}\n", copy);
                //write!(output, "{}", &out_string)?;
                modified.push_str(&out_string);
            }
        }
    }

    //Run a patch check ensure no src line is deleted.
    let patch_str = create_patch(&original, &modified).to_string();
    let patch: Vec<_> = (&patch_str).lines().collect();
    for line in patch {
        if DELETION.is_match(line) {
            return Err(Error::new(
                ErrorKind::Other,
                format!("attempt to delete line: {} in src file: {}", line, src),
            ));
        }
    }

    //write out if no error
    write!(output, "{}", &modified)?;
    Ok(())
}
