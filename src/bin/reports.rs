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
	report_id: i32,
	terminal_id: i32,
	value: f64,
}

#[derive(Deserialize)]
struct InputPost {
	terminal_id: i32,
	value: f64,
}

#[derive(Serialize)]
struct OutputSuccessPost {
	success: bool,
	report_id: i32,
}

#[derive(Serialize)]
struct OutputSuccessDelete {
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
		let sql = "SELECT id, terminal_id, value FROM reports WHERE user_id=$1 AND date=current_date";
		let rows = conn.query(sql, &[&user_id]).unwrap();
		
		// output json
		let mut output: Vec<OutputSuccessGet> = Vec::with_capacity(rows.len());
		for row in rows.iter() {
			output.push(OutputSuccessGet {
				success: true,
				report_id: row.get(0),
				terminal_id: row.get(1),
				value: row.get(2),
			});
		}
		//let token = format!("{{{}}}", token);
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: InputPost = serde_json::from_reader(std::io::stdin()).unwrap();
		
		let sql = "INSERT INTO reports(id, date, time, user_id, terminal_id, value) VALUES (DEFAULT, current_date, current_time, $1, $2, $3) RETURNING id";
		let rows = conn.query(sql, &[&user_id, &input.terminal_id, &input.value]).unwrap();
		
		// output json
		let output = OutputSuccessPost {
			success: true,
			report_id: rows.get(0).get(0),
		};
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "DELETE" {
		let path_info = get_env("PATH_INFO");
		let mut path = path_info.split('/');
		let report_id = path.nth(1).unwrap();
		let report_id: i32 = report_id.parse().unwrap();
		
		let sql = "DELETE FROM reports WHERE user_id=$1 AND id=$2 AND date=current_date";
		conn.execute(sql, &[&user_id, &report_id]).unwrap();
		
		// output json
		let output = OutputSuccessDelete {
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
