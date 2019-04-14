use std::fs::File;
use std::rc::Rc;
use std::io::Write;

pub struct LocalSourceFile {
    pub path: String,
    pub file: Rc<File>
}

impl LocalSourceFile {
    pub fn new(path: String) -> LocalSourceFile {
        match File::open(&path) {
            Ok(f) => {
                LocalSourceFile {
                    path: path,
                    file: Rc::new(f)
                }
            },
            Err(e) => {
                panic!("Could not open file, {:?}",e);
            }
        }
    }
}

pub struct LocalOutputFile {
    pub path: String,
    pub file: Rc<File>
}

impl LocalOutputFile {
    pub fn new(path: &str) -> LocalOutputFile {
        match File::create(&path) {
            Ok(f) => {
                LocalOutputFile {
                    path: String::from(path),
                    file: Rc::new(f)
                }
            },
            Err(e) => {
                panic!("Could not open file, {:?}",e);
            }
        }
    }
    pub fn write_all(&mut self, buf: Vec<u8>) {
        let mut output_file = Rc::get_mut(&mut self.file).unwrap();
        output_file.write_all(&buf);
    }
}