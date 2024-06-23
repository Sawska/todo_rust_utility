use std::process::exit;
use std::{option, thread};
use std::time::Duration;
use console::Term;
use dialoguer::{Confirm, Input};
use rusqlite::{params, Connection, Result,Error};
use serde_json;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
struct User {
    id:i32,
    login:String,
    password:String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoList {
    name: String,
    done: bool,
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
    done: bool,
}



fn main() {
    start();
}

fn start()
{
    let conn =  Connection::open("todos.db").unwrap();
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

    let user = create_account(name, password, &conn);

    match user {
        Some(user) => {
            term.write_line("created user");

            show_menu(user, &term, &conn);

        },
        None => {
            term.write_line("error occured during creating user");
        },
    }




}
fn create_account(login: String, password: String, conn: &Connection) -> Option<User> {
    conn.execute("INSERT INTO users (login, password) VALUES (?1, ?2)", params![login, password]);

    check_account(login, password, conn)
    
}

fn login(term:Term,conn:Connection)
{
    term.write_line("Please enter your login");
    let name = term.read_line().unwrap();
    term.write_line("Enter your password");
    let password = term.read_line().unwrap();

    let user = check_account(name, password, &conn);

    match user {
        Some(user) => {
            term.write_line("loggined");

            show_menu(user, &term, &conn);
        },
        None => {
            term.write_line("wrong login or password");
            login(term, conn);
        }
    }
    
}

fn create_new_list()
{

}

fn show_todolists(todos: &Vec<TodoList>, term: &Term) -> Result<(), std::io::Error> {
    term.write_line("Your todos:")?;
    for (i, todo) in todos.iter().enumerate() {
        let status = if todo.done {
            "Finished".to_owned()
        } else {
            "In process".to_owned()
        };

        term.write_line(&format!("[{}] {} status: {}", i + 1, todo.name, status))?;
    }
    Ok(())
}

fn print_optiions(term: &Term)
{
    term.write_line("1) Create new todolist");
    term.write_line("2) Select todolist");
    term.write_line("3) Exit");
}



fn check_account(login: String, password: String, conn: &Connection) -> Option<User> {
    let mut stmt = match conn.prepare("SELECT id, login, password FROM users WHERE login = ?1 AND password = ?2") {
        Ok(stmt) => stmt,
        Err(_) => return None,
    };

    let user_result: Result<User, _> = stmt.query_row([login, password], |row| {
        Ok(User {
            id: row.get(0)?,
            login: row.get(1)?,
            password: row.get(2)?,
        })
    });

    match user_result {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}

fn load_lists(id: i32, conn: &Connection) -> Result<Vec<TodoList>, Error> {
    let mut stmt = conn.prepare("SELECT name, done, tasks FROM todos WHERE id = ?1")?;
    
    let todo_res = stmt.query_map(params![id], |row| {
        let tasks_json: String = row.get(2)?;
        let tasks: Vec<Task> = serde_json::from_str(&tasks_json).unwrap_or_else(|_| Vec::new());

        Ok(TodoList {
            name: row.get(0)?,
            done: row.get(1)?,
            tasks,
        })
    })?;

    let mut todos = Vec::new();

    for todo in todo_res {
        todos.push(todo?);
    }

    Ok(todos)
}


fn show_menu(user:User,term: &Term,conn: &Connection)
{
    let todos = load_lists(user.id, &conn);

            match todos {
                Ok(todos) => {
                    term.write_line("Choose option");
                    print_optiions(&term);
                    
                    let mut option: i32;
                    loop {
                        option = Input::new().interact_text().unwrap();
                        
                        if (1..=3).contains(&option) {
                            break;
                        } else {
                            term.write_line( "Invalid option. Please enter a number between 1 and 3.");
                        }
                    }

                    match option {
                        1 => {
                            create_new_list();
                        }
                        2 => {
                            if todos.len() == 0 {
                                term.write_line( "There is no todolists to select");
                            }

                            show_todolists(&todos, &term);
                            select_todo_list(&user, term, conn, todos);

                        }
                        3 => {
                            exit(0);
                        }
                        _ => unreachable!(),
                    }

                },
                Err(err) => {
                    println!("error occured  {}",err);
                },
            }
}


fn select_todo_list(user:&User,term: &Term,conn: &Connection,todos:Vec<TodoList>)
{

    term.write_line("Seelect todolist");
    let mut option:i32;

    loop {
        option = Input::new().interact_text().unwrap();

        if (1..todos.len() as i32).contains(&option) {
            break;
        }    else {
            
            term.write_line(&format!("Invalid option. Please enter a number between 1 and {}",todos.len()));
        }
    }
}

fn todolist_options(term:&Term)
{
    term.write_line("1)Show tasks");
    term.write_line("2)Delete");
}