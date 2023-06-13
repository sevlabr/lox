use bvm::vm::VM;
use bvm::{repl, run_file, Config};
use std::env;
use std::process;

fn main() {
    let mut config = Config::default();
    let vm = VM::default();

    let mut args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl(config, vm),
        2 => match args[1].as_str() {
            "-b" | "-d" | "-s" | "-t" => {
                set_config(&mut config, args[1].as_str());
                repl(config, vm);
            }
            "-h" => println!("{}", HELP_MESSAGE),
            _ => run_file(config, vm, args.swap_remove(1)),
        },
        3 => match args[1].as_str() {
            "-b" | "-d" | "-s" | "-t" => {
                set_config(&mut config, args[1].as_str());
                run_file(config, vm, args.swap_remove(2));
            }
            _ => {
                eprintln!("Wrong option: '{}'.", args[1].as_str());
                eprintln!("{}", ERROR_MESSAGE);
                process::exit(64);
            }
        },
        _ => {
            eprintln!("{}", ERROR_MESSAGE);
            process::exit(64);
        }
    }
}

fn set_config(config: &mut Config, option: &str) {
    match option {
        "-b" => config.bytecode = true,
        "-d" => config.debug = true,
        "-s" => config.scanner = true,
        "-t" => config.trace = true,
        _ => unreachable!("Expected one of: -b -d -s -t"),
    }
}

const ERROR_MESSAGE: &str = "Usage: lox [OPTIONS] [PATH]

Use 'lox -h' for more information";

const HELP_MESSAGE: &str = "Usage: lox [OPTIONS] [PATH]

Options:
  -b  Print generated top-level bytecode without program execution
  -d  Debug mode: execute normally, in case of error print bytecode
  -s  Print tokens generated by lexer (scanner) without program execution
  -t  Tracing mode (online debugging): execute program and additionally
      print each bytecode instruction and virtual machine stack state
  -h  Print help information

There are 2 modes of execution available: interactive prompt and source file program.
The first one is activated when [PATH] to a source file isn't provided.
All options except [-h] can be used in both modes. Simultaneous usage of more than
one option at a time is not supported yet.

Examples:
  lox                  Interactive prompt
  lox path/to/file     Source file execution
  lox -h               Help information
  lox -d               Interactive prompt in debug mode
  lox -b               Print generated top-level bytecode for each interactively written line of code
                       (it won't be executed by virtual machine)
  lox -t path/to/file  Source file execution in tracing mode
  lox -h path/to/file  Error, can't print help information and then execute code in normal mode";
