// Copyright 2020 Oxide Computer Company

pub struct Reader<I, F> {
    input: I,
    lookup: F,
}

#[derive(Debug, PartialEq)]
pub enum Entry {
    Include(String),
    Dir(Dir),
    File(File),
    Link(Link),
    Unknown(String),
}

#[derive(Default, Debug, PartialEq)]
pub struct FsAttr {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Dir {
    pub path: String,
    pub attr: FsAttr,
}

#[derive(Debug, PartialEq)]
pub struct File {
    pub path: String,
    pub attr: FsAttr,
    pub chash: Option<String>,
    pub cname: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Link {
    pub path: String,
    pub attr: FsAttr,
    pub target: String,
}

impl Entry {
    pub fn get_path(&self) -> Option<&str> {
        match self {
            Entry::Dir(dir) => Some(&dir.path),
            Entry::File(file) => Some(&file.path),
            Entry::Link(link) => Some(&link.path),
            _ => None
        }
    }
}

impl<I, F> Reader<I, F>
where
    I: Iterator<Item = String>,
    F: Fn(&str) -> Option<String>,
{
    pub fn new(input: I, lookup: F) -> Self {
        Self { input, lookup }
    }
}

impl<I, F> Iterator for Reader<I, F>
where
    I: Iterator<Item = String>,
    F: Fn(&str) -> Option<String>,
{
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = get_full_line(&mut self.input) {
            let replaced = replace_vars(line.as_str(), &self.lookup);
            // Replacement of variables may have emptied or commented out the line
            match replaced.chars().nth(0) {
                Some('#') | None => {
                    continue;
                }
                Some(_) => {}
            }
            return Some(parse_entry(replaced.trim()));
        }
        None
    }
}

fn get_full_line<I: Iterator<Item = String>>(iter: &mut I) -> Option<String> {
    let mut line = iter.next()?;

    // ignore any commented lines
    while line.starts_with('#') || line.is_empty() {
        line = iter.next()?;
    }

    // Handle backslash continuation
    while line.ends_with(" \\") {
        line.pop();
        if let Some(next_line) = iter.next() {
            line.push_str(&next_line)
        }
    }
    Some(line)
}

fn replace_vars<F>(line: &str, lookup: F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let mut result = String::with_capacity(line.len());
    let mut tokens = line.split('$');

    // Copy anything leading to the first (if any) '$'
    result.push_str(tokens.next().unwrap());

    for sub in tokens {
        match (sub.find('('), sub.find(')')) {
            (Some(0), Some(x)) => {
                // Follows $(VAR_NAME) form
                if let Some(replace) = lookup(&sub[1..x]) {
                    result.push_str(&replace);
                }
                if x < sub.len() - 1 {
                    result.push_str(&sub[(x + 1)..]);
                }
            }
            _ => {
                // Does not follow $(VAR_NAME) form, so do no replacement
                result.push('$');
                result.push_str(sub);
            }
        }
    }

    result
}

// <include system-library.man3ldap.inc>
// dir path=lib
// file path=lib/$(ARCH64)/c_synonyms.so.1
// link path=lib/$(ARCH64)/libadm.so target=libadm.so.1

fn parse_file_fields(input: &str) -> (Option<String>, Option<String>, FsAttr, Option<String>, Option<String>) {
    let mut chash: Option<String> = None;
    let mut cname: Option<String> = None;
    let mut path: Option<String> = None;
    let mut target: Option<String> = None;
    let mut attrs: FsAttr = Default::default();

    // Find all name=value pairs (TODO: handle name="quoted value")
    let items = input.split_whitespace().filter_map(|x| {
        if let Some(idx) = x.find('=') {
            let (name, raw_value) = x.split_at(idx);
            Some((Some(name), &raw_value[1..]))
        } else {
            Some((None, x))
        }
    });

    for (name, value) in items {
        if let Some(field) = match name {
            Some("owner") => Some(&mut attrs.owner),
            Some("group") => Some(&mut attrs.group),
            Some("mode") => Some(&mut attrs.mode),
            Some("path") => Some(&mut path),
            Some("target") => Some(&mut target),
            Some("chash") => Some(&mut chash),
            None => Some(&mut cname),
            _ => None,
        } {
            *field = Some(value.to_string());
        }
    }

    (path, target, attrs, chash, cname)
}

fn parse_entry(input: &str) -> Entry {
    let (kind, rest) = input.split_at(input.find(' ').unwrap_or(0));
    let rest = rest.trim_start();

    match kind {
        "<include" => {
            let len = rest.len();
            if len != 0 && rest.bytes().last().unwrap() == b'>' {
                return Entry::Include(rest[..len - 1].to_string());
            }
        }
        "dir" => {
            if let (Some(path), _, attr, _, _) = parse_file_fields(rest) {
                return Entry::Dir(Dir { path, attr });
            }
        }
        "file" => {
            if let (Some(path), _, attr, chash, cname) = parse_file_fields(rest) {
                return Entry::File(File { path, attr, chash, cname, });
            }
        }
        "link" => {
            if let (Some(path), Some(target), attr, _, _) = parse_file_fields(rest) {
                return Entry::Link(Link { path, attr, target });
            }
        }
        _ => {}
    }
    Entry::Unknown(input.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Cursor, BufRead};

    #[test]
    fn comments_and_continuations() {
        let input = r"
# comment line
normal line
cont \
line
# empty line below

long \
cont \
line";
        let br = Cursor::new(input);
        // Assume (for now) that valid input is utf8 clean
        let mut iter = br.lines().filter_map(|x| x.ok());

        assert_eq!(get_full_line(&mut iter), Some("normal line".to_string()));
        assert_eq!(get_full_line(&mut iter), Some("cont line".to_string()));
        assert_eq!(get_full_line(&mut iter), Some("long cont line".to_string()));
        assert_eq!(get_full_line(&mut iter), None);
    }

    #[test]
    fn var_replacing() {
        let clean_cases = &[
            "clean",
            "$leading sign",
            "trailing sign$",
            "mid$mid",
            "two$$two",
            "unmatched parens $(content",
            "spaced $ (some)",
        ];
        for case in clean_cases.iter() {
            assert_eq!(
                *case,
                replace_vars(case, |_| panic!("shound not be called"))
            );
        }

        let simple_fn = |x: &str| Some(x.to_uppercase());
        let simple_cases = &[
            ("one $(a)", "one A"),
            ("$(a) two $(b)", "A two B"),
            ("$(a)$(b)$(c)", "ABC"),
        ];
        for (inp, outp) in simple_cases.iter() {
            assert_eq!(*outp, replace_vars(inp, simple_fn));
        }
        assert_eq!(
            "elidevar",
            replace_vars("elide$(a)var", |x| {
                assert_eq!(x, "a");
                None
            })
        );
    }
    #[test]
    fn entry_parsing() {
        assert_eq!(
            parse_entry("<include something.inc>"),
            Entry::Include("something.inc".to_string())
        );
        assert_eq!(
            parse_entry("dir path=bin"),
            Entry::Dir(Dir {
                path: "bin".to_string(),
                attr: Default::default()
            })
        );
        assert_eq!(
            parse_entry("file path=bin/ls"),
            Entry::File(File {
                path: "bin/ls".to_string(),
                attr: Default::default()
            })
        );
        assert_eq!(
            parse_entry("file path=bin/secret owner=special group=selective mode=0540"),
            Entry::File(File {
                path: "bin/secret".to_string(),
                attr: FsAttr {
                    owner: Some("special".to_string()),
                    group: Some("selective".to_string()),
                    mode: Some("0540".to_string()),
                }
            })
        );
        assert_eq!(
            parse_entry("link path=bin/redirect target=secret"),
            Entry::Link(Link {
                path: "bin/redirect".to_string(),
                attr: Default::default(),
                target: "secret".to_string(),
            })
        );
    }
}
