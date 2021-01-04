mod ast;
mod checker;
mod color;
mod error;
mod file_provider;
mod interpreter;
mod operators;
mod parser;
mod scope;
mod span;
mod tokens;
mod utils;

mod backend;
mod options;

// use file_provider::{FileProvider, FileReader};
use std::env::current_dir;
use utils::RecursiveFn;

use termcolor::{BufferWriter, ColorChoice};

use std::time::Instant;

use structopt::StructOpt;

fn main() {
    let opts = options::Options::from_args();
    println!("{:?}", opts);
    let fprov = file_provider::fs::FileProvider::new(&current_dir().expect("NO CWD"));
    let start_tok = Instant::now();
    let tokens = match tokens::tokenize(&opts.input_file, &fprov) {
        Ok(t) => t,
        Err(e) => {
            println!("Tokenization time: {}", start_tok.elapsed().as_secs_f32());
            e.default_display(&fprov, true);
            panic!("Error returned wasn't fatal")
        }
    };
    let tok_time = start_tok.elapsed();
    println!("Tokenization time: {}", tok_time.as_secs_f32());
    //println!("{}", fprov.get_file(&"test_files/test.lang").contents());
    let mut type_db = scope::TypeDB::new();
    //println!("{:?}", tokens);

    //eprintln!("{}", tokens[0].error("").display(&fprov));

    let buffer_writer = BufferWriter::stdout(ColorChoice::Always);
    println!("Parsing");
    let start_parse = Instant::now();
    match parser::parse_lines(tokens, false) {
        Ok(items) => {
            let parse_time = start_parse.elapsed();
            println!("Parse time: {}", parse_time.as_secs_f32());
            println!(
                "Parse and tokenize time: {}",
                (parse_time + tok_time).as_secs_f32()
            );
            println!("Checking");
            let start_check = Instant::now();
            if let Err(e) = checker::check(&items, &mut type_db, opts.lib) {
                let check_time = start_check.elapsed();
                println!("Check time: {}", check_time.as_secs_f32());
                println!("Tokenize, parse and check time: {}", (parse_time + tok_time + check_time).as_secs_f32());
                e.display(&fprov, &buffer_writer);
                return;
            }
            let check_time = start_check.elapsed();
            println!("Check time: {}", check_time.as_secs_f32());
            println!("Tokenize, parse and check time: {}", (parse_time + tok_time + check_time).as_secs_f32());
            
            println!("Codegen phase");
            let (codegen_opts, backend) = opts.into_codegen_options();
            backend.get_codegen().gen_code(&items, &mut type_db, codegen_opts);
        }
        Err(e) => {
            let parse_time = start_parse.elapsed();
            println!("Parse time: {}", parse_time.as_secs_f32());
            println!(
                "Parse and tokenize time: {}",
                (parse_time + tok_time).as_secs_f32()
            );
            e.display(&fprov, &buffer_writer)
        }
    };

    // repl();
    // let ast = parser::parse_expr(tokens);
    // println!("{}", ast);
    // let tokens = tokens::tokenize("1 + 2 * 3 * 4 + 1");
    // let ast = parser::parse_expr(tokens);
    // println!("{}", ast);
    //println!("{} {}", offset, tokens.len());
}

#[allow(unused)]
fn repl() {
    use std::io::Write;
    let mut file_provider = file_provider::repl::FileProvider::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut type_db = scope::TypeDB::new();
    let mut trait_db = checker::TraitDB::new();
    let mut checker_scope = match checker::load_scope(&mut type_db, &mut trait_db) {
        Ok(s) => s,
        Err(e) => panic!("{:?}", e),
    };
    let mut interpreter_scope = interpreter::load_scope();
    // let bool_type = type_db.get(&ast::TypeData::Bool);
    // println!("Bool is eq? {}", bool_type.get_impl_trait("Eq", &[&bool_type]).is_some());
    loop {
        write!(stdout, ">> ");
        stdout.flush().expect("Error flushing");
        let mut line = String::new();
        stdin.read_line(&mut line).expect("Error reading line");
        file_provider.new_line(format!("{}", line.trim_end()));
        let tokens = tokens::tokenize("repl", &file_provider);
        if let Err(mut e) = match tokens {
            Ok(tokens) => {
                let ast = parser::parse_expr(&tokens, false, 0);
                match ast {
                    Ok(ast) => {
                        if let Err(e) = checker::check_expr(&ast, &mut checker_scope, &mut type_db)
                        {
                            Err(e)
                        } else {
                            println!(
                                "{}",
                                match interpreter::run_expr(
                                    &mut interpreter_scope,
                                    &mut type_db,
                                    ast.as_ref()
                                ) {
                                    interpreter::RetVal::Value(v) => v,
                                    interpreter::RetVal::Return(v) => v,
                                }
                            );
                            Ok(())
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        } {
            e.default_display(&file_provider, false)
        }
    }
}
