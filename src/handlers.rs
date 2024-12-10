use std::net::TcpStream;
use std::io::{Read, Write};
use postgres::{Client, NoTls};
use crate::models::User;
use crate::utils::{get_id, get_user_request_body};
use crate::constants::{OK_RESPONSE, NOT_FOUND, INTERNAL_SERVER_ERROR};

pub fn handle_client(mut stream: TcpStream, db_url: &str) {
    let mut buffer = [0; 1024];
    let mut request = String::new();
 
    match stream.read(&mut buffer) { 
        Ok(size) => { 
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /users") => handle_post_request(r, db_url),
                r if r.starts_with("GET /users/") => handle_get_request(r, db_url),
                r if r.starts_with("GET /users") => handle_get_all_request(r, db_url),
                r if r.starts_with("PUT /users") => handle_put_request(r, db_url),
                r if r.starts_with("DELETE /users/") => handle_delete_request(r, db_url),
                _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
            };

            stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

pub fn handle_post_request(request: &str, db_url: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(db_url, NoTls)) {
        (Ok(user), Ok(mut client)) => {
            client
                .execute(
                    "INSERT INTO users (name, email) VALUES ($1, $2)",
                    &[&user.name, &user.email],
                )
                .unwrap();
            (OK_RESPONSE.to_string(), "user created".to_string())
        }
        _ => (
            INTERNAL_SERVER_ERROR.to_string(),
            "Error ".to_string(),
        ),
    }
}

pub fn handle_get_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(db_url, NoTls)){
        (Ok(id), Ok(mut client)) => 
        match client.query_one("SELECT * FROM users WHERE id = $1", &[&id]) {
            Ok(row) => {
                let user = User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                };
                (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
            }
            _ => (NOT_FOUND.to_string(), "User not found".to_string()),
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

pub fn handle_get_all_request(_request: &str, db_url: &str) -> (String, String) {
    match Client::connect(db_url, NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();
            for row in client.query("SELECT * FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }
            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
} 

pub fn handle_put_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(),
    get_user_request_body(&request),
    Client::connect(db_url, NoTls)) { 
        (Ok(id), Ok(user), Ok(mut client)) => {
            client.execute(
                "UPDATE users SET name = $1, email = $2 WHERE id = $3",
                &[&user.name, &user.email, &id]
            ).unwrap();
            (OK_RESPONSE.to_string(), "User updated".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

pub fn handle_delete_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute(
                "DELETE FROM users WHERE id = $1",
                &[&id]
            ).unwrap();
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "User not found".to_string());
            }
            (OK_RESPONSE.to_string(), "User deleted".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}