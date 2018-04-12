use std::io::{Read,Write,Result};
use std::process::Child;

pub trait ProcessIO {
    fn stdout(&mut self) -> Option<String>;
    fn stdin(&mut self, input: String) -> Option<Result<usize>>;
}

impl ProcessIO for Child {
    fn stdout(&mut self) -> Option<String> {
        match self.stdout {
            Some(ref mut pipe) => {
                let mut output = String::new();
                if let Err(_) = pipe.read_to_string(&mut output) {
                    return None;
                }
                Some(output)
            },
            None => None,
        }
    }

    /* stupid return type, but write can error, and there can be no stdin pipe */
    fn stdin(&mut self, input: String) -> Option<Result<usize>> {
        match self.stdin {
            Some(ref mut pipe) => {
                Some(pipe.write(&input.into_bytes()[..]))
            },
            None => None,
        }
    }
}

