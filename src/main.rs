
mod types;

mod parser;
mod checker;
mod builder;

#[macro_use] extern crate bitflags;
#[path = "llvm_sys/src/lib.rs"]
#[allow(dead_code)]
mod llvm_sys;

extern crate getopts;
use getopts::Options;
use std::env;

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::process::Command;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Useage: {} [options]", program);
	print!("{}", opts.usage(&brief));
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	// Get input and output files from command line.
	let mut opts = Options::new();
	opts.optopt("o", "", "set output file name", "NAME");
	opts.optflag("h", "help", "print this help menu");
	let matches = opts.parse(&args[1..]).unwrap();
	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	}
	let output = matches.opt_str("o");
	let input = if !matches.free.is_empty() {
		matches.free[0].clone()
	} else {
		print_usage(&program, opts);
		return;
	};
	let output: String = match output {
		Some(output) => output,
		None => {
			let input_path = PathBuf::from(input.clone());
			let mut output_path = PathBuf::new();
			output_path.push(input_path.parent().unwrap());
			output_path.push(input_path.file_stem().unwrap());
			output_path.to_str().unwrap().to_string()
		},
	};

	// Load input file.
	let mut file = File::open(&input).unwrap_or_else(|e| {
		panic!("Failed to open file '{}': {}", input, e);
	});
	let mut code = String::new();
	file.read_to_string(&mut code).unwrap_or_else(|_| {
		panic!("Given file '{}' is not proper unicode.", input);
	});

	// Compile program to llvm.
	let (mut block, e) = parser::parse(&code);
	if !e.is_empty() {
		for error in e {
			println!("{}", error);
		}
		panic!();
	}
	let mut scope = checker::Scope::new_root();
	match checker::passes(&mut block, &mut scope) {
		Ok(_)  => (),
		Err(e) => panic!("Err: {}", e),
	}
	builder::build(&block, &mut scope, "temp.ll");

	// Compile llvm to executable.
	let res = Command::new("llc").arg("temp.ll").output().unwrap_or_else(|e| {
		panic!("Failed to run llc: {}", e);
	});
	print!("{}", String::from_utf8(res.stdout).unwrap());
	print!("{}", String::from_utf8(res.stderr).unwrap());
	let res = Command::new("clang").arg("-o").
	                                arg(output).
	                                arg("temp.s").
	                                arg("shim.a").output().unwrap_or_else(|e| {
		panic!("Failed to run clang: {}", e);
	});
	print!("{}", String::from_utf8(res.stdout).unwrap());
	print!("{}", String::from_utf8(res.stderr).unwrap());
}

