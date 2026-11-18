/// 将受支持的 /mnt 挂载路径转换为 NAS 上的绝对路径。
pub fn nas_path(path: &str) -> Result<String, &'static str> {
    let rest = path.strip_prefix("/mnt/").ok_or("路径必须以 /mnt/ 开头")?;
    let (mount, suffix) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let volume = match mount {
        "x" => "/volume2/GitHub",
        "z" => "/volume1/CMIP6",
        "y" => "/volume1/Researches",
        "o" => "/volume1/CUG-hydro",
        _ => return Err("不支持的挂载点，仅支持 /mnt/x、/mnt/z、/mnt/y、/mnt/o"),
    };
    Ok(format!("{volume}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::nas_path;

    #[test]
    fn converts_mapped_paths() {
        for (input, expected) in [
            ("/mnt/x", "/volume2/GitHub"),
            ("/mnt/z/", "/volume1/CMIP6/"),
            ("/mnt/y/my project/data.csv", "/volume1/Researches/my project/data.csv"),
            ("/mnt/o/水文", "/volume1/CUG-hydro/水文"),
        ] {
            assert_eq!(nas_path(input).unwrap(), expected);
        }
    }

    #[test]
    fn rejects_unmapped_paths() {
        let paths =
            ["/mnt/xray/data", "/mnt/w", "/mnt/", "/mntx/x", "/tmp/mnt/x", "/volume2/GitHub"];
        for path in paths {
            assert!(nas_path(path).is_err(), "{path}");
        }
    }
}
