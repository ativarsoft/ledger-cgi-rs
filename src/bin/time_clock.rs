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
struct OutputSuccessGet {
	success: bool,
	timestamp: i64,
	now: i64,
	is_arriving: bool,
}

#[derive(Deserialize)]
struct Input {
	is_arriving: i32,
}

#[derive(Serialize)]
struct OutputSuccessPost {
	success: bool,
}

fn main() {
	// get environment variables
	let method = get_env("REQUEST_METHOD");
	let token = get_env("HTTP_X_AUTH_TOKEN");

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
		let sql = "SELECT extract(epoch from horario), extract(epoch from NOW()), is_arriving FROM time_clock WHERE user_id=$1 ORDER BY horario DESC";
		let rows = conn.query(sql, &[&user_id]).unwrap();
		
		if rows.len() != 1 {
			let output = OutputSuccessGet {
				success: true,
				timestamp: 0,
				now: 0,
				is_arriving: false,
			};
			let output: Vec<u8> = serde_json::to_vec(&output).unwrap();
			std::io::stdout().write(&output).unwrap();
			println!("");
			std::process::exit(1);
		}
		
		// output json
		let row = rows.get(0);
		let timestamp: f64 = row.get(0);
		let timestamp: i64 = timestamp as i64;
		let now: f64 = row.get(1);
		let now: i64 = now as i64;
		let is_arriving: i32 = row.get(2);
		let is_arriving: bool = is_arriving != 0;
		let output = OutputSuccessGet {
			success: true,
			timestamp: timestamp,
			now: now,
			is_arriving: is_arriving,
		};
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: Input = serde_json::from_reader(std::io::stdin()).unwrap();
		let is_arriving = input.is_arriving;

		let sql = "INSERT INTO time_clock(user_id, horario, is_arriving) VALUES($1, NOW(), $2)";
		conn.execute(sql, &[&user_id, &is_arriving]).unwrap();
		
		// output json
		let output = OutputSuccessPost {
			success: true,
		};
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
