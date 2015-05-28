extern crate rustc_serialize;
extern crate docopt;

use std::fs::File;
use std::path::Path;
use std::io::Result;
use std::io::Read;
use std::char;
use std::str;
use std::collections::HashMap;
use std::option::Option::{Some, None};

use docopt::Docopt;

static USAGE: &'static str = "
Usage: rustyfck [-b -z] <source>

Options:
    -b, --brackets   
    -z, --zero
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_source: String,
    flag_brackets: bool,
    flag_zero: bool
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
    LOOP_BEGIN(Option<usize>),
    LOOP_END(Option<usize>),
    ZERO
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
    lookup.insert('[', Op::LOOP_BEGIN(None));
    lookup.insert(']', Op::LOOP_END(None));
    for c in instructions.chars() {
        if let Some(op) = lookup.get(&c) {
            decoded.push(op.clone());
        }
    }

    decoded
}

fn elide_zeroing_loop(ops: &Vec<Op>) -> Vec<Op> {
    let mut new_ops = Vec::with_capacity(ops.len());
    let mut ip: usize = 0;
    let mut count = 0;
    while ip < ops.len() {
        match ops[ip] {
            op @ Op::INC_DP | 
            op @ Op::DEC_DP | 
            op @ Op::INC_DEREF_DP |
            op @ Op::DEC_DEREF_DP |
            op @ Op::OUT_DEREF_DP | 
            op @ Op::IN_DEREF_DP |
            op @ Op::LOOP_END(..) => {
                new_ops.push(op)
            },
            Op::LOOP_BEGIN(..) => {
                let mut elided = false;
                match ops[ip+1] {
                    Op::DEC_DEREF_DP => {
                        match ops[ip+2] {
                            Op::LOOP_END(..) => {
                                new_ops.push(Op::ZERO);
                                elided = true;
                                count +=1 ;
                                ip += 2;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                if !elided {
                    new_ops.push(Op::LOOP_BEGIN(None));
                }
            },
            _ => { panic!("Unexpected op") }
        }
        ip += 1;
    }
    
    println!("Elided {} zero-loops", count);

    new_ops

}

fn match_brackets(ops: &Vec<Op>) -> Vec<Op> {
    let mut new_ops = Vec::with_capacity(ops.len());
    let mut ip: usize = 0;
    
    while ip < ops.len() {
        match ops[ip] {
            op @ Op::INC_DP | 
            op @ Op::DEC_DP | 
            op @ Op::INC_DEREF_DP |
            op @ Op::DEC_DEREF_DP |
            op @ Op::OUT_DEREF_DP | 
            op @ Op::IN_DEREF_DP => {
                new_ops.push(op)
            },
            Op::LOOP_BEGIN(..) => {
                let mut blevel = 1;
                let mut end_ip = ip;
                while blevel > 0 {
                    end_ip += 1;
                    match ops[end_ip] {
                        Op::LOOP_BEGIN(..) => { blevel +=1; },
                        Op::LOOP_END(..) => { blevel -= 1; },
                        _ => {}
                    }
                }
                new_ops.push(Op::LOOP_BEGIN(Some(end_ip)));
            },
            Op::LOOP_END(..) => {
                let mut blevel = 1;
                let mut begin_ip = ip;
                while blevel > 0 {
                    begin_ip -= 1;
                    match ops[begin_ip] {
                        Op::LOOP_END(..) => { blevel += 1; },
                        Op::LOOP_BEGIN(..) => { blevel -= 1; },
                        _ => {}
                    }
                }
                new_ops.push(Op::LOOP_END(Some(begin_ip)));
            },
            _ => {}
        }
        ip += 1;
    }
    
    new_ops
}

fn interpret( ops: &Vec<Op>,  mem: &mut Vec<u8>) {
    let mut dp: usize = 0;
    let mut ip: usize = 0;
    println!("ip {} dp {} mem[dp] {} op {:?}", ip, dp, mem[dp], ops[ip]);

    println!("ops.len: {}", ops.len());
    while ip < ops.len() {
        println!("ip {} dp {} mem[dp] {} op {:?}", ip, dp, mem[dp], ops[ip]);
        match ops[ip] {
            Op::INC_DP => { 
                dp += 1; 
                ip += 1; 
            } ,
            Op::DEC_DP => { 
                dp -= 1; 
                ip += 1; 
            } ,
            Op::INC_DEREF_DP => { 
                mem[dp] = mem[dp].wrapping_add(1); 
                ip += 1; 
            }
            Op::DEC_DEREF_DP => { 
                mem[dp] = mem[dp].wrapping_sub(1); 
                ip += 1; 
            }
            Op::OUT_DEREF_DP => { 
                print!("{}", mem[dp] as char);
                ip += 1;
            },
            Op::IN_DEREF_DP => { ip += 1 }, // no-op for now
            Op::LOOP_BEGIN(None) => { 
                if mem[dp] != 0 {
                    ip += 1
                } 
                else {
                    let mut level = 1;
                    while level > 0 {
                        ip +=1 ;
                        level = match ops[ip] {
                            Op::LOOP_BEGIN(_) => level + 1,
                            Op::LOOP_END(_) => level - 1,
                            _ => level
                        }
                    }
                    ip +=1 ;    
                }
            },
            Op::LOOP_BEGIN(Some(matching)) => { 
                if mem[dp] != 0 {
                    ip += 1;
                }
                else {
                    ip = matching + 1; 
                }
            }
            Op::LOOP_END(None) =>  { 
                if mem[dp] == 0 {
                    ip += 1
                } 
                else {
                    let mut level = 1;
                    while level > 0 {
                        ip -=1 ;
                        level = match ops[ip] {
                            Op::LOOP_END(_) => level + 1,
                            Op::LOOP_BEGIN(_) => level - 1,
                            _ => level
                        }
                    }
                    ip +=1 ;    
                }
            }
            Op::LOOP_END(Some(matching)) => {
                 if mem[dp] == 0 {
                    ip += 1
                } 
                else {
                    ip = matching + 1;
                }
            }
            Op::ZERO => {
                mem[dp] = 0;
                ip += 1;
            }
        }
    }    
}


fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
   
    let code = source(&args.arg_source);

    let mut memory = Vec::with_capacity(32000);
    for _ in 0..32000 {
        memory.push(0u8);
    }

    let mut ops = decode(&code);
    if args.flag_zero {
        ops = elide_zeroing_loop(&ops);
    }
    // optimize for matching brackets
    if args.flag_brackets {
        ops = match_brackets(&ops);
    }

    interpret(&ops, &mut memory);
}
