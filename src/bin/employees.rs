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
	username: String,
	name: String,
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
	println!("");
	
	// generate session token
    let token: uuid::Uuid = uuid::Uuid::parse_str(&*token).unwrap();

    // connect to database
	let conn_params = CONN_PARAMS;
    let conn = postgres::Connection::connect(conn_params, postgres::TlsMode::None).unwrap();
    
    // get user information
    let sql = "SELECT user_id FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.token = $1 AND time + '10 hours' > NOW()";
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
    
    let row = rows.get(0);
    let user_id: i32 = row.get(0);
    
    if method == "GET" {
		// get products for company
		//let sql = "SELECT id, name, value FROM products WHERE company=$1";
		let sql = "SELECT username, name FROM users WHERE company=$1";
		let rows = conn.query(sql, &[&user_id]).unwrap();
		
		// output json
		let mut output: Vec<OutputSuccess> = Vec::with_capacity(rows.len());
		for row in rows.iter() {
			output.push(OutputSuccess {
				username: row.get(0),
				name: row.get(1),
			});
		}
		//let token = format!("{{{}}}", token);
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else {
		println!("Status: 404");
		println!("");
		println!("Page not found.");
		std::process::exit(1);
	}
	println!("");
}
