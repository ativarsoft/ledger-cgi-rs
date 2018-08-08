extern crate ledger;

use ledger::*;

fn main() {
	let request_uri = get_env("REQUEST_URI");
	let query_string = get_env("QUERY_STRING");
	let script_name = get_env("SCRIPT_NAME");
	let document_root = get_env("DOCUMENT_ROOT");
	let path_translated = get_env("PATH_TRANSLATED");
	let path_info = get_env("PATH_INFO");
	println!("Content-Type: text/plain");
	println!("");
	println!("REQUEST_URI: {}", request_uri);
	println!("QUERY_STRING: {}", query_string);
	println!("SCRIPT_NAME: {}", script_name);
	println!("DOCUMENT_ROOT: {}", document_root);
	println!("PATH_TRANSLATED: {}", path_translated);
	println!("PATH_INFO: {}", path_info); // This is important for REST. This is the part after the CGI script.
	//println!("next path: {}", path_info.split('/').nth(1).unwrap());
}
