use std::fs::File;
use std::path::Path;
use std::io::{self, Read, Write, Seek, SeekFrom};
mod word_analysis;
mod syntax;
mod virtual_machine;





//     let mut file_path = String::new();
//     println!("请输入文件路径: ");  

//     io::stdin()  
//         .read_line(&mut file_path)
//         .expect("无法读取输入");

//     let file_path = file_path.trim(); 



fn main() -> io::Result<()> {
    let mut input_filepath = String::new();
    println!("请输入文件路径：");

    io::stdin()  
    .read_line(&mut input_filepath)
    .expect("无法读取输入");

    let input_filepath = input_filepath.trim();


    let path = Path::new(input_filepath);

    // 不带扩展名的文件名
    let file_name = if let Some(stem) = path.file_stem() {
        stem.to_str().unwrap_or("") // 如果提取失败，则使用空字符串
    } else {
        println!("无法提取文件名");
        return Ok(())
    };


    let output_word = input_filepath.replace(file_name, &format!("{}_word", file_name));


    println!("{}", output_word);


    word_analysis::compile(input_filepath, &output_word);

    let output_syntax_readable = input_filepath.replace(file_name, &format!("{}_syntax_readable", file_name));
    let output_syntax_binary = input_filepath.replace(file_name, &format!("{}_syntax_binary", file_name));


    let mut compiler = syntax::Compiler::new();
    compiler.set_tokenfile(output_word);
    compiler.set_codeout(output_syntax_readable);
    compiler.set_codeout2(output_syntax_binary.clone());

    let es = compiler.test_parse();

    if es == 0 {
        println!("语法、语义分析并生成代码成功!");
    } else {
        println!("错误代码：{}", es);
        println!("语法、语义分析并生成代码错误!");
    }

    // 虚拟机部分
    match virtual_machine::read_codes(&output_syntax_binary){
        Ok(codes) => {
            let map = virtual_machine::init_map();
            // virtual_machine::display_codes(&codes);  // 显示中间代码
            virtual_machine::test_machine(&codes, &map);  // 执行虚拟机
        }
        Err(e) => println!("Error reading codes: {}", e),
    }

    Ok(())
}




