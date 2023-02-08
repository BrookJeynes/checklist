use std::{error::Error, fs, io::Write};

pub fn save_file(contents: String, path: &String) -> Result<(), Box<dyn Error>> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;

    file.write_all(contents.as_bytes())?;
    file.flush()?;

    Ok(())
}
