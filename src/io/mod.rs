use std::fs::File;
use std::rc::Rc;

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