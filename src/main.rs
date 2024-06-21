use std::{option, thread};
use std::time::Duration;
use console::Term;
use dialoguer::{Confirm};
use rusqlite::{Connection,Result};

#[derive(Debug)]
struct User {
    id:i32,
    login:String,
    password:String,
}



fn main() {
    start();
}

fn start()
{
    let conn = Connection::open_in_memory().unwrap();
    let term = Term::stdout();
    term.write_line("Welcome to todo list console-utilities");
    term.clear_line();

    confirm(term,conn);
}

fn confirm(term:Term,conn:Connection) {
    let option = Confirm::new()
        .with_prompt("Login or Register")
        .interact_opt()
        .unwrap();

    match option {
        Some(true) => login(term,conn),
        Some(false) => register(term,conn),
        None => println!("No choice was made."),
    }
}

fn register(term:Term,conn:Connection)
{
    term.write_line("Create your username");

    let name = term.read_line().unwrap();
    


    let mut password:String = "".to_string();
    let mut password2:String = "a".to_string();

    while password != password2 {
    term.write_line("Enter your password");
     password = term.read_line().unwrap();
    term.write_line("Reenter password");
    password2 = term.read_line().unwrap();
    if password != password2
    {
        term.write_line("Passwords do not match");
    }
    }
}

fn login(term:Term,conn:Connection)
{
    term.write_line("Please enter your login");
    let name = term.read_line().unwrap();
    term.write_line("Enter your password");
    let password = term.read_line().unwrap();

    check_account(name, password, conn);
}

fn check_account(login:String,password:String,conn:Connection)
{
    let user = conn.execute(
        "SELECT * FROM users WHERE login = ?1 AND password = ?2", (login,password));



}