use bincode::config::{self, Configuration};

pub const SERIALIZATION_CONFIG: Configuration = config::standard();
pub const CONCURRENT_FILE_IO_COUNT: usize = 16;
