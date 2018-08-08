use std::io::Write;

pub const CONN_PARAMS: &'static str = "postgres://user:password@localhost:5432/boxxy";

pub fn get_env(key: &'static str) -> String {
	match std::env::var(key) {
		Ok(val) => val,
		Err(e) => {
			writeln!(std::io::stderr(), "Undefined environment variable {:?}: {}", key, e).unwrap();
			std::process::exit(1);
		},
	}
}
