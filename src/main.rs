mod ast;
mod checker;
mod error;
mod file_provider;
mod operators;
mod parser;
mod scope;
mod span;
mod tokens;
mod utils;
mod color;

use file_provider::{FileProvider, FileReader};
use std::env::current_dir;
use utils::RecursiveFn;

use termcolor::{BufferWriter, ColorChoice};

fn main() {
    let fprov =
        file_provider::fs::FileProvider::new(&current_dir().expect("NO CWD").join("test_files"));
    let tokens = match tokens::tokenize("test.lang", &fprov) {
        Ok(t) => t,
        Err(e) => {
            e.default_display(&fprov, true);
            panic!("Error returned wasn't fatal")
        }
    };
    println!("{}", fprov.get_file(&"test.lang").contents());
    //println!("{:?}", tokens);

    //eprintln!("{}", tokens[0].error("").display(&fprov));

    let buffer_writer = BufferWriter::stdout(ColorChoice::Always);
    match parser::parse_lines(tokens) {
        Ok(items) => {
            if let Err(e) = checker::check(&items) {
                e.display(&fprov, &buffer_writer);
                return
            }
            for item in items {
                println!(" {}", item);
            }
        }
        Err(e) => e.display(&fprov, &buffer_writer),
    };

    //repl();
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
    loop {
        write!(stdout, ">> ");
        stdout.flush().expect("Error flushing");
        let mut line = String::new();
        stdin.read_line(&mut line).expect("Error reading line");
        file_provider.new_line(line);
        let tokens = tokens::tokenize("repl", &file_provider);
        if let Err(mut e) = match tokens {
            Ok(tokens) => {
                let ast = parser::parse_expr(&tokens);
                match ast {
                    Ok(ast) => {
                        println!("-> {}", ast);
                        println!("->? {:?}", ast);
                        Ok(())
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
