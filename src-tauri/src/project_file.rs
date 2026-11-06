use crate::project::Project;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

pub fn read_project(path: &Path) -> Result<Project, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Не удалось открыть архив: {e}"))?;
    let entry = archive
        .by_name("project.json")
        .map_err(|_| "В архиве отсутствует project.json.")?;
    if entry.size() > 64 * 1024 * 1024 {
        return Err("Файл проекта слишком велик.".into());
    }
    let mut json = String::new();
    entry
        .take(64 * 1024 * 1024 + 1)
        .read_to_string(&mut json)
        .map_err(|e| e.to_string())?;
    let project: Project =
        serde_json::from_str(&json).map_err(|e| format!("Некорректный файл проекта: {e}"))?;
    project.validate()?;
    Ok(project)
}

pub fn write_project(path: &Path, project: &Project) -> Result<(), String> {
    project.validate()?;
    let directory = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(directory).map_err(|e| e.to_string())?;
    {
        let mut zip = ZipWriter::new(temporary.as_file_mut());
        zip.start_file(
            "project.json",
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
        )
        .map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut zip, project).map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    temporary.as_file_mut().flush().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cps::parse_cps;
    #[test]
    fn cps_and_project_roundtrip() {
        let input = "FSASCI 0 1 COMPUTED 0 1e30\nFSNROW 2 3\nFSLIMI 10 30 40 50 0 9\n->Surface\n0 1e30 2 3 4 5";
        let grid = parse_cps(input).unwrap();
        assert_eq!(
            grid.values,
            vec![None, Some(3.0), Some(5.0), Some(0.0), Some(2.0), Some(4.0)]
        );
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.wellforge");
        let project = Project {
            grid: Some(grid),
            ..Project::default()
        };
        write_project(&path, &project).unwrap();
        let loaded = read_project(&path).unwrap();
        assert_eq!(
            loaded.grid.unwrap().values,
            project.grid.as_ref().unwrap().values
        );
        let mut updated = project.clone();
        updated.epsilon = 0.75;
        updated.grid.as_mut().unwrap().values[0] = Some(6186631.463839343);
        write_project(&path, &updated).unwrap();
        let loaded = read_project(&path).unwrap();
        assert_eq!(loaded.epsilon, 0.75);
        assert_eq!(loaded.grid.unwrap().values[0], Some(6186631.463839343));
        updated.format_version = 2;
        assert!(write_project(&path, &updated).is_err());
        assert_eq!(read_project(&path).unwrap().epsilon, 0.75);
        assert!(parse_cps(&input.replace("4 5", "4")).is_err());
    }
}
