extern crate ledger;
extern crate postgres;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate uuid;
extern crate chrono;

use std::io::Write;
use ledger::*;

#[derive(Serialize)]
struct OutputError {
	success: bool,
	msg: &'static str,
}

#[derive(Serialize)]
struct OutputSuccessGet {
	money_form: i32,
	amount: i32,
	value: f64,
}

#[derive(Deserialize)]
struct Input {
	money_form: i32,
	amount: i32,
	value: f64,
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
		let date: chrono::NaiveDate = chrono::Utc::now().date().naive_utc();

		// get products for company
		let sql = "SELECT money_form, amount, value, horario FROM armored_car WHERE user_id = $1 AND data = date_trunc('day', CAST($2 AS DATE))";
		let rows = conn.query(sql, &[&user_id, &date]).unwrap();
		
		let mut output: Vec<OutputSuccessGet> = Vec::new();
		for row in rows.iter() {
			output.push(OutputSuccessGet {
				money_form: row.get(0),
				amount: row.get(1),
				value: row.get(2),
			});
		}
		
		// output json
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: Input = serde_json::from_reader(std::io::stdin()).unwrap();

		// get products for company
		let sql = "INSERT INTO armored_car(user_id, money_form, amount, value, data, horario) VALUES($1, $2, $3, $4, NOW(), NOW())";
		conn.execute(sql, &[&user_id, &input.money_form, &input.amount, &input.value]).unwrap();
		
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
