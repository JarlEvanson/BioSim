#![allow(non_snake_case, non_camel_case_types)]

use std::fmt::Write;

extern crate proc_macro;
use proc_macro::TokenStream;


#[proc_macro]
pub fn mergeEnums(item: TokenStream) -> TokenStream {
    parseTokenStream(&item)
}
#[derive(Debug)]
struct Enumeration {
    name: String,
    values: Vec<String>
}

impl std::fmt::Display for Enumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#[derive(Debug, Copy, Clone)]")?;
        write!(f, "pub enum {} {{ ", self.name)?;
        for value_index in 0..self.values.len() {
            write!(f, "{} = {}", self.values[value_index], value_index)?;
            if value_index < self.values.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f,  " }}")?;
        Ok(())
    }
}

fn parseTokenStream(stream: &TokenStream) -> TokenStream {
    let string = stream.to_string();

    let mut iter = string.split_whitespace().peekable();

    let final_name = iter.next().unwrap().strip_suffix(',').unwrap().to_string();

    let mut enums: Vec<Enumeration> = Vec::new();

    while iter.peek() == Some(&"enum") {
        iter.next();
        let mut name: String = String::new();
        let mut values: Vec<String> = Vec::new();
        let mut iters = 0;
        while iter.peek() != Some(&"}") {
            if iters == 0 {
                name = iter.next().unwrap().to_string();
            } else {
                let str = iter.next().unwrap();
                if str.find('{') != None {
                    continue;
                }
                if str.find('}') != None {
                    break;
                }
                let stripped = str.strip_suffix(',');
                if stripped != None {
                    values.push(stripped.unwrap().to_string());
                } else {
                    values.push(str.to_string());
                }
            }

            iters += 1;
        }

        enums.push(Enumeration { name, values });
    }

    let mut final_string = String::new();

    let mut final_values = Vec::new();

    let mut index = 0;

    for enum_index in 0..enums.len() {
        for variant_index in 0..enums[enum_index].values.len() {
            final_values.push(enums[enum_index].values[variant_index].clone());
        }
        write!(final_string, "pub const FIRST_{}_INDEX: usize = {};\n", enums[enum_index].name, index).expect("Failed to write to output");
        index += enums[enum_index].values.len();
        write!(final_string, "pub const {}_COUNT: usize = {};\n", enums[enum_index].name, enums[enum_index].values.len()).expect("Failed to write to output");
    }

    let final_enum = Enumeration { name: final_name, values: final_values };

    write!(final_string, "pub const {}_COUNT: usize = {};\n", final_enum.name, final_enum.values.len()).expect("Failed to write to output");

    write!(final_string, "{}", final_enum).expect("Failed to write enum");    

    final_string.parse().unwrap()
}
