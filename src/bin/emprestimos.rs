/*
NOTES
Emprestimos nao tem delete. Um emprestimo oferecito deve ser pego de volta por quem ofereceu.
Metodos:
POST: create emprestimo
GET: read emprestimos
PUT: update emprestimos
DELETE: N/A
*/

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
	id: i32,
	date: String,
	money_form: i32,
	value: f64,
	from: String,
	to: String,
	is_taken: bool,
}

#[derive(Deserialize)]
struct InputPost {
	money_form: i32,
	value: f64,
	to: String,
}

#[derive(Deserialize)]
struct InputPut {
	emprestimo_id: i32,
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
		let path_info = get_env("PATH_INFO");
		let mut path = path_info.split('/');
		let request_set = path.nth(1).unwrap();
		let request_user = path.next().unwrap();
		let request_date_start = path.next().unwrap();
		let request_date_end = path.next().unwrap();

		/*let date: chrono::NaiveDate = chrono::Utc::now().date().naive_utc();
		let sql = "SELECT emprestimos.id, money_form, value, users1.username, users2.username, is_taken FROM emprestimos INNER JOIN users AS users1 ON from_user_id = users1.id INNER JOIN users AS users2 ON to_user_id = users2.id WHERE date=$1 AND (to_user_id=$2 OR to_user_id=-1 OR from_user_id=$2)";
		let rows = conn.query(sql, &[&date, &user_id]).unwrap();*/

		let rows = if request_set == "user" {
			let sql = "SELECT emprestimos.id, to_char(date, 'YYYY-MM-DD'), money_form, value, users1.username, users2.username, is_taken FROM emprestimos INNER JOIN users AS users1 ON from_user_id = users1.id INNER JOIN users AS users2 ON to_user_id = users2.id WHERE date >= to_date($2, 'YYYY-MM-DD') AND date <= to_date($3, 'YYYY-MM-DD') AND (users2.username=$1 OR to_user_id=-1 OR users1.username=$1)";
			conn.query(sql, &[&request_user, &request_date_start, &request_date_end]).unwrap()
		} else {
			return;
		};
		
		let mut output: Vec<OutputSuccessGet> = Vec::new();
		for row in rows.iter() {
			let is_taken: i32 = row.get(6);
			output.push(OutputSuccessGet {
				id: row.get(0),
				date: row.get(1),
				money_form: row.get(2),
				value: row.get(3),
				from: row.get(4),
				to: row.get(5),
				is_taken: is_taken != 0,
			});
		}
		
		// output json
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: InputPost = serde_json::from_reader(std::io::stdin()).unwrap();
		
		let date: chrono::NaiveDate = chrono::Utc::now().date().naive_utc();

		// get products for company
		let sql = "INSERT INTO emprestimos(date, money_form, value, from_user_id, to_user_id, is_taken) VALUES($1, $2, $3, $4, (SELECT users.id FROM users WHERE users.username = $5), 0)";
		conn.execute(sql, &[&date, &input.money_form, &input.value, &user_id, &input.to]).unwrap();
		
		// output json
		let output = OutputSuccessPost {
			success: true,
		};
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "PUT" {
		// retrieve and verify input
		let input: InputPut = serde_json::from_reader(std::io::stdin()).unwrap();

		// get products for company
		let sql = "UPDATE emprestimos SET is_taken=1, to_user_id=$2 WHERE id=$1 AND (to_user_id = $2 OR to_user_id=-1) AND is_taken=0";
		conn.execute(sql, &[&input.emprestimo_id, &user_id]).unwrap();
		
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
