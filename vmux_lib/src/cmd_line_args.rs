pub struct CmdLineArgs {
    args: Vec<String>,
}

impl CmdLineArgs {
    pub fn new() -> CmdLineArgs {
        let args: Vec<String> = std::env::args().collect();

        CmdLineArgs { args }
    }

    pub fn has_key(&mut self, key: &str) -> bool {
        for f in &self.args {
            if f == &format!("--{}", key) {
                return true;
            }
        }
        false
    }
    pub fn get_key(&mut self, key: &str) -> Option<String> {
        let mut next = false;
        for f in &self.args {
            if next {
                return Some(f.to_owned());
            }
            if f == &format!("--{}", key) {
                next = true;
            }
        }
        None
    }
}
