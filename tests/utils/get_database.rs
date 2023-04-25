use std::{fs::read_to_string, path::PathBuf};

pub fn get_config(mut paths: Vec<PathBuf>) -> Result<String, Box<dyn std::error::Error>> {
    let paths = {
        let mut default_paths = vec![
            PathBuf::from("tests/config.txt"),
            PathBuf::from("tests/config.ci.txt"),
        ];
        paths.append(&mut default_paths);

        paths
    };

    for path in paths.iter() {
        if path.is_file() {
            return read_to_string(path).map_err(|e| e.into());
        }
    }

    Err(format!(
        "Could not find any of the given configuration files in {{{}}}'.",
        paths
            .iter()
            .map(|p| format!("'{}'", p.display()))
            .collect::<Vec<String>>()
            .join(", ")
    )
    .into())
}
