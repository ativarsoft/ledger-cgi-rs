extern crate ledger;
extern crate postgres;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate uuid;

use std::io::Write;
use ledger::*;

#[derive(Serialize)]
struct OutputError {
	success: bool,
	msg: &'static str,
}

#[derive(Serialize)]
struct OutputSuccess {
	success: bool,
	name: String,
	photo: String,
	flags: i32,
}

fn main() {
	// get environment variables
	let method = get_env("REQUEST_METHOD");
	let token = get_env("HTTP_X_AUTH_TOKEN");
	
	if method != "GET" {
		println!("Status: 404");
		println!("");
		println!("Page not found.");
		std::process::exit(1);
	}

	println!("Content-Type: application/json");
	println!("Access-Control-Allow-Origin: *");
	println!("");
	
	// generate session token
    let token: uuid::Uuid = uuid::Uuid::parse_str(&*token).unwrap();

    // get connection
	let conn_params = CONN_PARAMS;
    let conn = postgres::Connection::connect(conn_params, postgres::TlsMode::None).unwrap();
    
    // get user information
    let sql = "SELECT username, name, flags FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.token = $1 AND time + '10 hours' > NOW()";
    let rows = conn.query(sql, &[&token]).unwrap();
    if rows.len() != 1 {
		let output = OutputError {
			success: false,
			msg: "Expired session. Please login again.",
		};
		let output: Vec<u8> = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
		std::process::exit(1);
    }
	
	// output json
	let row = rows.get(0);
	let username: String = row.get(0);
	let output = OutputSuccess {
		success: true,
		name: row.get(1),
		photo: format!("photos/{}.jpg", username),
		flags: row.get(2)
	};
	//let token = format!("{{{}}}", token);
	let output = serde_json::to_vec(&output).unwrap();
	std::io::stdout().write(&output).unwrap();
	println!("");
}
