use ::rustfmt::{format_input, Input};
use regex::Regex;
use scraper::{Html, Selector};
use std::fs::{read_to_string, File};
use std::io::{Sink, Write};
use std::process::Command;

pub fn format_code(orig: String) -> String {
    format_input(Input::Text(orig), &<_>::default(), None::<&mut Sink>)
        .map(|(_, v, _)| v.into_iter().next())
        .ok()
        .flatten()
        .map(|(_, m)| m.to_string())
        .expect("source_code input should be formatted")
}

fn get_doc(path: &str) -> String {
    String::from_utf8(
        Command::new("rustup")
            .args(["doc", "--path", path])
            .output()
            .expect("To find the HTML document")
            .stdout,
    )
    .expect("to have path")
    .replace('\n', "")
}

fn get_mod_paths(path: &str) -> Vec<String> {
    let modules =
        Regex::new(r#"item-left module-item"><a class="mod" href="(?P<href>.*?)""#).unwrap();
    let doc = &read_to_string(path).expect("to have document");
    modules
        .captures_iter(doc)
        .map(|p| {
            path.trim()
                .replace("index.html", p.name("href").unwrap().as_str())
        })
        .collect()
}

fn get_trait_paths(path: &str) -> Vec<String> {
    let traits =
        Regex::new(r#"item-left module-item"><a class="trait" href="(?P<href>trait.*?)""#).unwrap();
    let doc = &read_to_string(path).expect("to have document");
    traits
        .captures_iter(doc)
        .map(|p| {
            path.trim()
                .replace("index.html", p.name("href").unwrap().as_str())
        })
        .collect()
}

fn get_trait_code(path: &str) -> String {
    let code = &read_to_string(path.trim()).expect("no error");
    let fragment = Html::parse_fragment(code);

    let trait_selector = Selector::parse(".rust.trait").unwrap();
    let pre = fragment.select(&trait_selector).next().unwrap();

    pre.text().collect::<Vec<_>>().join("")
}

fn create_traits_from(base: &str) {
    let fix_where_clause = Regex::new(r#"where(?P<name>.*?):"#).unwrap();
    let trait_name = Regex::new(r#"trait (?P<name>[A-Za-z]*)"#).unwrap();
    let core_path = get_doc(base);
    let core_mod_paths = get_mod_paths(&core_path);

    let mut names = Vec::new();
    let mut traits = Vec::new();

    core_mod_paths
        .iter()
        .filter_map(|path| {
            let paths = get_trait_paths(path.trim());
            if !paths.is_empty() {
                Some(paths)
            } else {
                None
            }
        })
        .flat_map(|paths| {
            paths
                .iter()
                .filter_map(|path| {
                    let tr_code = get_trait_code(path.trim()).replace(|c: char| !c.is_ascii(), "");

                    if tr_code.contains("{ ... }") || tr_code.contains("auto trait") {
                        None
                    } else {
                        let code = fix_where_clause
                            .replace(&tr_code, " where $name:")
                            .to_string();
                        Some(code)
                    }
                })
                .collect::<Vec<String>>()
        })
        .for_each(|trait_code| {
            let fmtd_code = format_code(trait_code);
            let trait_ident = trait_name
                .captures(&fmtd_code)
                .unwrap()
                .name("name")
                .unwrap()
                .as_str();

            names.push(trait_ident.to_string());
            traits.push(fmtd_code.clone());

            let file = File::create(format!("../src/dispatch/{}/{}.rs", base, trait_ident));
            file.unwrap()
                .write_all(&fmtd_code.into_bytes())
                .expect("write file");
        });

    let core_enum = format_code(format!("enum Core {{ {} }}", names.join(",")));

    let impl_from_str = format_code(format!(
        "impl FromStr for Core {{ fn from_str(value: &str) -> Self {{ match value {{  {}  }} }} }}",
        names
            .iter()
            .map(|n| { format!("\"{}\" => Self::{}", n, n) })
            .collect::<Vec<_>>()
            .join(",")
    ));

    let impl_from_enum = format_code(format!(
        "impl From<Core> for Dispatcher<ItemTrait> {{ fn from(value: Core) -> Self {{ match value {{  {}  }} }} }}",
        names.iter()
            .map(|n| { format!("Self::{} => parse_quote!(include!(\"./std/{}.rs\"))", n, n) })
            .collect::<Vec<_>>()
            .join(",")
    ));

    let file = File::create("../src/dispatch/std.rs");
    let file2 = File::create("../src/dispatch/transform.rs");
    let file3 = File::create("../src/dispatch/enum.rs");

    file.unwrap()
        .write_all(&impl_from_enum.into_bytes())
        .expect("write file");
    file2
        .unwrap()
        .write_all(&impl_from_str.into_bytes())
        .expect("write file");
    file3
        .unwrap()
        .write_all(&core_enum.into_bytes())
        .expect("write file");
}

fn main() {
    create_traits_from("std");
}
