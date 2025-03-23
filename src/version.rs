pub struct Version {
    pub letter: &'static str,
    
    pub name: &'static str,
    
    pub compatibility: Option<&'static str>,
    pub author: &'static str,
}

pub static CURRENT_VERSION: Version = Version {
    letter: "v-pocket-R1",
    name: "Pocket Release 1.0.2",
    compatibility: None,
    author: "frgmt0",
};

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pocket CLI {}\n", CURRENT_VERSION.letter)?;
        write!(f, "Release: {}\n", CURRENT_VERSION.name)?;
        write!(f, "Author: {}", CURRENT_VERSION.author)?;

        if let Some(compat) = CURRENT_VERSION.compatibility {
            write!(f, "\nCompatibility: {}", compat)?;
        }

        Ok(())
    }
}