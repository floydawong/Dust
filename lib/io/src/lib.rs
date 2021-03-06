
use std::path::{Path, PathBuf};
use std::{fs};
use std::io::{self, BufReader};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    FileContainsNil
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::Io(other)
    }
}

#[cfg(target_arch = "x86_64")]
pub fn load_string(resource_name: &str) -> Result<String, Error>
{
    let mut read_buffer = load_read_buffer(resource_name)?;
    let buffer = read_buffer.fill_buf()?;

    if buffer.iter().find(|i| **i == 0).is_some() {
        return Err(Error::FileContainsNil);
    }
    let temp = std::str::from_utf8(buffer).unwrap();
    Ok(temp.to_string())
}

fn load_read_buffer(resource_name: &str) -> Result<Box<io::BufRead>, Error>
{
    let root_path: PathBuf = PathBuf::from("");
    let file = fs::File::open(
        resource_name_to_path(&root_path,resource_name)
    )?;

    let buffer = BufReader::new(file);
    Ok(Box::new(buffer))
}

fn resource_name_to_path(root_dir: &Path, location: &str) -> PathBuf {
    let mut path: PathBuf = root_dir.into();

    for part in location.split("/") {
        path = path.join(part);
    }

    path
}


#[cfg(target_arch = "wasm32")]
pub fn load_string(resource_name: &str) -> Result<String, Error>
{
    Ok("".to_string())
}


/*#[cfg(target_os = "emscripten")]
pub fn load(name: &str) -> Result<Box<io::BufRead>, Error>
{
    use emscripten::{emscripten};
    emscripten::wget(name);
    println!("Out name: {}", name);
    load_read_buffer(name)
}

#[cfg(not(target_os = "emscripten"))]
pub fn load(name: &str) -> Result<Box<io::BufRead>, Error>
{
    load_read_buffer(name)
}

#[cfg(target_os = "emscripten")]
pub fn fetch(url: &str)
{
    use emscripten::{emscripten};
    emscripten::fetch(url);
}

#[cfg(not(target_os = "emscripten"))]
pub fn fetch(_url: &str)
{

}

#[cfg(target_os = "emscripten")]
pub fn load_async<F>(name: &str, mut on_load: F) where F: FnMut(Box<io::BufRead>)
{
    let on_l = |temp: String| {
        let data = load_read_buffer(temp.as_str()).unwrap();
        on_load(data);
    };
    let on_error = |cause: String| {
        panic!(cause);
    };
    use emscripten::{emscripten};
    emscripten::async_wget(name, on_l, on_error);
}

#[cfg(not(target_os = "emscripten"))]
pub fn load_async<F>(name: &str, mut on_load: F) where F: FnMut(Box<io::BufRead>)
{
    let data = load_read_buffer(name).unwrap();
    on_load(data);
}
*/