#![allow(unused_macros)]
#![allow(unused_imports)]

macro_rules! open_file {
    ($str_arg:expr) => {{
        let mut path = PathBuf::from(file!());
        path.pop();
        path.pop();
        path.push("data/");
        path.push($str_arg);
        File::open(path).expect("Benchmark setup: unable to open file")
    }};
}
pub(crate) use open_file;

macro_rules! read_file {
    ($str_arg:expr) => {{
        let mut file = open_file!($str_arg);
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Benchmark setup: unable to read file");
        buffer
    }};
}
pub(crate) use read_file;
