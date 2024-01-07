use dirs;
use std::fs::DirBuilder;
use std::path::PathBuf;

pub fn get_cache_name(name: &str) -> String {
    let mut path = dirs::home_dir().unwrap();
    path.push(".cache/jadwal-shalat/");
    let _ = create_cache_dir(&path);
    // println!("Create directory: {:?}", path);
    // let _ = DirBuilder::new().recursive(true).create(&path);

    let cache_dir = path.into_os_string().into_string().unwrap();
    cache_dir + name
}

fn create_cache_dir(path: &PathBuf) -> Result<(), std::io::Error> {
    DirBuilder::new().recursive(true).create(&path)?;
    Ok(())
}
