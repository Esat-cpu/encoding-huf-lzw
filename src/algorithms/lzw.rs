use std::collections::HashMap;

#[derive(Default)]
pub struct Lzw {
    pub code_table: HashMap<String, String>,
    pub steps: Vec<LzwStep>,
}


#[derive(Default)]
pub struct LzwStep {
    number: usize,
    input: char,
    buffer: String,
    in_dict: bool,
    temp: String,
    atd: String, // add to (the) dictionary (code_table)
    output: String,
}


impl Lzw {
    pub fn encode(st: &str) -> Lzw {
        let mut lzw = Lzw::default();
        let mut temp = "".to_owned();


        // Assign sequential codes to each unique character
        for ch in st.chars() {
            let code = lzw.code_table.len().to_string();
            lzw.code_table.entry(ch.to_string()).or_insert(code);
        }


        for (n, input) in st.chars().enumerate() {
            let buffer = format!("{temp}{input}");
            let in_dict = lzw.code_table.contains_key(&buffer);

            let output: String;
            let atd: String;

            if in_dict {
                output = temp.clone();
                temp = buffer.clone();
                atd = "--".to_owned();
            } else {
                output = "--".to_owned();
                temp = input.to_string();
                let code = lzw.code_table.len();
                atd = format!("{buffer}({code})");
                lzw.code_table.insert(buffer.clone(), code.to_string());
            }

            lzw.steps.push(LzwStep {
                number: n + 1,
                input,
                buffer,
                in_dict,
                temp: temp.clone(),
                atd,
                output,
            });
        }

        if !temp.is_empty() {
            lzw.steps.push(LzwStep {
                number: lzw.steps.len() + 1,
                output: temp,
                in_dict: true,
                ..Default::default()
            });
        }

        lzw
    }
}

