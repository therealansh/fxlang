use std::{
    env, fs, io::{self, Write}, process,
};
use fxlang::frontend::lexer::Lexer;
use colored::*;
use fxlang::frontend::parser::Parser;
use fxlang::frontend::expr::AstPrinter;
use fxlang::frontend::interpreter::Interpreter;
use fxlang::frontend::error::Error;
use std::io::stdin;
use std::process::exit;
use fxlang::frontend::resolver::Resolver;

struct FxLang{
    interpreter: Interpreter,
}

impl FxLang {
    fn new() -> Self {
        FxLang {
            interpreter:Interpreter::new()
        }
    }
//TODO look into return error
    fn run_file(&mut self, path: &str) {
        //Read the file .fx
        let input = fs::read_to_string(path);
        /*
            Result<T,E> -> https://doc.rust-lang.org/std/result/
            match ~ switch in C
        */
        match input {
            Ok(bytes) => self.run(bytes),
            Err(e) => {
                eprintln!("Failed to read file {:?}", e);
                process::exit(74);
            }
        }
    }

    fn run_repl(&mut self) {
        println!("Welcome to {} {}\n", "f(x)".green().italic(), "REPL".bold());
        loop {
            print!("> ");
            io::stdout().flush().expect("Failed to flush stdout!");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read from stdin!");

            self.run(input);
        }
    }

    fn run(&mut self,src: String) {
        /*
            &str is fixed length and String is growable
        */
        let mut lexer = Lexer::new(src);
        let tokens = lexer.scan_tokens();
        let mut parser = Parser::new(tokens.to_vec());
        let statements = match parser.parse() {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Parsing Error: {:?}",e);
                process::exit(74)
            }
        };
        let mut resolver  = Resolver::new(&mut self.interpreter);
        resolver.resolve_stmts(&statements);
        if resolver.had_error{
            return ()
        }

        match self.interpreter.interpret(&statements){
            Ok(_)=> (),
            Err(e) => eprintln!("{}",e)
        }
        // let mut printer = AstPrinter;
        // match printer.print(expr) {
        //     Ok(res) => println!("{}", res),
        //     Err(e) => eprintln!("Error Printing AST: {:?}", e)
        // }
    }
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut fxlang = FxLang::new();
    match args.as_slice() {
        [_, file] => fxlang.run_file(file),
        [_] => fxlang.run_repl(),
        _ => {
            eprintln!("Usage: fxlang [script]");
            exit(64)
        }
    }
    Ok(())
}

