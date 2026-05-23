use serde::Serialize;
use std::io::Write;

pub fn write_json<T: Serialize>(value: &T) {
    let json = serde_json::to_string_pretty(value).expect("write_json: serialization failed");
    println!("{json}");
}

pub fn write_binary(data: &[u8], path: Option<&str>) -> Result<(), std::io::Error> {
    if let Some(path) = path {
        std::fs::write(path, data)
    } else {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(data)
    }
}

pub fn print_success(msg: &str, quiet: bool) {
    if !quiet {
        println!("{msg}");
    }
}

pub fn print_error(msg: &str) {
    eprintln!("Error: {msg}");
}

pub fn print_verbose(msg: &str, verbose: bool) {
    if verbose {
        eprintln!("[verbose] {msg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_success_quiet() {
        print_success("hello", true);
    }

    #[test]
    fn test_print_success_not_quiet() {
        print_success("hello", false);
    }

    #[test]
    fn test_print_verbose_enabled() {
        print_verbose("request details", true);
    }

    #[test]
    fn test_print_verbose_disabled() {
        print_verbose("request details", false);
    }

    #[test]
    fn test_print_error_output() {
        print_error("something bad");
    }

    #[test]
    fn test_write_json_round_trip() {
        #[derive(Serialize)]
        struct Test {
            name: String,
            value: i32,
        }
        let t = Test {
            name: "foo".into(),
            value: 42,
        };
        write_json(&t);
    }

    #[test]
    fn test_write_binary_to_stdout() {
        let data = b"hello";
        write_binary(data, None).unwrap();
    }

    #[test]
    fn test_write_binary_to_file() {
        let data = b"binary data";
        let path = "/tmp/test_write_binary.bin";
        write_binary(data, Some(path)).unwrap();
        let contents = std::fs::read(path).unwrap();
        assert_eq!(contents, data);
        let _ = std::fs::remove_file(path);
    }
}
