//@ # The Driver



use codemap::{CodeMap, FileMap, Span};

//@ The `Driver` contains a `CodeMap` and various other configuration settings 
//@ required to run the analysis.

/// The driver is in charge of orchestrating the whole analysis process and 
/// making sure all the bits and pieces integrate nicely.
#[derive(Debug)]
pub struct Driver {
    codemap: CodeMap,
}

//@ He has various methods to allow users to add files to be analysed, as well as 
//@ other convenience methods for setting things up.

impl Driver {
    /// Create a new driver.
    pub fn new() -> Driver {
        Driver {
            codemap: CodeMap::new(),
        }
    }

    /// Get access to the driver's `CodeMap`.
    pub fn codemap(&mut self) -> &mut CodeMap {
        &mut self.codemap
    }
}

impl Default for Driver {
    fn default() -> Driver {
        Driver::new()
    }
}
