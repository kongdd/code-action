use super::*;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn configuration_errors_are_not_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    assert!(load_store_at(path.clone()).is_ok());
    std::fs::write(&path, "{broken").unwrap();
    let error = load_store_at(path.clone()).unwrap_err();
    assert!(error.contains("配置文件损坏"));
    assert!(error.contains(&path.display().to_string()));
    assert_eq!(std::fs::read_to_string(path).unwrap(), "{broken");
}

#[test]
fn open_validates_options_and_paths() {
    assert!(matches!(parse_open(&args(&["--vscode", "/a"])), Ok((Action::Vscode, "/a"))));
    assert!(matches!(parse_open(&args(&["--", "-path"])), Ok((Action::Default, "-path"))));
    for values in [&[][..], &["--bad", "/a"], &["/a", "/b"], &["--zed", "--dir", "/a"]] {
        assert!(parse_open(&args(values)).is_err(), "{values:?}");
    }
}

#[test]
fn remove_requires_exactly_one_path() {
    assert_eq!(parse_remove(&args(&["/a"])).unwrap(), "/a");
    assert_eq!(parse_remove(&args(&["--", "-path"])).unwrap(), "-path");
    for values in [&[][..], &["--bad"], &["/a", "/b"], &["--", "/a", "/b"]] {
        assert!(parse_remove(&args(values)).is_err(), "{values:?}");
    }
}
