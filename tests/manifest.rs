use std::path::Path;
use toml::Table;

fn read_toml(name: &str) -> Table {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
        .parse::<Table>()
        .unwrap_or_else(|e| panic!("cannot parse {name}: {e}"))
}

fn extension() -> Table {
    read_toml("extension.toml")
}

fn str_field<'a>(table: &'a Table, key: &str) -> &'a str {
    table
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("missing string field `{key}`"))
}

#[test]
fn id_uses_allowed_characters() {
    let manifest = extension();
    let id = str_field(&manifest, "id");
    assert!(!id.is_empty());
    assert!(id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'));
}

#[test]
fn id_does_not_contain_zed_or_extension() {
    let manifest = extension();
    let id = str_field(&manifest, "id");
    assert!(!id.contains("zed"), "id must not contain `zed`");
    assert!(!id.contains("extension"), "id must not contain `extension`");
}

#[test]
fn schema_version_is_1() {
    let manifest = extension();
    assert_eq!(manifest["schema_version"].as_integer(), Some(1));
}

#[test]
fn version_matches_cargo_toml() {
    let cargo = read_toml("Cargo.toml");
    let cargo_version = cargo["package"]["version"].as_str().unwrap();
    assert_eq!(str_field(&extension(), "version"), cargo_version);
}

#[test]
fn every_language_has_a_language_id() {
    let manifest = extension();
    let server = &manifest["language_servers"]["cssmodules"];
    let ids = server["language_ids"]
        .as_table()
        .expect("language_ids table");
    let languages = server["languages"].as_array().expect("languages array");
    assert!(!languages.is_empty());
    for language in languages {
        let name = language.as_str().unwrap();
        assert!(ids.contains_key(name), "no language_id for `{name}`");
    }
}

#[test]
fn no_orphan_language_ids() {
    let manifest = extension();
    let server = &manifest["language_servers"]["cssmodules"];
    let languages: Vec<&str> = server["languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect();
    for key in server["language_ids"].as_table().unwrap().keys() {
        assert!(languages.contains(&key.as_str()), "orphan id `{key}`");
    }
}
