use std::path::Path;
use sysinfo::System;

pub struct ProcessesWatcher<'a> {
    filter: Box<dyn ProcessesFilter + 'a>
}

pub trait ProcessesFilter : Send {
    fn is_restricted(&self, exe: &Path) -> bool;
}

impl ProcessesWatcher<'_> {
    pub fn new() -> Self {
        ProcessesWatcher {
            filter: Box::new(NoneFilter{})
        }
    }

    pub fn check(&self) {
        let sys = System::new_all();

        for (_, process) in sys.processes() {
            if let Some(path) = process.exe() {
                if self.filter.is_restricted(path) {
                    process.kill();
                }
            }
        }
    }
}

impl<'a> ProcessesWatcher<'a> {
    pub fn set_filter(&mut self, filter: impl ProcessesFilter + 'a) {
        self.filter = Box::new(filter);
    }

}

pub struct BlacklistFilter {
    paths: Vec<Box<Path>>
}

impl BlacklistFilter {
    pub fn new(paths: Vec<Box<Path>>) -> Self {
        BlacklistFilter {
            paths
        }
    }
}

impl ProcessesFilter for BlacklistFilter {
    fn is_restricted(&self, exe: &Path) -> bool {
        for path in &self.paths {
            if **path == *exe {
                return true
            }
        }

        false
    }
}

pub struct NoneFilter {

}

impl ProcessesFilter for NoneFilter {
    fn is_restricted(&self, _: &Path) -> bool {
        false
    }
}