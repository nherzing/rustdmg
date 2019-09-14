use usize;
use std::fmt;
use std::fs;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug)]
struct Loc(usize, usize);

pub struct Symbols {
    labels: HashMap<Loc, String>
}

impl Symbols {
    pub fn new(path: std::path::PathBuf) -> Self {
        let data = fs::read_to_string(path).unwrap();
        let lines = data.lines().filter(|s| !s.starts_with(";"));
        let mut label_lines = lines.skip_while(|s| !s.starts_with("[labels]"));
        label_lines.next();

        let mut labels = HashMap::new();

        for line in label_lines {
            if line.contains(":") {
                let parts: Vec<&str> = line.split(' ').collect();
                let loc_vs: Vec<&str> = parts[0].split(':').collect();
                let loc = Loc(
                    usize::from_str_radix(loc_vs[0], 16).unwrap(),
                    usize::from_str_radix(loc_vs[1], 16).unwrap()
                );
                labels.insert(loc, parts[1].to_owned());
            }
        }

        Self { labels }
    }

    pub fn get(&self, addr: usize) -> Option<&String> {
        self.labels.get(&Loc(1, addr))
    }
}
