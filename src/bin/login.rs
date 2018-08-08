extern crate ledger;
extern crate postgres;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate uuid;

use ledger::*;

use std::io::Write;

#[derive(Deserialize)]
struct Input {
	username: String,
	password: String,
}

#[derive(Serialize)]
struct OutputSuccess {
	success: bool,
	token: String,
}

fn main() {
	// get environment variables
	let method = get_env("REQUEST_METHOD");
	let ip = get_env("REMOTE_ADDR");
	
	if method != "POST" {
		println!("Status: 404");
		println!("");
		println!("Page not found.");
		std::process::exit(1);
	}
	
	// retrieve and verify input
	let input: Input = match serde_json::from_reader(std::io::stdin()) {
		Ok(x) => x,
		Err(x) => {
			println!("Status: 401");
			println!("");
			println!("Error parsing JSON input: {}", x);
			return;
		}
	};
	let username = input.username;
	let password = input.password;

	// get connection
	let conn_params = CONN_PARAMS;
    let conn = postgres::Connection::connect(conn_params, postgres::TlsMode::None).unwrap();
    
    // get user information
    let sql = "SELECT id FROM users WHERE username=$1 AND password=$2";
    let rows = conn.query(sql, &[&username, &password]).unwrap();
    if rows.len() != 1 {
		println!("Status: 401");
		println!("");
		println!("Incorrect username or password.");
		return;
    }
    
    // generate session token
    let token = uuid::Uuid::new_v4();
    
    let user_id: i32 = rows.get(0).get(0);
	let sql = "INSERT INTO sessions (user_id, ip, token) VALUES ($1, $2, $3)";
	conn.execute(sql, &[&user_id, &ip, &token]).unwrap();
	let token = token.hyphenated().to_string();
	
	// output json
	let output = OutputSuccess {
		success: true,
		token: token,
	};
	//let token = format!("{{{}}}", token);
	let output = serde_json::to_vec(&output).unwrap();

	println!("Content-Type: application/json");
	println!("");
	std::io::stdout().write(&output).unwrap();
	println!("");
}
