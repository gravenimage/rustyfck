extern crate rustc_serialize;
extern crate docopt;

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::collections::HashMap;
use std::option::Option::{Some, None};
use std::fmt::Display;

use docopt::Docopt;

static USAGE: &'static str = "
Usage: rustyfck [-d -b -z -r] <source>

Options:
    -b, --brackets   
    -z, --zero
    -r, --rle
    -d, --debug
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_source: String,
    flag_brackets: bool,
    flag_zero: bool,
    flag_debug: bool,
    flag_rle: bool
}

fn source(filename: &str) -> String {
    let mut text = String::new();
    let mut f = File::open(&Path::new(filename)).ok().expect("Cannot find input file");
    f.read_to_string(&mut text).ok().expect("Cannot read contents");
   
    text
}


#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum DebuggingLevel {
    Silent,
    Verbose
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
enum Op {
    IncDp(usize),
    DecDp(usize),
    IncDerefDp(usize),
    DecDerefDp(usize),
    OutDerefDp,
    InDerefDp,
    LoopBegin(usize),
    LoopEnd(usize),
    Zero
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Op::IncDp(i) => write!(f, ">_{}", i),
            Op::DecDp(i) =>  write!(f, "<_{}", i),
            Op::IncDerefDp(i) => write!(f,"+_{}", i),
            Op::DecDerefDp(i) => write!(f, "-_{}", i),
            Op::OutDerefDp => write!(f, "."),
            Op::InDerefDp => write!(f, ","),
            Op::LoopBegin(i) => write!(f, "[_{}", i),

            Op::LoopEnd(i) => write!(f, "]_{}", i),
            Op::Zero => write!(f, "Z"),
        }
    }
}

fn decode(instructions: &str) -> Vec<Op> {
    let mut decoded = Vec::new();
    let mut lookup = HashMap::new();
    lookup.insert('>', Op::IncDp(1));
    lookup.insert('<', Op::DecDp(1));
    lookup.insert('+', Op::IncDerefDp(1));
    lookup.insert('-', Op::DecDerefDp(1));
    lookup.insert('.', Op::OutDerefDp);
    lookup.insert(',', Op::InDerefDp);
    lookup.insert('[', Op::LoopBegin(0));
    lookup.insert(']', Op::LoopEnd(0));
    for c in instructions.chars() {
        if let Some(op) = lookup.get(&c) {
            decoded.push(op.clone());
        }
    }

    decoded
}

fn run_length(op: Op, ops: &Vec<Op>, idx: usize) -> usize {
    let mut i = idx;
    let mut count = 0;
    while i < ops.len() {
        if ops[i] != op {
            break;
        }
        i += 1;
        count += 1;
    }
    count 
}

fn rle(ops: &Vec<Op>) -> Vec<Op> {
    let mut new_ops = Vec::with_capacity(ops.len());
    let mut ip: usize = 0;
    while ip < ops.len() {
        match ops[ip] {
            Op::IncDp(1) => {
                let count = run_length(Op::IncDp(1), ops, ip);
                new_ops.push(Op::IncDp(count));
                ip += count;
            }
            Op::DecDp(1) => {
                let count = run_length(Op::DecDp(1), ops, ip);
                new_ops.push(Op::DecDp(count));
                ip += count;
            }
            Op::IncDerefDp(1) => {
                let count = run_length(Op::IncDerefDp(1), ops, ip);
                new_ops.push(Op::IncDerefDp(count));
                ip += count;
            }
            Op::DecDerefDp(1) => {
                let count = run_length(Op::DecDerefDp(1), ops, ip);
                new_ops.push(Op::DecDerefDp(count));
                ip += count;
            }
            op @ _ => { new_ops.push(op); ip += 1;}
        }
    }
    new_ops
}


fn elide_zeroing_loop(ops: &Vec<Op>) -> Vec<Op> {
    let mut new_ops = Vec::with_capacity(ops.len());
    let mut ip: usize = 0;

    while ip < ops.len() {
        match ops[ip] {
	    Op::LoopBegin(0) => {
                let mut elided = false;
                match ops[ip+1] {
                    Op::DecDerefDp(1) => {
                        match ops[ip+2] {
                            Op::LoopEnd(..) => {
                                new_ops.push(Op::Zero);
                                elided = true;
                                ip += 2;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                if !elided {
                    new_ops.push(Op::LoopBegin(0));
                }
            },
            op @ Op::IncDp(_) | 
            op @ Op::DecDp(_) | 
            op @ Op::IncDerefDp(_) |
            op @ Op::DecDerefDp(_) |
            op @ Op::OutDerefDp | 
            op @ Op::InDerefDp |
            op @ Op::LoopEnd(_) => {
                new_ops.push(op)
            },
            
            _ => { panic!("Unexpected op") }
        }
        ip += 1;
    }

    new_ops
}

fn match_brackets(ops: &Vec<Op>) -> Vec<Op> {
    let mut new_ops = Vec::with_capacity(ops.len());
    let mut ip: usize = 0;
    
    while ip < ops.len() {
        match ops[ip] {
            op @ Op::IncDp(_) | 
            op @ Op::DecDp(_) | 
            op @ Op::IncDerefDp(_) |
            op @ Op::DecDerefDp(_) |
            op @ Op::OutDerefDp | 
            op @ Op::InDerefDp |
            op @ Op::Zero => {
                new_ops.push(op)
            },
            Op::LoopBegin(..) => {
                let mut blevel = 1;
                let mut end_ip = ip;
                while blevel > 0 {
                    end_ip += 1;
                    match ops[end_ip] {
                        Op::LoopBegin(..) => { blevel +=1; },
                        Op::LoopEnd(..) => { blevel -= 1; },
                        _ => {}
                    }
                }
                new_ops.push(Op::LoopBegin(end_ip));
            },
            Op::LoopEnd(..) => {
                let mut blevel = 1;
                let mut begin_ip = ip;
                while blevel > 0 {
                    begin_ip -= 1;
                    match ops[begin_ip] {
                        Op::LoopEnd(..) => { blevel += 1; },
                        Op::LoopBegin(..) => { blevel -= 1; },
                        _ => {}
                    }
                }
                new_ops.push(Op::LoopEnd(begin_ip));
            },
        }
        ip += 1;
    }
    
    new_ops
}


fn interpret( ops: &Vec<Op>,  mem: &mut Vec<u8>, debug: DebuggingLevel) {
    let mut dp: usize = 0;
    let mut ip: usize = 0;
    
    if debug > DebuggingLevel::Silent {
        println!("ops.len: {}", ops.len());
    }
    while ip < ops.len() {
        if debug > DebuggingLevel::Silent {
            println!("dp {} mem[dp] {} op {}", dp, mem[dp], ops[ip]);
        }
        match ops[ip] {
            Op::IncDp(count) => {
                dp += count;
                ip += 1;
            }
            Op::DecDp(count) => {
                dp -= count;
                ip += 1;
            }
            Op::IncDerefDp(count) => { 
                mem[dp] = mem[dp].wrapping_add(count as u8); 
                ip += 1; 
            }
            Op::DecDerefDp(count) => { 
                mem[dp] = mem[dp].wrapping_sub(count as u8); 
                ip += 1; 
            }
            Op::OutDerefDp => { 
                print!("{}", mem[dp] as char);
                ip += 1;
            },
            Op::InDerefDp => { ip += 1 }, // no-op for now
	    
            Op::LoopBegin(0) => { 
                if mem[dp] != 0 {
                    ip += 1
                } 
                else {
                    let mut level = 1;
                    while level > 0 {
                        ip +=1 ;
                        level = match ops[ip] {
                            Op::LoopBegin(_) => level + 1,
                            Op::LoopEnd(_) => level - 1,
                            _ => level
                        }
                    }
                    ip +=1 ;    
                }
            },
	    
            Op::LoopBegin(matching) => { 
                if mem[dp] != 0 {
                    ip += 1;
                }
                else {
                    ip = matching + 1; 
                }
            }
            Op::LoopEnd(0) =>  { 
                if mem[dp] == 0 {
                    ip += 1
                } 
                else {
                    let mut level = 1;
                    while level > 0 {
                        ip -=1 ;
                        level = match ops[ip] {
                            Op::LoopEnd(_) => level + 1,
                            Op::LoopBegin(_) => level - 1,
                            _ => level
                        }
                    }
                    ip +=1 ;    
                }
            }
            Op::LoopEnd(matching) => {
                 if mem[dp] == 0 {
                    ip += 1
                } 
                else {
                    ip = matching + 1;
                }
            }
            Op::Zero => {
                mem[dp] = 0;
                ip += 1;
            }
        }
    }    
}


fn dump_instructions(ops: &Vec<Op>) {
    let mut indent = 0;
    for op in ops.iter() {
        match *op {
            Op::LoopBegin(_) => { 
                let indent_str = std::iter::repeat(" ").take(indent).collect::<String>();
                print!("\n{}", indent_str); 
                indent += 1;
            },
            Op::LoopEnd(_) => {
                indent -= 1;
            }
            _ => {}
        }
        
        print!("{}", op);
    }
    println!("");
}

fn main() {
    let mem_size = 32000;
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
   
    let code = source(&args.arg_source);

    let mut memory = Vec::with_capacity(mem_size);
    for _ in 0..mem_size {
        memory.push(0u8);
    }

    let mut ops = decode(&code);

    if args.flag_zero {
        ops = elide_zeroing_loop(&ops);
    }
    
    if args.flag_rle {
        ops = rle(&ops);
    }

    // matching brackets has to be last as it encodes absolute positions
    if args.flag_brackets {
        ops = match_brackets(&ops);
    }

    if args.flag_debug {
        println!("Optimized:");
        dump_instructions(&ops);
    }

    interpret(&ops, &mut memory,  
              if args.flag_debug  {
                  DebuggingLevel::Verbose 
              } 
              else { 
                  DebuggingLevel::Silent
              });
}

#[test]
fn always_passes() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn always_fails() {
    //assert!(false);
}

#[test]
fn test_rle_1() {
   let optimized = rle(&vec![Op::IncDp(1), Op::IncDp(1)]);
   assert_eq!(optimized, [Op::IncDp(2)]);
}

#[test]
fn test_rle_2() {
   let optimized = rle(&vec![Op::IncDp(1)]);
   assert_eq!(optimized, [Op::IncDp(1)]);
}

