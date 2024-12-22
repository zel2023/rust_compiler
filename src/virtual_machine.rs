use std::fs::File;
use std::io::{self, Read};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opt {
    LOAD,
    LOADI,
    STO,
    STI,
    ADD,
    SUB,
    MULT,
    DIV,
    BR,
    BRF,
    EQ,
    NOTEQ,
    GT,
    LES,
    GE,
    LE,
    AND,
    OR,
    NOT,
    IN,
    OUT,
    RETURN,
    ENTER,
    CAL,
    PAS,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Code {
    opt: [u8; 10], // 操作码
    operand: i32,  // 操作数
}

// 读取二进制文件
pub fn read_codes(file_path: &str) -> io::Result<Vec<Code>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // 确保文件大小是 Code 结构体的整数倍
    assert_eq!(buffer.len() % std::mem::size_of::<Code>(), 0);

    let num_codes = buffer.len() / std::mem::size_of::<Code>();
    let mut codes = Vec::with_capacity(num_codes);

    for i in 0..num_codes {
        let offset = i * std::mem::size_of::<Code>();
        let code: Code = unsafe {
            std::ptr::read(buffer[offset..].as_ptr() as *const Code)
        };
        codes.push(code);
    }

    Ok(codes)
}

// 字节转字符串
fn byte_array_to_opt_str(opt: &[u8; 10]) -> String {
    opt.iter()
        .take_while(|&&byte| byte != 0) // 忽略零字节
        .map(|&byte| byte as char)
        .collect()
}

// 打印读取文件的内容
pub fn display_codes(codes: &[Code]) {
    for (i, code) in codes.iter().enumerate() {
        let opt_str = byte_array_to_opt_str(&code.opt);
        println!("Code {}: opt = {}, operand = {}", i, opt_str, code.operand);
    }
}

// 初始化操作码映射
pub fn init_map() -> HashMap<String, Opt> {
    let mut map = HashMap::new();
    map.insert("LOAD".to_string(), Opt::LOAD);
    map.insert("LOADI".to_string(), Opt::LOADI);
    map.insert("STO".to_string(), Opt::STO);
    map.insert("STI".to_string(), Opt::STI);
    map.insert("ADD".to_string(), Opt::ADD);
    map.insert("SUB".to_string(), Opt::SUB);
    map.insert("MULT".to_string(), Opt::MULT);
    map.insert("DIV".to_string(), Opt::DIV);
    map.insert("BR".to_string(), Opt::BR);
    map.insert("BRF".to_string(), Opt::BRF);
    map.insert("EQ".to_string(), Opt::EQ);
    map.insert("NOTEQ".to_string(), Opt::NOTEQ);
    map.insert("GT".to_string(), Opt::GT);
    map.insert("LES".to_string(), Opt::LES);
    map.insert("GE".to_string(), Opt::GE);
    map.insert("LE".to_string(), Opt::LE);
    map.insert("AND".to_string(), Opt::AND);
    map.insert("OR".to_string(), Opt::OR);
    map.insert("NOT".to_string(), Opt::NOT);
    map.insert("IN".to_string(), Opt::IN);
    map.insert("OUT".to_string(), Opt::OUT);
    map.insert("RETURN".to_string(), Opt::RETURN);
    map.insert("ENTER".to_string(), Opt::ENTER);
    map.insert("CAL".to_string(), Opt::CAL);
    map.insert("PAS".to_string(), Opt::PAS);
    map
}


fn show_stack_info(stack: &Vec<i32>, top: usize, base: usize) {
    let mut temp = 0;
    println!("\t************");

    while temp <= top && stack[temp] >= 0 {
        if top == base {
            if top == temp {
                println!("\t*    {}    *    <----top(base)", stack[temp]);
            } else {
                println!("\t*    {}    *", stack[temp]);
            }
        } else {
            if top == temp {
                println!("\t*    {}    *    <----top", stack[temp]);
            }
            if base == temp {
                println!("\t*    {}    *    <----base", stack[temp]);
            }
            if top != temp && base != temp {
                println!("\t*    {}    *", stack[temp]);
            }
        }
        temp += 1;
    }

    if top == temp {
        println!("\t*          *    <----top\n");
    } else {
        println!("\t*         *\n");
    }

    println!("\t------------\n");
}




// 模拟抽象机运行
pub fn test_machine(codes: &[Code], map: &HashMap<String, Opt>) {
    let mut stack = vec![0; 100];
    let mut top = 0;
    let mut base = 0;
    let mut ip = 0;
    let mut step = 0;
    let mut outflag: bool = false;
    // println!("{}", codes.len());


    loop{
        let instruction = &codes[ip];
        ip += 1;
        outflag = false;
        let opt_str = byte_array_to_opt_str(&instruction.opt);
        if let Some(&operation) = map.get(&opt_str) {
            match operation {
                Opt::LOAD => {
                    stack[top] = stack[base + instruction.operand as usize];
                    top += 1;
                    outflag = true;
                }
                Opt::LOADI => {
                    stack[top] = instruction.operand;
                    top += 1;
                    outflag = true;
                }
                Opt::STO => {
                    top -= 1;
                    stack[base + instruction.operand as usize] = stack[top];
                    outflag = true;
                }
                Opt::ADD => {
                    top -= 1;
                    stack[top - 1] += stack[top];
                }
                Opt::SUB => {
                    top -= 1;
                    stack[top - 1] -= stack[top];
                }
                Opt::MULT => {
                    top -= 1;
                    stack[top - 1] *= stack[top];
                }
                Opt::DIV => {
                    top -= 1;
                    stack[top - 1] /= stack[top];
                }
                Opt::OUT => {
                    top -= 1;
                    println!("程序输出: {}", stack[top]);
                }
                Opt::BR => {
                    ip = instruction.operand as usize; // 无条件跳转
                    outflag = true;
                }
                Opt::BRF => {
                    top -= 1;
                    if stack[top] == 0 {
                        ip = instruction.operand as usize; // 条件跳转
                    }
                    outflag = true;
                }
                Opt::EQ => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] == stack[top] { 1 } else { 0 };
                }
                Opt::NOTEQ => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] != stack[top] { 1 } else { 0 };
                }
                Opt::GT => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] > stack[top] { 1 } else { 0 };
                }
                Opt::LES => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] < stack[top] { 1 } else { 0 };
                }
                Opt::GE => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] >= stack[top] { 1 } else { 0 };
                }
                Opt::LE => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] <= stack[top] { 1 } else { 0 };
                }
                Opt::AND => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] != 0 && stack[top] != 0 { 1 } else { 0 };
                }
                Opt::OR => {
                    top -= 1;
                    stack[top - 1] = if stack[top - 1] != 0 || stack[top] != 0 { 1 } else { 0 };
                }
                Opt::NOT => {
                    stack[top - 1] = if stack[top - 1] == 0 { 1 } else { 0 };
                }
                Opt::IN => {
                    use std::io::{self, Write};
                    print!("请输入数据：");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    stack[top] = input.trim().parse().unwrap();
                    top += 1;
                }
                Opt::ENTER => {
                    top += instruction.operand as usize; // 为被调函数开辟栈空间
                    outflag = true;
                }
                Opt::RETURN => {
                    top = base;          // 释放被调函数的栈空间
                    ip = stack[top + 1] as usize; // 获取返回地址
                    base = stack[top] as usize;   // 恢复主调函数的基地址
                }
                Opt::CAL => {
                    stack[top] = base as i32;        // 保存当前基地址
                    stack[top + 1] = ip as i32; // 保存返回地址
                    base = top;               // 更新基地址
                    ip = instruction.operand as usize; // 跳转到被调函数
                    outflag = true;
                }
                Opt::PAS => {
                    top -= 1;
                    stack[top + 2] = stack[top]; // 参数传递
                }
                _ => println!("未实现的操作: {}", opt_str),
            }
        } else {
            println!("未知操作码: {}", opt_str);
        }

        if outflag{
            println!("Step{}:  {}    {}", step, opt_str, instruction.operand);
            step += 1;
        }

        else{
            println!("Step{}:  {}", step, opt_str);
            step += 1;
        }

        // println!("Step {}: {}", step, opt_str);
        // step += 1;
        show_stack_info(&stack, top, base);


        if ip == 0 {
            break;
        }
    }
}



// fn main() {
//     let mut file_path = String::new();
//     println!("请输入文件路径: ");  

//     io::stdin()  
//         .read_line(&mut file_path)
//         .expect("无法读取输入");

//     let file_path = file_path.trim(); 



//     match read_codes(file_path) {
//         Ok(codes) => {
//             let map = init_map();
//             display_codes(&codes); // 显示中间代码
//             test_machine(&codes, &map); // 执行抽象机
//         }
//         Err(e) => println!("Error reading codes: {}", e),
//     }
// }
