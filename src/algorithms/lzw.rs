use std::collections::HashMap;

#[derive(Default)]
pub struct Lzw {
    pub code_table: HashMap<String, String>,
    pub steps: Vec<LzwStep>,
}


#[derive(Default)]
pub struct LzwStep {
    pub number: usize,
    pub input: char,
    pub buffer: String,
    pub in_dict: bool,
    pub temp: String,
    pub atd: String, // add to (the) dictionary (code_table)
    pub output: String,
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
                output = "--".to_owned();
                temp = buffer.clone();
                atd = "--".to_owned();
            } else {
                output = temp.clone();
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


#[cfg(test)]
mod tests {
    use super::*;

    // Encoding an empty string should produce no code table entries and no steps.
    #[test]
    fn encode_empty_string() {
        let lzw = Lzw::encode("");
        assert!(lzw.code_table.is_empty());
        assert!(lzw.steps.is_empty());
    }

    // A single character produces an initial code for it and a final flush step.
    #[test]
    fn encode_single_char() {
        let lzw = Lzw::encode("a");
        assert_eq!(lzw.code_table.len(), 1);
        assert_eq!(lzw.code_table["a"], "0");
        // One main step + one final flush
        assert_eq!(lzw.steps.len(), 2);
        // First step: char in dict, no output
        assert_eq!(lzw.steps[0].output, "--");
        // Final step flushes the accumulated temp
        assert_eq!(lzw.steps[1].output, "a");
    }

    // Each unique character gets a unique sequential code starting from 0.
    // New substrings encountered during encoding are also added to the table.
    #[test]
    fn encode_two_distinct_chars() {
        let lzw = Lzw::encode("ab");
        // "a"(0), "b"(1), plus "ab"(2) added during encoding
        assert_eq!(lzw.code_table.len(), 3);
        assert_eq!(lzw.code_table["a"], "0");
        assert_eq!(lzw.code_table["b"], "1");
        assert_eq!(lzw.code_table["ab"], "2");
    }

    // The first step always has "--" as output since the first char is always in the initial dict.
    #[test]
    fn first_step_output_is_dash() {
        let lzw = Lzw::encode("abc");
        assert_eq!(lzw.steps[0].output, "--");
        assert!(lzw.steps[0].in_dict);
    }

    // Steps are numbered sequentially starting from 1.
    #[test]
    fn steps_are_numbered() {
        let lzw = Lzw::encode("abc");
        for (i, step) in lzw.steps.iter().enumerate() {
            assert_eq!(step.number, i + 1);
        }
    }

    // The buffer field concatenates the accumulated temp and the current input char.
    #[test]
    fn buffer_contains_temp_plus_input() {
        let lzw = Lzw::encode("ab");
        // Step 1: temp="" + 'a' = "a"
        assert_eq!(lzw.steps[0].buffer, "a");
        assert_eq!(lzw.steps[0].input, 'a');
        // Step 2: temp="a" + 'b' = "ab"
        assert_eq!(lzw.steps[1].buffer, "ab");
        assert_eq!(lzw.steps[1].input, 'b');
    }

    // When the buffer is already in the dictionary, output is "--" and temp extends.
    #[test]
    fn in_dict_no_output() {
        let lzw = Lzw::encode("aba");
        // Step 3: buffer="ba" is new -> output="b", temp resets to "a"
        // Step 2 (ab is new) then step 3 (a follows)
        // Actually let's trace "aba":
        // Init: "a"->0, "b"->1
        // n=0 'a': buf="a" in_dict=YES out="--" temp="a"
        // n=1 'b': buf="ab" in_dict=NO  out="a"  temp="b"
        // n=2 'a': buf="ba" in_dict=NO  out="b"  temp="a"
        // Final: temp="a" -> step 4 output="a"

        // Step 3: n=2, 'a'
        let step = &lzw.steps[2];
        assert_eq!(step.input, 'a');
        assert_eq!(step.buffer, "ba");
        assert!(!step.in_dict);
        assert_eq!(step.output, "b");
        assert_eq!(step.temp, "a");
    }

    // When the buffer is already in the dictionary, output is "--" and temp extends.
    #[test]
    fn reuse_dict_entry() {
        let lzw = Lzw::encode("abab");
        // Step 4: n=3, 'b', buffer="ab" (already in dict)
        let step = &lzw.steps[3];
        assert_eq!(step.input, 'b');
        assert_eq!(step.buffer, "ab");
        assert!(step.in_dict);
        assert_eq!(step.output, "--");
        assert_eq!(step.temp, "ab");
    }

    // The final step flushes the remaining temp as output.
    #[test]
    fn final_step_flushes_temp() {
        let lzw = Lzw::encode("ab");
        let last = lzw.steps.last().unwrap();
        assert_eq!(last.output, "b");
    }

    // The code table grows as new substrings are encountered.
    #[test]
    fn code_table_grows() {
        let lzw = Lzw::encode("abab");
        // Initial entries: "a", "b"
        // New entries added: "ab"(2), "ba"(3)
        assert_eq!(lzw.code_table.len(), 4); // "a", "b", "ab", "ba"
        assert_eq!(lzw.code_table["ab"], "2");
        assert_eq!(lzw.code_table["ba"], "3");
    }

    // Repeated single character should still produce correct LZW encoding.
    #[test]
    fn encode_all_same_char() {
        let lzw = Lzw::encode("aaaa");
        assert_eq!(lzw.code_table.len(), 3); // "a"(0), "aa"(1), "aaa"(2) added on steps 2, 4
        // n=0 'a': buf="a"  in_dict=YES out="--" temp="a"
        // n=1 'a': buf="aa" in_dict=NO  out="a"  temp="a"
        // n=2 'a': buf="aa" in_dict=YES out="--" temp="aa"
        // n=3 'a': buf="aaa"in_dict=NO  out="aa" temp="a"
        // Final: temp="a" -> out="a"
        assert_eq!(lzw.steps.len(), 5); // 4 main + 1 flush
        assert_eq!(lzw.steps[4].output, "a");
    }

    // Unicode characters are handled correctly in code table keys.
    #[test]
    fn encode_unicode_chars() {
        let lzw = Lzw::encode("héllo");
        assert!(lzw.code_table.contains_key("h"));
        assert!(lzw.code_table.contains_key("é"));
        assert!(lzw.code_table.contains_key("l"));
        assert!(lzw.code_table.contains_key("o"));
    }

    // When in_dict is false, atd should contain the buffer and its new code.
    #[test]
    fn atd_format_when_not_in_dict() {
        let lzw = Lzw::encode("ab");
        // Step 2: buffer="ab" not in dict, atd="ab(2)"
        assert_eq!(lzw.steps[1].atd, "ab(2)");
    }

    // When in_dict is true, atd should be "--".
    #[test]
    fn atd_is_dash_when_in_dict() {
        let lzw = Lzw::encode("aba");
        // Step 1: "a" in dict
        assert_eq!(lzw.steps[0].atd, "--");
    }
}

