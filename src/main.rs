// RCL -- A sane configuration language.
// Copyright 2023 Ruud van Asseldonk

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

use rcl::runtime::Env;
use rcl::source::{DocId, Document, Inputs};

const USAGE: &str = r#"
RCL -- Ruud's Configuration Language.

Usage:
  rcl eval <file>
  rcl fmt <file>
  rcl highlight <file>
  rcl repl
  rcl -h | --help

Arguments:
  <file>        The input file to process, or '-' for stdin.

Options:
  -h --help     Show this screen.
"#;

fn main_eval(inputs: &Inputs) -> rcl::error::Result<()> {
    let debug = false;

    for (i, doc) in inputs.iter().enumerate() {
        let id = DocId(i as u32);
        let tokens = rcl::lexer::lex(id, doc.data)?;
        if debug {
            for (token, span) in &tokens {
                eprintln!("{span:?} {token:?}");
            }
        }

        let cst = rcl::parser::parse(id, doc.data)?;
        if debug {
            eprintln!("{cst:#?}");
        }

        let ast = rcl::abstraction::abstract_expr(doc.data, &cst);
        if debug {
            eprintln!("{ast:#?}");
        }

        let mut env = Env::new();
        let val = rcl::eval::eval(&mut env, &ast)?;

        let mut val_json = String::new();
        rcl::json::format_json(val.as_ref(), &mut val_json)?;
        println!("{}", val_json);
    }

    Ok(())
}

struct Data {
    path: String,
    data: String,
}

impl Data {
    fn load(fname: &str) -> Data {
        // TODO: Read from stdin if fname is '-'.
        let data = std::fs::read_to_string(fname).expect("Failed to load example.");
        Data {
            path: fname.to_string(),
            data: data,
        }
    }

    fn as_ref(&self) -> Document {
        Document {
            path: &self.path,
            data: &self.data,
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_refs: Vec<&str> = args.iter().map(|a| &a[..]).collect();
    match &args_refs[..] {
        ["-h"] | ["--help"] => {
            println!("{}", USAGE.trim());
            std::process::exit(0);
        }
        ["eval", fname] => {
            let data = Data::load(fname);
            let inputs = [data.as_ref()];
            if let Err(err) = main_eval(&inputs) {
                err.print(&inputs);
            }
        }
        ["fmt", fname] => {
            let data = Data::load(fname);
            let _inputs = [data.as_ref()];
            unimplemented!("TODO: Implement fmt.");
        }
        ["highlight", fname] => {
            let data = Data::load(fname);
            let _inputs = [data.as_ref()];
            unimplemented!("TODO: Implement highlight.");
        }
        ["repl"] => {
            unimplemented!("TODO: Implement repl.");
        }
        _ => {
            eprintln!("Failed to parse command line. Run with --help for usage.");
            std::process::exit(1);
        }
    }
}
