//! The crate is a single compilation unit, so its layering is a convention.
//! These checks read the sources and fail when production code breaks it:
//! `client → gameplay → simulation` only, and no cycles between client modules.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

/// Production text of every source file under `directory`, by relative path.
fn sources(directory: &str) -> Vec<(String, String)> {
    fn visit(root: &Path, directory: &Path, found: &mut Vec<(String, String)>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, found);
                continue;
            }
            let name = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .to_string();
            if !name.ends_with(".rs") || name.ends_with("tests.rs") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            // Unit tests may reach across layers to build fixtures.
            let production = text
                .split("#[cfg(test)]\nmod tests")
                .next()
                .unwrap()
                .to_string();
            found.push((name, production));
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(directory);
    let mut found = Vec::new();
    visit(&root, &root, &mut found);
    found
}

#[test]
fn lower_layers_never_reference_the_layers_above_them() {
    for (layer, forbidden) in [
        ("simulation", &["crate::client", "crate::gameplay"][..]),
        ("gameplay", &["crate::client"][..]),
    ] {
        for (file, text) in sources(layer) {
            for name in forbidden {
                assert!(!text.contains(name), "{layer}/{file} references {name}");
            }
        }
    }
}

/// Top-level client modules which `text` names through `super::` / `client::`
/// paths or inside a `use super::{…}` group.
fn referenced<'a>(text: &str, modules: &'a BTreeSet<String>) -> BTreeSet<&'a str> {
    let mut groups = String::new();
    for opener in [
        "use super::{",
        "use super::super::{",
        "use crate::client::{",
    ] {
        for part in text.split(opener).skip(1) {
            groups.push_str(part.split("};").next().unwrap());
            groups.push('\n');
        }
    }
    let boundary = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
    let mut found = BTreeSet::new();
    for module in modules {
        let direct = ["super::", "client::"].iter().any(|prefix| {
            let path = format!("{prefix}{module}");
            text.match_indices(&path)
                .any(|(at, _)| boundary(text[at + path.len()..].chars().next()))
        });
        let grouped = groups.match_indices(module.as_str()).any(|(at, _)| {
            let before = groups[..at].chars().next_back();
            let after = groups[at + module.len()..].chars().next();
            boundary(after) && before.is_none_or(|c| c.is_whitespace() || c == '{' || c == ',')
        });
        if direct || grouped {
            found.insert(module.as_str());
        }
    }
    found
}

#[test]
fn client_modules_do_not_depend_on_each_other_in_a_cycle() {
    let files = sources("client");
    let module_of = |file: &str| file.split(['/', '.']).next().unwrap().to_string();
    let modules: BTreeSet<String> = files
        .iter()
        .map(|(file, _)| module_of(file))
        .filter(|module| module != "mod")
        .collect();
    let mut uses: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (file, text) in &files {
        let owner = module_of(file);
        let Some(owner) = modules.get(&owner) else {
            continue; // client/mod.rs composes every module.
        };
        let entry = uses.entry(owner.as_str()).or_default();
        entry.extend(
            referenced(text, &modules)
                .into_iter()
                .filter(|module| *module != owner),
        );
    }
    assert!(uses["combat"].contains("avatar"), "the scan found no edges");

    // Depth-first search; `path` is the chain of modules currently open.
    fn visit<'a>(
        module: &'a str,
        uses: &BTreeMap<&'a str, BTreeSet<&'a str>>,
        done: &mut BTreeSet<&'a str>,
        path: &mut Vec<&'a str>,
    ) {
        if let Some(start) = path.iter().position(|open| *open == module) {
            let mut cycle = path[start..].to_vec();
            cycle.push(module);
            panic!("client modules form a cycle: {}", cycle.join(" → "));
        }
        if !done.insert(module) {
            return;
        }
        path.push(module);
        for next in uses.get(module).into_iter().flatten() {
            visit(next, uses, done, path);
        }
        path.pop();
    }
    let mut done = BTreeSet::new();
    for module in uses.keys() {
        visit(module, &uses, &mut done, &mut Vec::new());
    }
}
