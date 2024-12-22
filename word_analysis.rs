use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};

const KEYWORDS: &[&str] = &["int", "if", "while", "for", "else", "read", "write", "function", "call"];
const SINGLE_WORDS: &[char] = &['+', '-', '*', '(', ')', ';', ',', ':', '{', '}'];
const DOUBLE_WORDS: &[char] = &['<', '>', '=', '!'];

fn is_single_word(ch: char) -> bool {
    SINGLE_WORDS.contains(&ch)
}

fn is_double_word(ch: char) -> bool {
    DOUBLE_WORDS.contains(&ch)
}

fn compile_word(rfile: &mut File, wfile: &mut File, row: &mut usize) -> io::Result<i32> {
    let mut word = String::new();
    let mut buffer = [0; 1];

    // 跳过空白字符，并记录行号
    loop {
        if rfile.read(&mut buffer)? == 0 {
            return Ok(-1); // 文件结束
        }
        let ch: char = buffer[0] as char;
        if ch == '\n' {
            *row += 1;
        } else if !ch.is_whitespace() {
            break; // 跳过空白字符并继续
        }
    }

    // 读取当前字符
    let ch = buffer[0] as char;

    if ch.is_alphabetic() {
        // 处理标识符或关键字
        word.push(ch);
        while rfile.read(&mut buffer)? != 0 {
            let next_ch = buffer[0] as char;
            if next_ch.is_alphanumeric() {
                word.push(next_ch);
            } else {
                // 回退未处理的字符
                rfile.seek(SeekFrom::Current(-1))?; // 向前回退1个字节
                break;
            }
        }
        let word_lower = word.to_lowercase();
        if KEYWORDS.contains(&word_lower.as_str()) {
            writeln!(wfile, "\t{}\t\t{}", word_lower, word)?;
        } else {
            writeln!(wfile, "\tID\t\t{}", word)?;
        }
    } else if ch.is_digit(10) {
        // 处理数字
        word.push(ch);
        while rfile.read(&mut buffer)? != 0 {
            let next_ch = buffer[0] as char;
            if next_ch.is_digit(10) {
                word.push(next_ch);
            } else {
                // 回退未处理的字符
                rfile.seek(SeekFrom::Current(-1))?; // 向前回退1个字节
                break;
            }
        }
        writeln!(wfile, "\tNUM\t\t{}", word)?;
    } else if is_single_word(ch) {
        // 处理单字符操作符（包括括号）
        writeln!(wfile, "\t{}\t\t{}", ch, ch)?;
    } else if is_double_word(ch) {
        // 处理双字符操作符
        let mut operator = String::new();
        operator.push(ch);
        if rfile.read(&mut buffer)? != 0 {
            let next_ch = buffer[0] as char;
            if next_ch == '=' {
                operator.push(next_ch);
            } else {
                // 回退未处理的字符
                rfile.seek(SeekFrom::Current(-1))?; // 向前回退1个字节
            }
        }
        writeln!(wfile, "\t{}\t\t{}", operator, operator)?;
    } else {
        // 处理非法字符
        println!("错误：非法字符：{}\t错误位置在第{}行", ch, row);
        return Ok(2);
    }

    Ok(0)
}

fn compile(input_filename: &str, output_filename: &str) -> io::Result<()> {
    let input_file = File::open(input_filename)?;
    let mut rfile = input_file;  // 使用 File 类型，而不是 BufReader
    let mut wfile = File::create(output_filename)?;

    //writeln!(wfile, "--------------------编译结果--------------------")?;

    let mut row = 1;
    loop {
        match compile_word(&mut rfile, &mut wfile, &mut row)? {
            -1 => break,
            0 => (),
            _ => println!("错误发生在第{}行", row),
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let input_filename = "D:\\my_project\\src\\a.txt";
    let output_filename = "D:\\my_project\\src\\b.txt";

    match compile(input_filename, output_filename) {
        Ok(_) => println!("编译完成，结果已保存到{}", output_filename),
        Err(e) => println!("编译失败: {}", e),
    }

    Ok(())
}
