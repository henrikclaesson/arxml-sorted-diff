use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub short_name: Option<String>,
    pub uuid: Option<String>,
    pub attributes: HashMap<String, String>,
    pub text: Option<String>,
    pub children: Vec<Element>,
}

impl Element {
    pub fn new(tag: String) -> Self {
        Self { tag, short_name: None, uuid: None, attributes: HashMap::new(), text: None, children: vec![] }
    }
}

fn local_name(name: &str) -> &str {
    match name.rfind(':') {
        Some(idx) => &name[idx+1..],
        None => name,
    }
}

pub fn parse_file(path: &str) -> Result<Element> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut stack: Vec<Element> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut el = Element::new(tag);
                for attr in e.attributes().with_checks(false) {
                    if let Ok(a) = attr {
                        let key = String::from_utf8_lossy(a.key.as_ref()).to_string();
                        if let Ok(val) = a.unescape_value() {
                            let v = val.to_string();
                            el.attributes.insert(key.clone(), v.clone());
                            if key.eq_ignore_ascii_case("UUID") { el.uuid = Some(v); }
                        }
                    }
                }
                stack.push(el);
            }
            Ok(Event::Text(e)) => {
                if let Some(last) = stack.last_mut() {
                    let txt = e.unescape().unwrap_or_else(|_| std::borrow::Cow::Borrowed(""));
                    let t = txt.trim().to_string();
                    if !t.is_empty() {
                        // Accumulate text for the current element (may arrive in several chunks)
                        match &mut last.text {
                            Some(existing) => {
                                existing.push(' ');
                                existing.push_str(&t);
                            }
                            None => last.text = Some(t),
                        }
                    }
                }
            }
            Ok(Event::End(_e)) => {
                if let Some(el) = stack.pop() {
                    // If this element is a SHORT-NAME, attach its text to the parent.short_name
                    if local_name(&el.tag).eq_ignore_ascii_case("SHORT-NAME") {
                        if let Some(parent) = stack.last_mut() {
                            parent.short_name = el.text.clone();
                            // do not add the SHORT-NAME as a child to avoid duplication
                            continue;
                        }
                    }

                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(el);
                    } else {
                        // root element finished
                        return Ok(el);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!(e)),
            _ => {}
        }
        buf.clear();
    }
    Err(anyhow::anyhow!("No root element found"))
}
