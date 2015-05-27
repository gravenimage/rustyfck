use std::fs::File;
use std::path::Path;
use std::io::Result;
use std::io::Read;
use std::char;
use std::str;


fn source() -> String {
    let mut text = String::new();
    let mut f = File::open(&Path::new("mandelbrot.bf")).ok().expect("Cannot find prog.bf");
    f.read_to_string(&mut text).ok().expect("Cannot read contents");
   
    text
}
#[derive(Debug)]
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
    for c in instructions.chars() {
        if (c == '>') {
            decoded.push(Op::INC_DP);
        }
        if (c == '<') {
            decoded.push(Op::DEC_DP);
        }
        if (c == '+') {
            decoded.push(Op::INC_DEREF_DP);
        }
        if (c == '-') {
            decoded.push(Op::DEC_DEREF_DP);
        }
        if (c == '.') {
            decoded.push(Op::OUT_DEREF_DP);
        }
        if (c == ',') {
            decoded.push(Op::IN_DEREF_DP);
        }
        if (c == '[') {
            decoded.push(Op::LOOP_BEGIN);
        }
        if ( c== ']') {
            decoded.push(Op::LOOP_END);
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
        match(ops[ip]) {
            Op::INC_DP => { dp += 1; ip += 1; } ,
            Op::DEC_DP => { dp -= 1; ip += 1; } ,
            Op::INC_DEREF_DP => { mem[dp] = mem[dp] + 1; ip += 1; }
            Op::DEC_DEREF_DP => { mem[dp] = mem[dp] - 1; ip += 1; }
            Op::OUT_DEREF_DP => { 
                print!("{}", mem[dp] as char);
                ip += 1;
            },
            Op::IN_DEREF_DP => { ip += 1 }, // no-op for now
            Op::LOOP_BEGIN => { if (mem[dp] != 0) {
                                    ip += 1
                                } 
                                else {
                                    let mut level = 1;
                                    while (level > 0) {
                                        ip +=1 ;
                                        level = match(ops[ip]) {
                                            Op::LOOP_BEGIN => level + 1,
                                            Op::LOOP_END => level - 1,
                                            _ => level
                                        }
                                    }
                                    ip +=1 ;    
                                }
            },
            Op::LOOP_END =>  { if (mem[dp] == 0) {
                                    ip += 1
                                } 
                                else {
                                    let mut level = 1;
                                    while  (level > 0) {
                                        ip -=1 ;
                                        level = match(ops[ip]) {
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
    println!("Hello, world!");
    println!("{}", source());
    let mut memory = Vec::with_capacity(10000);
    for _ in 0..10000 {
        memory.push(0);
    }
    interpret(&source(), &mut memory);
}
