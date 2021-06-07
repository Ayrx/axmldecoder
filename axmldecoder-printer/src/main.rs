use anyhow::Result;

use axmldecoder::Element;
use std::fs::File;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let fname = args.get(1).unwrap();

    let mut f = File::open(fname)?;
    let xml = axmldecoder::parse(&mut f)?;

    let root = xml.get_root().as_ref().unwrap();

    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    format_xml(&root, 0_usize, &mut s);

    let s = s.trim().to_string();
    println!("{}", s);

    Ok(())
}

fn format_xml(e: &Element, level: usize, output: &mut String) {
    output.push_str(&format!(
        "{:indent$}{}\n",
        "",
        &format_start_element(&e),
        indent = level * 2
    ));

    for child in e.get_children() {
        format_xml(&child, level + 1, output)
    }

    if !e.get_children().is_empty() {
        output.push_str(&format!(
            "{:indent$}{}\n",
            "",
            &format_end_element(&e),
            indent = level * 2
        ));
    }
}

fn format_start_element(e: &Element) -> String {
    let mut s = String::new();
    s.push('<');
    s.push_str(e.get_tag());

    if e.get_tag() == "manifest" {
        s.push(' ');
        s.push_str("xmlns:android=\"http://schemas.android.com/apk/res/android\"");
    }

    for (key, val) in e.get_attributes().iter() {
        s.push(' ');
        s.push_str(key);
        s.push('=');
        s.push('"');
        s.push_str(val);
        s.push('"');
    }

    if e.get_children().is_empty() {
        s.push('/');
    }

    s.push('>');

    s
}

fn format_end_element(e: &Element) -> String {
    let mut s = String::new();
    s.push('<');
    s.push('/');
    s.push_str(e.get_tag());
    s.push('>');
    s
}
