use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::vec::Vec;
use std::fmt::Write as FmtWrite;
use std::{clone, str};

const MAX_SYMBOL_INDEX: usize = 100;  // 定义符号表的容量
const MAX_CODE_INDEX: usize = 200;    // 中间代码数组的容量

#[derive(Debug)]
enum CategorySymbol {
    Variable,
    Function,
}

#[derive(Debug)]
struct Node {
    s: String,              // 结点文字
    children: Vec<Node>,    // 存放子节点
}

impl Node {
    fn new(word: &str) -> Self {
        Node {
            s: word.to_string(),
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child_node: Node) {
        self.children.push(child_node);
    }
}

#[derive(Debug)]
struct Symbol {
    name: String,
    kind: CategorySymbol,
    address: i32,
    action_function: String,
    var_num: i32, // 参数数量，若本身为变量，则此值为0
}

#[derive(Debug)]
struct Code {
    opt: String,  // 操作码
    operand: i32,  // 操作数
}

#[derive(Debug)]
struct Compiler {
    codesIndex: i32,
    token: String,
    token1: String,
    tokenfile: String,
    codeout: String,
    syntaxtree: String,
    fp_tokenin: Option<File>,   // 单词流文件指针
    fp_code_binary: Option<File>, // 中间代码二进制文件指针
    fp_code_text: Option<File>,   // 中间代码文本文件指针
    fp_syntaxtree: Option<File>,  // 语法树文件指针
    codes: Vec<Code>,            // 中间代码数组
    symbol: Vec<Symbol>,         // 符号表
    symbol_index: usize,         // 符号表当前索引
    codes_index: usize,          // 中间代码数组当前索引
    es: i32,                     // 错误码
    root: Option<Node>,          // 语法树根节点
    Lastdefinedfunction: String,
            numofvariable:usize,
            offset:i32,
}

fn fscanf_token(tokenfile: &str) -> io::Result<(String, String)> {
    // 定义 token 和 token1
    let mut token = String::new();
    let mut token1 = String::new();

    // 打开文件
    let file = File::open(tokenfile)?;
    let reader = io::BufReader::new(file);

    // 读取文件中的一行
    for line in reader.lines() {
        let line = line?; // 解包错误
        let mut parts = line.split_whitespace(); // 按空白字符拆分

        if let Some(first) = parts.next() {
            token = first.to_string(); // 第一个值赋给 token
        }
        if let Some(second) = parts.next() {
            token1 = second.to_string(); // 第二个值赋给 token1
        }

        // 读取到两个 token 后返回
        break;
    }

    Ok((token, token1)) // 返回 token 和 token1
}


impl Compiler {
    fn new() -> Self {
        Compiler {
            codesIndex: 0,
            token: String::new(),
            token1: String::new(),
            tokenfile: String::new(),
            codeout: String::new(),
            syntaxtree: String::new(),
            fp_tokenin: None,
            fp_code_binary: None,
            fp_code_text: None,
            fp_syntaxtree: None,
            codes: Vec::new(),
            symbol: Vec::with_capacity(MAX_SYMBOL_INDEX),
            symbol_index: 0,
            codes_index: 0,
            es: 0,
            root: None,
            Lastdefinedfunction: String::new(),
            numofvariable:0,
            offset:0,
        }
    }

    fn add_child(&mut self, n: &mut Node) {
        //let mut str = self.token1.clone();
        let child_node = Node::new(str);
        n.add_child(child_node);
    }

    fn test_parse(&mut self) -> i32 {
        self.codes_index = 0;
        let mut es = 0;

        // 读取文件名
        println!("请输入单词流文件名（包括路径）：");
        let mut tokenfile = String::new();
        io::stdin().read_line(&mut tokenfile).unwrap();
        self.tokenfile = tokenfile.trim().to_string();

        self.fp_tokenin = OpenOptions::new().read(true).open(&self.tokenfile).ok();

        if self.fp_tokenin.is_none() {
            println!("\n打开{}错误!", self.tokenfile);
            return 10;
        }

        es = self.program();
        // if es != 0 {
        //     return es;
        // }
        println!("==语法、语义分析及代码生成程序结果==");
        match self.es {
            0 => println!("语法、语义分析成功并抽象机汇编生成代码!"),
            10 => println!("打开文件 {} 失败!", self.tokenfile),
            1 => println!("缺少{{!"),
            2 => println!("缺少}}!"),
            3 => println!("缺少标识符!"),
            4 => println!("少分号!"),
            5 => println!("缺少(!"),
            6 => println!("缺少)!"),
            7 => println!("缺少操作数!"),
            11 => println!("函数开头缺少{{!"),
            12 => println!("函数结束缺少}}!"),
            13 => println!("最后一个函数的名字应该是main!"),
            21 => println!("符号表溢出!"),
            22 => println!("变量 {} 重复定义!", self.token1),
            23 => println!("变量未声明!"),
            24 => println!("程序中main函数结束后，还有其它多余字符"),
            25 => println!("参数设置未结束!"),
            26 => println!("传参未结束!"),
            32 => println!("函数名重复定义!"),
            34 => println!("call语句后面的标识符 {} 不是变量名!", self.token1),
            35 => println!("read语句后面的标识符不是变量名!"),
            36 => println!("赋值语句的左值 {} 不是变量名!", self.token1),
            37 => println!("因子对应的标识符不是变量名!"),
            38 => println!("函数传入的参数数量不对!"),
            _ => {}
        }
        println!("请输入要生成的文本形式的中间代码文件的名字（包括路径）：");
        let mut codeout = String::new();
        io::stdin().read_line(&mut codeout).unwrap();
        self.codeout = codeout.trim().to_string();

        let fp_code_text = File::create(&self.codeout);
        let mut fp_code_text = match fp_code_text {
            Ok(file) => file,
            Err(_) => {
                println!("\n创建 {} 错误!", self.codeout);
                self.es = 10;
                return self.es;
            }
        };

        for i in 0..self.codes_index {
            if ["LOAD", "LOADI", "STO", "BR", "BRF", "CAL", "ENTER"].contains(&self.codes[i].opt.as_str()) {
                writeln!(fp_code_text, " {:3} {:<5} {:<3}", i, self.codes[i].opt, self.codes[i].operand).unwrap();
            } else {
                writeln!(fp_code_text, " {:3} {:<5}", i, self.codes[i].opt).unwrap();
            }
        }

        // 生成二进制形式的中间代码文件
        println!("请输入要生成的二进制形式的中间代码文件的名字（结构体存储）:");
        let mut codeout = String::new();
        io::stdin().read_line(&mut codeout).unwrap();
        let fp_code_binary = File::create(&codeout);
        let mut fp_code_binary = match fp_code_binary {
            Ok(file) => file,
            Err(_) => {
                println!("\n创建 {} 错误!", codeout.trim());
                self.es = 10;
                return self.es;
            }
        };

        // 写入二进制文件
        use std::io::Write;
        let bytes = bincode::serialize(&self.codes).unwrap();
        fp_code_binary.write_all(&bytes).unwrap();

        // 生成语法树文件
        println!("请输入要生成的语法树文件的名字:");
        let mut syntaxtree = String::new();
        io::stdin().read_line(&mut syntaxtree).unwrap();
        self.syntaxtree = syntaxtree.trim().to_string();

        let fp_syntaxtree = File::create(&self.syntaxtree);
        let mut fp_syntaxtree = match fp_syntaxtree {
            Ok(file) => file,
            Err(_) => {
                println!("\n创建 {} 错误!", self.syntaxtree);
                self.es = 10;
                return self.es;
            }
        };

        // 输出语法树
        self.output_tree(self.root.as_ref().unwrap(), 0);

        es
    }

    fn program(&mut self) -> i32 {
        let mut es = 0;

        
        
        // 读取token
        
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        self.root = Some(Node::new("<program>"));
        let  root = self.root.as_mut().unwrap();


        // 添加无条件跳转指令，跳转到 main 函数入口
        self.codes[self.codes_index].opt = "BR".to_string();
        // self.codes[self.codes_index].operand = String::from("main");  // 假设 main 地址会在后面填充
        self.codes_index += 1;

        

        while self.token == "function" {
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;

            es = self.fun_declaration(root);
            if es != 0 {
                return es;
            }
            
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;
        }


        if self.token != "ID" {
            es = 1; // 错误，缺少ID
            return es;
        }

        if self.token1 != "main" {
            es = 13; // 错误，最后一个函数必须是main
            return es;
        }

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        

        es = self.main_declaration(root);

        if es > 0 {
            return es;
        }

        if !self.is_end_of_file() {
            es = 24; // 程序结束后有多余字符
            return es;
        }

        // 输出符号表内容
        println!("符号表");
        println!("名字\t \t类型 \t地址\t作用函数");
        for symbol in &self.symbol {
            println!(
                "{:<8} \t{:?} \t{:?} \t{:?}",
                symbol.name, symbol.kind, symbol.address, symbol.action_function
            );
        }
        // for symbol in &self.symbol {
        //     // 使用 println! 宏来格式化输出符号表的内容
        //     println!(
        //         "{:<8} \t{:>3} \t{:>3} \t{:<8}",
        //         symbol.name, symbol.kind, symbol.address, symbol.action_function
        //     );
        es
    }

// 自定义一个方法来判断文件是否已结束
    fn is_end_of_file(&self) -> bool {
        if let Some(fp) = &self.fp_tokenin {
            let mut buf_reader = BufReader::new(fp);
            let mut buffer = String::new();

            match buf_reader.read_to_string(&mut buffer) {
                Ok(0) => true,  // 文件已经读取完毕
                Ok(_) => false, // 仍有内容
                Err(_) => false, // 错误时，也视为未到达 EOF
            }
        } else {
            true // 如果文件指针为空，直接返回 true
        }
    }




    fn fun_declaration(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut child_node = Node::new("<fun_declaration>");
        root.add_child(child_node);

        if self.token != "ID" {
            es = 2;
            return es;
        }
        self.add_child(&mut child_node);
        self.insert_symbol("function", &self.token1); // 将函数名插入符号表

        let mut temp = String::from(&self.token1);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != "(" {
            es = 5;
            return es;
        }
        self.add_child(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        let mut fun_pos = 0;
        self.symbol[self.symbol_index - 1].address = self.codes_index; // 将函数体的入口地址填入符号表中的地址
        es = self.parameter_list(&mut child_node);
        if es > 0 {
            return es;
        }

        // es = self.lookup(&temp, &mut fun_pos);
        es=self.lookup(&temp, &mut symbol_pos, &self.Lastdefinedfunction);
        if es > 0 {
            return es;
        }

        self.symbol[fun_pos].var_num = self.num_of_variable;
        if self.token != ")" {
            es = 6;
            return es;
        }
        self.add_child(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es = self.function_body(&mut child_node);

        es
    }

    fn main_declaration(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut child_node = Node::new("<main_declaration>");
        root.add_child(child_node);

        self.insert_symbol("function", "main");

        if self.token != "(" {
            es = 5;
            return es;
        }
        self.add_child(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        self.symbol[self.symbol_index - 1].address = self.codes_index; // 填写函数体地址
        es = self.parameter_list(&mut child_node);

        if es > 0 {
            return es;
        }

        if self.token != ")" {
            es = 6;
            return es;
        }
        self.add_child(&mut child_node);

        self.codes[0].operand = self.codes_index; // 设置代码跳转目标

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es = self.function_body(&mut child_node);

        es
    }

    fn function_body(&mut self, root: &mut Node) -> i32 {
        let mut es;
        let mut child_node = Node::new("<function_body>");
        root.add_child(child_node);

        if self.token != "{" {
            es = 11;
            return es;
        }

        child_node = Node::new("<function_body>");
        self.add_child(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es = self.declaration_list(&mut child_node);
        if es > 0 {
            return es;
        }

        self.codes.push(Code {
            opt: String::from("ENTER"),
            operand: self.offset, // 假设offset为2
        });
        self.codes_index += 1;

        es = self.statement_list(&mut child_node);

        if es > 0 {
            return es;
        }

        if self.token != "}" {
            es = 12;
            return es;
        }
        self.add_child(&mut child_node);

        self.codes.push(Code {
            opt: String::from("RETURN"),
            operand: 0,
        });

        es
    }

    fn declaration_list(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;

        let child_node = Node::new("<declaration_list>");
        root.add_child(child_node);

        while self.token == "int" {
            es = self.declaration_stat(root);
            if es > 0 {
                return es;
            }
        }

        es
    }

    // // <declaration_stat> -> int ID;
    fn declaration_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut child_node = Node::new("<declaration_stat>");
        root.add_child(child_node);

        child_node = Node::new("<declaration_stat>");
        self.add_child(&mut child_node);

        // 读取 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != "ID" {
            return 3; // 错误：不是标识符
        }

        es = self.insert_symbol("variable", &self.token1); // 插入符号表
        child_node = Node::new("<declaration_stat>");
        self.add_child(&mut child_node);

        if es > 0 {
            return es;
        }

        // 读取下一个 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != ";" {
            return 4; // 错误：缺少分号
        }

        self.add_child(&mut child_node);

        // 读取下一个 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es
    }

    // <statement_list> -> { <statement> }
    fn statement_list(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;

        let mut child_node = Node::new("<statement_list>");
        root.add_child(child_node);
        child_node = Node::new("<statement_list>");

        while self.token != "}" {
            es = self.statement(&mut child_node);
            if es > 0 {
                return es;
            }
        }

        es
    }

    // // <statement> -> <if_stat> | <while_stat> | <for_stat>
    // //             | <compound_stat> | <expression_stat> | <call_stat>
    fn statement(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;

        let mut child_node = Node::new("<statement>");
        root.add_child(child_node);
        child_node = Node::new("<statement>");

        if es==0 && self.token == "if" {
            es = self.if_stat(&mut child_node); // <if 语句>
        }
        if es==0&&self.token == "while" {
            es = self.while_stat(&mut child_node); // <while>
        } 
        if es==0&&self.token == "for" {
            es = self.for_stat(&mut child_node); // <for 语句>
        } 
        if es==0&&self.token == "read" {
            es = self.read_stat(&mut child_node); // <read 语句>
        }
        if es==0&&self.token == "write" {
            es = self.write_stat(&mut child_node); // <write 语句>
        }
        if es==0&&self.token == "{" {
            es = self.compound_stat(&mut child_node); // <复合语句>
        }
        if es==0&&self.token == "call" {
            es = self.call_stat(&mut child_node); // <函数调用语句>
        }
        if es==0&&(self.token == "ID" || self.token == "NUM" || self.token == "(") {
            es = self.expression_stat(&mut child_node); // <表达式语句>
        }

        es
    }

    // // <if_stat> -> if '(' <expr> ')' <statement> [else <statement>]
    fn if_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut cx1;
        let mut cx2;

        let mut child_node = Node::new("<if_stat>");
        root.add_child(child_node);

        self.add_child(&mut child_node);

        // 读取 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        if self.token != "(" {
            return 5; // 错误：缺少左括号
        }
        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        if self.token != ")" {
            return 6; // 错误：缺少右括号
        }
        self.add_child(&mut child_node);

        // 生成条件判断的指令
        self.codes[self.codes_index].opt = "BRF".to_string();
        cx1 = self.codes_index;
        self.codes_index += 1;

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es = self.statement(&mut child_node);
        if es > 0 {
            return es;
        }

        // 生成跳转指令
        self.codes[self.codes_index].opt = "BR".to_string();
        cx2 = self.codes_index;
        self.codes_index += 1;
        self.codes[cx1].operand = self.codes_index;

        // 处理 else 部分
        if self.token == "else" {
            self.add_child(&mut child_node);
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;
            es = self.statement(&mut child_node);
            if es > 0 {
                return es;
            }
        }

        self.codes[cx2].operand = self.codes_index;
        es
    }

    // // <while_stat> -> while '(' <expr> ')' <statement>
    fn while_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut cx1;
        let mut cx_entrance;

        let mut child_node = Node::new("<while_stat>");
        root.add_child(child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        self.add_child(&mut child_node);

        if self.token != "(" {
            return 5; // 错误：缺少左括号
        }
        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        cx_entrance = self.codes_index;
        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        if self.token != ")" {
            return 6; // 错误：缺少右括号
        }
        self.add_child(&mut child_node);

        // 生成条件判断的指令
        self.codes[self.codes_index].opt = "BRF".to_string();
        cx1 = self.codes_index;
        self.codes_index += 1;

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es = self.statement(&mut child_node);
        if es > 0 {
            return es;
        }

        // 生成跳转指令
        self.codes[self.codes_index].opt = "BR".to_string();
        self.codes[self.codes_index].operand = cx_entrance;
        self.codes_index += 1;
        self.codes[cx1].operand = self.codes_index;

        es
    }

    // // <for_stat> -> for '(' <expr> ; <expr> ; <expr> ')' <statement>
    fn for_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut cx1;
        let mut cx2;
        let mut cx_exp2;
        let mut cx_exp3;

        let mut child_node = Node::new("<for_stat>");
        root.add_child(child_node);

        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != "(" {
            return 5; // 错误：缺少左括号
        }
        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        if self.token != ";" {
            return 4; // 错误：缺少分号
        }
        self.add_child(&mut child_node);
        cx_exp2 = self.codes_index;

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        self.codes[self.codes_index].opt = "BRF".to_string();
        cx1 = self.codes_index;
        self.codes_index += 1;

        self.codes[self.codes_index].opt = "BR".to_string();
        cx2 = self.codes_index;
        self.codes_index += 1;

        if self.token != ";" {
            return 4; // 错误：缺少分号
        }
        self.add_child(&mut child_node);
        cx_exp3 = self.codes_index;

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        self.codes[self.codes_index].opt = "BR".to_string();
        self.codes[self.codes_index].operand = cx_exp2;
        self.codes_index += 1;
        self.codes[cx2].operand = self.codes_index;

        if self.token != ")" {
            return 6; // 错误：缺少右括号
        }
        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es = self.statement(&mut child_node);
        if es > 0 {
            return es;
        }

        self.codes[self.codes_index].opt = "BR".to_string();
        self.codes[self.codes_index].operand = cx_exp3;
        self.codes_index += 1;
        self.codes[cx1].operand = self.codes_index;

        es
    }

    // // <write_stat> -> write <expression>;
    fn write_stat(&mut self, root: &mut Node) -> i32 {
        let es;

        let mut child_node = Node::new("<write_stat>");
        root.add_child(child_node);

        child_node = Node::new("<write_stat>");
        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        if self.token != ";" {
            return 4; // 错误：缺少分号
        }
        self.add_child(&mut child_node);

        // 生成输出指令
        self.codes[self.codes_index].opt = "OUT".to_string();
        self.codes_index += 1;

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        es
    }

    fn read_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut child_node = Node::new("<read_stat>");
        root.add_child(child_node);

        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != "ID" {
            return 3; // 错误：缺少标识符
        }

        self.add_child(&mut child_node);
        let mut symbol_pos = 0;
        es = self.lookup(&self.token1, &mut symbol_pos, &self.Lastdefinedfunction);
        if es > 0 {
            return es;
        }

        if self.symbol[symbol_pos].kind != "variable" {
            return 35; // 错误：符号不是变量
        }

        self.codes[self.codes_index].opt = "IN".to_string();
        self.codes_index += 1;
        self.codes[self.codesIndex].opt = "STO".to_string();
        self.codes[self.codes_index].operand = self.symbol[symbol_pos].address;
        self.codes_index += 1;

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != ";" {
            return 4; // 错误：缺少分号
        }

        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es
    }

    // // <compound_stat> -> '{' <statement_list> '}'
    fn compound_stat(&mut self, root: &mut Node) -> i32 {
        //let mut es: i32;
        let mut child_node = Node::new("<compound_stat>");
        root.add_child(child_node);
        child_node = Node::new("<compound_stat>");

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;


        let es = self.statement_list(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es
    }

    // // <call_stat> -> call ID '(' <variable_list> ')'
    fn call_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut symbol_pos = 0;
        let mut child_node = Node::new("<call_stat>");
        root.add_child(child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        if self.token != "ID" {
            return 3; // 错误：缺少标识符
        }

        self.add_child(&mut child_node);
        es = self.lookup(&self.token1, &mut symbol_pos, &self.token1); // 查找函数
        if es > 0 {
            return es;
        }

        if self.symbol[symbol_pos].kind != "function" {
            return 34; // 错误：标识符不是函数
        }

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != "(" {
            return 5; // 错误：缺少左括号
        }

        self.add_child(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;
        if self.symbol[symbol_pos].varnum != 0 {
            es = self.variable_list(&mut child_node, self.symbol[symbol_pos].varnum);
            let mut a = self.symbol[symbol_pos].varnum;
            while a != 0 {
                self.codes[self.codes_index].opt = "PAS".to_string();
                self.codes_index += 1;
                a -= 1;
            }

            if es > 0 {
                return es;
            }
        }

        if self.token != ")" {
            return 6; // 错误：缺少右括号
        }

        self.add_child(&mut child_node);

        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != ";" {
            return 4; // 错误：缺少分号
        }

        self.add_child(&mut child_node);
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        self.codes[self.codes_index].opt = "CAL".to_string();
        self.codes[self.codes_index].operand = self.symbol[symbol_pos].address;
        self.codes_index += 1;

        es
    }

    // // <expression_stat> -> <expression> ';'
    fn expression_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;
        let mut child_node = Node::new("<expression_stat>");
        root.add_child(child_node);

        child_node = Node::new("<expression_stat>");

        if self.token == ";" {
            self.add_child(&mut child_node);

            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;
            return es;
        }

        child_node = Node::new("<expression_stat>");
        es = self.expression(&mut child_node);
        if es > 0 {
            return es;
        }

        if self.token == ";" {
            self.add_child(&mut child_node);
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;
            return es;
        } else {
            return 4; // 错误：缺少分号
        }
    }

    // // <expression> -> ID = <bool_expr> | <bool_expr>
    fn expression(&mut self, root: &mut Node) -> i32 {
        let mut es;
        let mut file_add = 0;
        let mut child_node = Node::new("<expression>");
        root.add_child(child_node);
        let mut token2 = String::new();
        let mut token3 = String::new();
        let mut symbol_pos = 0;

        if self.token == "ID" {
            file_add = self.fp_tokenin.tell();//有tell这个函数吗？

            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            token2 = new_token;  // 分别赋值
            token3 = new_token1;

            if self.token2 == "=" {
                self.add_child(&mut child_node);
                let mut son = Node::new(token2);
                child_node.add_child(son);
                es = self.lookup(&self.token1, &mut symbol_pos, &self.Lastdefinedfunction);
                if es > 0 {
                    return es;
                }

                if self.symbol[symbol_pos].kind != "variable" {
                    return 36; // 错误：不是变量
                }

                let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
                self.token = new_token;  // 分别赋值
                self.token1 = new_token1;
                es = self.bool_expr(&mut child_node);
                if es > 0 {
                    return es;
                }

                self.codes[self.codes_index].opt = "STO".to_string();
                self.codes[self.codes_index].operand = self.symbol[symbol_pos].address;
                self.codes_index += 1;
            } else {
                self.fp_tokenin.seek(file_add, 0); // 回到"="之前
                es = self.bool_expr(&mut child_node);
                if es > 0 {
                    return es;
                }
            }
        } else {
            es = self.bool_expr(&mut child_node);
        }

        es
    }
    // // <bool_expr> -> <additive_expr> | <additive_expr> ( > | < | >= | <= | == | != ) <additive_expr>
    fn bool_expr(&mut self, root: &mut Node) -> i32 {
        let mut es;
        let mut child_node = Node::new("<bool_expr>");
        root.add_child(child_node);

        // 处理 addtive_expr 部分
        child_node = Node::new("<bool_expr>");
        es = self.additive_expr(&mut child_node);
        if es > 0 {
            return es;
        }

        // 处理关系运算符部分
        if ["=", ">", ">=", "<", "<=", "==", "!="].contains(&self.token.as_str()) {
            self.add_child(&mut child_node);

            let token2 = self.token.clone(); // 保存运算符
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;

            es = self.additive_expr(&mut child_node);
            if es > 0 {
                return es;
            }

            // 根据 token2 设置不同的操作符
            match token2.as_str() {
                ">" =>  self.add_code("GT"),
                ">=" => self.add_code("GE"),
                "<" => self.add_code("LES"),
                "<=" => self.add_code("LE"),
                "==" => self.add_code("EQ"),
                "!=" => self.add_code("NOTEQ"),
                _ => {}
            }
        }

        es
    }

    // // <additive_expr> -> <term> { (+ | -) <term> }
    fn additive_expr(&mut self, root: &mut Node) -> i32 {
        let mut es;
        let mut child_node = Node::new("<additive_expr>");
        root.add_child(child_node);

        child_node = Node::new("<additive_expr>");

        es = self.term(&mut child_node);
        if es > 0 {
            return es;
        }

        // 处理 + 或 - 运算符
        while self.token == "+" || self.token == "-" {
            self.add_child(&mut child_node);
            let token2 = self.token.clone(); // 保存运算符
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;

            es = self.term(&mut child_node);
            if es > 0 {
                return es;
            }

            match token2.as_str() {
                "+" => self.add_code("ADD"),
                "-" => self.add_code("SUB"),
                _ => {}
            }
        }

        es
    }

    // // <term> -> <factor> { (* | /) <factor> }
    fn term(&mut self, root: &mut Node) -> i32 {
        let mut es: i32;
        let mut child_node = Node::new("<term>");
        root.add_child(child_node);
        child_node = Node::new("<term>");

        es = self.factor(&mut child_node);
        if es > 0 {
            return es;
        }

        // 处理 * 或 / 运算符
        while self.token == "*" || self.token == "/" {
            self.add_child(&mut child_node);
            let token2 = self.token.clone(); // 保存运算符
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;

            es = self.factor(&mut child_node);
            if es > 0 {
                return es;
            }

            match token2.as_str() {
                "*" => self.add_code("MULT"),
                "/" => self.add_code("DIV"),
                _ => {}
            }
        }

        es
    }

    // // 辅助方法：添加操作代码
    fn add_code(&mut self, op: &str) {
        // 假设 `codes` 是一个存储代码的数组，`codesIndex` 是当前索引
        self.codes[self.codes_index].opt = op.to_string();
        self.codes_index += 1;
    }
    // // <factor> -> '(' <additive_expr> ')' | ID | NUM
    fn factor(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;

        let mut child_node = Node::new("<factor>");
        root.add_child(child_node);

        if self.token == "(" {
            self.add_child(&mut child_node);
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;
            es = self.additive_expr(&mut child_node);
            if es > 0 {
                return es;
            }
            if self.token != ")" {
                return 6; // 错误：少右括号
            }
            self.add_child(&mut child_node);
            let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
            self.token = new_token;  // 分别赋值
            self.token1 = new_token1;
        } else {
            if self.token == "ID" {
                self.add_child(&mut child_node);
                let mut symbol_pos: i32 = 0;
                es = self.lookup(&self.token1, &mut symbol_pos, &self.Lastdefinedfunction);
                if es > 0 {
                    return es; // 变量未定义
                }

                if self.symbol[symbol_pos as usize].kind != SymbolKind::Variable {
                    return 37; // 变量语义检查失败
                }

                self.codes[self.codes_index as usize].opt = "LOAD".to_string();
                self.codes[self.codes_index as usize].operand = self.symbol[symbol_pos as usize].address;
                self.codes_index += 1;

                let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
                self.token = new_token;  // 分别赋值
                self.token1 = new_token1;
                return es;
            }

            if self.token == "NUM" {
                self.add_child(&mut child_node);
                self.codes[self.codes_index as usize].opt = "LOADI".to_string();
                self.codes[self.codes_index as usize].operand = self.token1.parse::<i32>().unwrap();
                self.codes_index += 1;

                let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
                self.token = new_token;  // 分别赋值
                self.token1 = new_token1;
                return es;
            } else {
                es = 7; // 错误：缺少操作数
                return es;
            }
        }

        es
    }

    // // 插入符号到符号表
    fn insert_symbol(&mut self, category: CategorySymbol, name: &str) -> i32 {
        if self.symbol_index >= self.max_symbol_index {
            return 21; // 符号表溢出
        }

        let mut es = 0;

        match category {
            CategorySymbol::Function => {
                self.last_defined_function = name.to_string();
                for i in (0..self.symbol_index).rev() {
                    if self.symbol[i as usize].name == name && self.symbol[i as usize].kind == SymbolKind::Function {
                        es = 32; // 错误：函数名重复
                        break;
                    }
                }
                self.symbol[self.symbol_index as usize].kind = SymbolKind::Function;
            }
            CategorySymbol::Variable => {
                for i in (0..self.symbol_index).rev() {
                    if self.symbol[i as usize].name == name && self.symbol[i as usize].kind == SymbolKind::Variable
                        && self.symbol[i as usize].action_function == self.last_defined_function
                    {
                        es = 22; // 错误：同一作用域内变量重复定义
                        break;
                    }
                }
                self.symbol[self.symbol_index as usize].kind = SymbolKind::Variable;
                self.symbol[self.symbol_index as usize].address = self.offset;
                self.offset += 1; // 数据区指针加1
            }
        }

        if es > 0 {
            return es;
        }

        self.symbol[self.symbol_index as usize].name = name.to_string();
        self.symbol[self.symbol_index as usize].action_function = self.last_defined_function.clone();
        self.symbol_index += 1;

        es
    }

    // // 查找符号表中的标识符
    fn lookup(&self, name: &str, p_position: &mut i32, fun_name: &str) -> i32 {
        for i in 0..self.symbol_index {
            if self.symbol[i as usize].name == name && self.symbol[i as usize].action_function == fun_name {
                *p_position = i;
                return 0; // 找到符号
            }
        }

        23 // 错误：标识符未定义
    }

    // 用于区分父子节点
    fn shift(&mut self, n: usize) {
        if let Some(ref mut file) = self.fp_syntaxtree {
            for _ in 0..n {
                write!(file, "    ").unwrap();  // 向文件写入四个空格
            }
        }
    }

    // 输出语法树
    fn output_tree(&mut self, root: &Node, n: usize) {
        self.shift(n);  // 写入空格
        if let Some(ref mut file) = self.fp_syntaxtree {
            writeln!(file, "{}", root.s).unwrap();  // 写入当前节点的字符串
        }

        let child_num = root.child.len();
        if child_num == 0 {
            return;
        } else {
            for i in 0..child_num {
                self.output_tree(&root.child[i], n + 1);  // 递归输出子树并增大空格数
            }
        }
    }
    // // <parameter_stat> -> int ID
    fn parameter_stat(&mut self, root: &mut Node) -> i32 {
        let es;

        let mut child_node = Node::new("<parameter_stat>");
        root.add_child(child_node);

        child_node = Node::new("<parameter_stat>");
        self.add_child(&mut child_node);

        // 读取 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        if self.token != "ID" {
            return 3; // 错误：不是标识符
        }

        let token = self.token1.clone();
        es = self.insert_symbol(CategorySymbol::Variable, &token); // 插入符号表
        self.add_child(&mut child_node);

        if es > 0 {
            return es;
        }

        // 读取下一个 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        es
    }

    // // <parameter_list> -> { <parameter_stat> ',' } <parameter_stat>
    fn parameter_list(&mut self, root: &mut Node) -> i32 {
        let mut flag = 0; // 判断是否参数设置错误
        let mut es = 0;
        self.offset=2;
        let mut child_node = Node::new("<parameter_list>");
        root.add_child(child_node);
        child_node = Node::new("<parameter_list>");


        if self.token != ")" {
            while self.token == "int" {
                flag = 0;
                self.numofvariable += 1;

                es = self.parameter_stat(&mut child_node);
                if es > 0 {
                    return es;
                }

                if self.token == "," {
                    let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
                    self.token = new_token;  // 分别赋值
                    self.token1 = new_token1;
                    flag = 1;
                } else {
                    break;
                }
            }

            if flag == 1 {
                es = 25; // 错误：参数列表格式错误
            }

            es
        } else {
            es
        }
    }

    // // <variable_stat> -> ID
    fn variable_stat(&mut self, root: &mut Node) -> i32 {
        let mut es = 0;

        let mut child_node = Node::new("<variable_stat>");
        root.add_child(child_node);


        self.add_child(&mut child_node);

        // 检查传入的参数是否已定义
        let mut pos: usize = 0;
        es = self.lookup(&self.token1, &mut pos, &self.Lastdefinedfunction);
        if es > 0 {
            return es;
        }

        // 读取下一个 token
        let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
        self.token = new_token;  // 分别赋值
        self.token1 = new_token1;

        // 生成代码
        self.codes[self.codes_index].opt = "LOAD".to_string();
        self.codes[self.codes_index].operand = self.symbol[pos].address;
        self.codes_index += 1;

        es
    }

    // // <variable_list> -> { <variable_stat> ',' } <variable_stat>
    fn variable_list(&mut self, root: &mut Node, num: i32) -> i32 {
        let mut flag = 0;
        let mut cnt = 0;
        let mut es = 0;


        let mut child_node = Node::new("<variable_list>");
        root.add_child(child_node);
        child_node = Node::new("<variable_list>");

        if self.token != ")" {
            while self.token == "ID" {
                flag = 0;
                cnt += 1;

                es = self.variable_stat(&mut child_node);
                if es > 0 {
                    return es;
                }

                if self.token == "," {
                    let (new_token, new_token1) = fscanf_token(&self.tokenfile).unwrap(); // 解构返回的元组
                    self.token = new_token;  // 分别赋值
                    self.token1 = new_token1;
                    flag = 1;
                } else {
                    break;
                }
            }

            if flag == 1 {
                es = 26; // 错误：变量列表格式错误
            } else if cnt != num {
                es = 38; // 错误：变量数目不符
            }

            es
        } else {
            es
        }
    }

}

fn main() {
    let mut compiler = Compiler::new();

    let es = compiler.test_parse();
    if es == 0 {
        println!("语法、语义分析并生成代码成功!");
    } else {
        println!("语法、语义分析并生成代码错误!");
    }
}
