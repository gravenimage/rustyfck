extern crate rustc_serialize;
extern crate docopt;

use std::fs::File;
use std::path::Path;
use std::io::Result;
use std::io::Read;
use std::char;
use std::str;
use std::collections::HashMap;

use docopt::Docopt;

static USAGE: &'static str = "
Usage: rustyfck [-b] <source>

Options:
    -b, --bob   
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_source: String,
    flag_bob: bool,
}

fn source(filename: &str) -> String {
    let mut text = String::new();
    let mut f = File::open(&Path::new(filename)).ok().expect("Cannot find input file");
    f.read_to_string(&mut text).ok().expect("Cannot read contents");
   
    text
}
#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
enum Op {
    INC_DP,
    DEC_DP,
    INC_DEREF_DP,
    DEC_DEREF_DP,
    OUT_DEREF_DP,
    IN_DEREF_DP,
    LOOP_BEGIN,
    LOOP_END
}

fn decode(instructions: &str) -> Vec<Op> {
    let mut decoded = Vec::new();
    let mut lookup = HashMap::new();
    lookup.insert('>', Op::INC_DP);
    lookup.insert('<', Op::DEC_DP);
    lookup.insert('+', Op::INC_DEREF_DP);
    lookup.insert('-', Op::DEC_DEREF_DP);
    lookup.insert('.', Op::OUT_DEREF_DP);
    lookup.insert(',', Op::IN_DEREF_DP);
    lookup.insert('[', Op::LOOP_BEGIN);
    lookup.insert(']', Op::LOOP_END);
    for c in instructions.chars() {
        if let Some(op) = lookup.get(&c) {
            decoded.push(op.clone());
        }
    }

    decoded
}

fn interpret( instructions : &str,  mem: &mut Vec<u8>) {
    let mut dp: usize = 0;
    let mut ip: usize = 0;
    let ops = decode(instructions);
    println!("ops.len: {}", ops.len());
    while ip < ops.len() {
        //println!("ip {} dp {} op {:?}", ip, dp, ops[ip]);
        match ops[ip] {
            Op::INC_DP => { dp += 1; ip += 1; } ,
            Op::DEC_DP => { dp -= 1; ip += 1; } ,
            Op::INC_DEREF_DP => { mem[dp] = mem[dp] + 1; ip += 1; }
            Op::DEC_DEREF_DP => { mem[dp] = mem[dp] - 1; ip += 1; }
            Op::OUT_DEREF_DP => { 
                print!("{}", mem[dp] as char);
                ip += 1;
            },
            Op::IN_DEREF_DP => { ip += 1 }, // no-op for now
            Op::LOOP_BEGIN => { if mem[dp] != 0 {
                                    ip += 1
                                } 
                                else {
                                    let mut level = 1;
                                    while level > 0 {
                                        ip +=1 ;
                                        level = match ops[ip] {
                                            Op::LOOP_BEGIN => level + 1,
                                            Op::LOOP_END => level - 1,
                                            _ => level
                                        }
                                    }
                                    ip +=1 ;    
                                }
            },
            Op::LOOP_END =>  { if mem[dp] == 0 {
                                    ip += 1
                                } 
                                else {
                                    let mut level = 1;
                                    while level > 0 {
                                        ip -=1 ;
                                        level = match ops[ip] {
                                            Op::LOOP_END => level + 1,
                                            Op::LOOP_BEGIN => level - 1,
                                            _ => level
                                        }
                                    }
                                    ip +=1 ;    
                                }
            }
        }
    }    
}


fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
   
    let code = source(&args.arg_source);

    let mut memory = Vec::with_capacity(10000);
    for _ in 0..10000 {
        memory.push(0);
    }

    interpret(&code, &mut memory);
}
