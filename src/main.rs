use std::io::Write;
use std::process::exit;
use console::Term;
use dialoguer::{Confirm, Input};
use rusqlite::{params, Connection, Result,Error};
use serde_json;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, DEFAULT_COST};

#[derive(Debug,Clone)]
struct User {
    id:i32,
    login:String,
    password:String
}



#[derive(Debug, Serialize, Deserialize,Clone)]
struct TodoList {
    name: String,
    done: bool,
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
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

    let _ = initialize_database(&conn);
    let _ = term.write_line("Welcome to todo list console-utilities");
    let _ = term.clear_line();

    confirm(term,conn);
}

fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            login TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            userid INTEGER NOT NULL,
            name TEXT NOT NULL,
            done BOOLEAN NOT NULL,
            tasks TEXT NOT NULL,
            FOREIGN KEY(userid) REFERENCES users(id)
        )",
        [],
    )?;

    Ok(())
}

fn confirm(term:Term,conn:Connection) {
    let mut option:Option<bool> = None;

    while  option == None {
        option = Confirm::new()
        .with_prompt("Login or Register")
        .interact_opt()
        .unwrap();
    match option {
        Some(true) => login(&term,&conn),
        Some(false) => register(&term,&conn),
        None => println!("No choice was made."),
    }   
    }
}

fn register(term:&Term,conn:&Connection)
{

    
    let mut name:String;

    loop {
        let _ = term.write_line("Create your username");
        name = term.read_line().unwrap();

        if !if_login_already_exists(&name, conn).unwrap() {
            break;
        }

        let _ = term.write_line("This username already taken");
    }
    


    let mut password:String = "".to_string();
    let mut password2:String = "a".to_string();

    while password != password2 {
    let _ = term.write_line("Enter your password");
     password = term.read_line().unwrap();
    let _ = term.write_line("Reenter password");
    password2 = term.read_line().unwrap();
    if password != password2
    {
        let _ = term.write_line("Passwords do not match");
    }
    }

    let user = create_account(name, password, &conn);

    match user {
        Some(user) => {
            let _ = term.write_line("created user");
                show_menu(&user, &term, &conn);   
        },
        None => {
            let _ = term.write_line("error occured during creation of a user");
        },
    }




}
fn create_account(login: String, password: String, conn: &Connection) -> Option<User> {

    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    let _ = conn.execute("INSERT INTO users (login, password) VALUES (?1, ?2)", params![login, hashed_password]);

    check_account(login, hashed_password, conn)
    
}

fn login(term:&Term,conn:&Connection)
{
    let _ = term.write_line("Please enter your login");
    let name = term.read_line().unwrap();
    let _ = term.write_line("Enter your password");
    let password = term.read_line().unwrap();

    let hashed_password = match hash(password, DEFAULT_COST) {
        Ok(hashed) => hashed,
        Err(_) => {
            let _ = term.write_line("Error hashing password");
            return;
        }
    };

    let user = check_account(name, hashed_password, &conn);

    match user {
        Some(user) => {
            let _ = term.write_line("loggined");

            show_menu(&user, &term, &conn);
        },
        None => {
            let _ = term.write_line("wrong login or password");
            login(term, conn);
        }
    }
    
}

fn create_new_list(user: &User,mut term: &Term,conn: &Connection) -> Result<()>
{
    let _ = term.write_all(b"Think for a name for todolist:\n");
    let new_name: String = Input::new().interact_text().unwrap();
    let tasks:Vec<Task> = Vec::new();
    let tasks_json = serde_json::to_string(&tasks).unwrap(); 
    conn.execute("INSERT INTO todos VALUES(name,done,tasks,userid) ?1,?2,?3", params![new_name,false,tasks_json,user.id])?;
    Ok(())
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
    let _ = term.write_line("1) Create new todolist");
    let _ = term.write_line("2) Select todolist");
    let _ = term.write_line("3) Update login");
    let _ = term.write_line("4) Update password");        
    let _ = term.write_line("5) Delete User");
    let _ = term.write_line("6) Exit");
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
    let mut stmt = conn.prepare("SELECT name, done, tasks FROM todos WHERE userid = ?1")?;
    
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


fn show_menu(user:&User,term: &Term,conn: &Connection)
{

    
    let _ = term.write_line(&format!("Welcome to the system {}",user.login));
    let todos = load_lists(user.id, &conn);

            match todos {
                Ok(todos) => {
                    let _ = term.write_line("Choose option");
                    print_optiions(&term);
                    
                    let mut option: i32;
                    loop {
                        option = Input::new().interact_text().unwrap();
                        
                        if (1..=6).contains(&option) {
                            break;
                        } else {
                            let _ = term.write_line( "Invalid option. Please enter a number between 1 and 6.");
                        }
                    }

                    match option {
                        1 => {
                            let _ = create_new_list(&user,term,conn);
                        }
                        2 => {
                            if todos.len() == 0 {
                                let _ = term.write_line( "There is no todolists to select");
                            }

                            let _ = show_todolists(&todos, &term);
                            let _ = select_todo_list(&user, term, conn, todos);


                        }
                        3 => {
                            change_login(user, term, conn).unwrap();
                        }
                        4 => {
                            update_password(user, term, conn).unwrap();
                        } 
                        5 => {
                            delete_user(user, term, conn).unwrap();
                        }
                        6 => {
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


fn select_todo_list(user:&User,mut term: &Term,conn: &Connection,todos:Vec<TodoList>)  -> Result<()>
{

    let mut option: i32;

    loop {
        option = Input::new().interact_text().unwrap_or(-1);

        if (1..=todos.len() as i32).contains(&option) {
            break;
        } else {
            let _ = term.write_all(format!("Invalid option. Please enter a number between 1 and {}\n", todos.len()).as_bytes());
        }
    }

    let todo = &todos[(option - 1) as usize];
    let _ = load_todo(todo, &mut term);

    let _ = todolist_options(&mut term);

    let mut option_list: i32;

    loop {
        option_list = Input::new().interact_text().unwrap_or(-1);

        if (1..=5).contains(&option_list) {
            break;
        } else {
            let _ = term.write_all(b"Invalid option. Please enter a number between 1 and 5\n");
        }
    }

    match option_list {
        1 => {
            load_todo(todo, &mut term).unwrap();
            
            let _ = select_task(user, &mut term, conn, &mut todo.clone());
        
        },
        2 => edit_name(&mut term, user, todo, conn).unwrap(),
        3 => edit_status(&mut term, user, todo, conn).unwrap(),
        4 => add_new_task(user,&mut todo.clone(),term,conn).unwrap(),
        5 => delete_todolist(&mut term, user, todo, conn).unwrap(),
        _ => unreachable!(),
    };

    Ok(())

}

fn add_new_task(user: &User,todo: &mut TodoList,mut term: &Term,conn: &Connection) -> Result<()>
{

    let _ = term.write_all(b"Think for a name for task:\n");
    let new_name: String = Input::new().interact_text().unwrap();

    let task:Task = Task {
        name: new_name,
        done: false,
    };

    todo.tasks.push(task);

    let tasks_json = serde_json::to_string(&todo.tasks).unwrap();
    conn.execute("UPDATE todos SET tasks = ?1 WHERE userid = ?2 AND name = ?3", params![tasks_json, user.id, todo.name])?;
    let _ = term.write_all(b"Added task\n");
    Ok(())
}


fn load_todo(todo: &TodoList, term: &mut impl std::io::Write) -> Result<()> {
    let _ = term.write_all(format!("{}\n", todo.name).as_bytes());

    let status = if todo.done { "Finished" } else { "In Progress" };
    let _ = term.write_all(format!("status: {}\n", status).as_bytes());

    for (i, task) in todo.tasks.iter().enumerate() {
        let task_status = if task.done { "✅" } else { "❌" };
        let _ = term.write_all(format!("[{}] {} [{}]\n", i + 1, task.name, task_status).as_bytes());
    }

    Ok(())
}

fn options_for_tasks(term: &mut impl std::io::Write) -> Result<()> {
    let _ = term.write_all(b"1) Edit name\n");
    let _ = term.write_all(b"2) Edit status\n");
    let _ = term.write_all(b"3) Delete\n");
    Ok(())
}

fn delete_todolist(term: &mut impl std::io::Write, user: &User, todo: &TodoList, conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE userid = ?1 AND name = ?2", params![user.id, todo.name])?;
    let _ = term.write_all(b"Todo list deleted successfully.\n");
    Ok(())
}

fn edit_name(term: &mut impl std::io::Write, user: &User, todo: &TodoList, conn: &Connection) -> Result<()> {
    let _ = term.write_all(b"Type new name:\n");
    let new_name: String = Input::new().interact_text().unwrap();
    conn.execute("UPDATE todos SET name = ?1 WHERE userid = ?2 AND name = ?3", params![new_name, user.id, todo.name])?;
    let _ = term.write_all(b"Todo list name updated successfully.\n");
    Ok(())
}

fn edit_status(term: &mut impl std::io::Write, user: &User, todo: &TodoList, conn: &Connection) -> Result<()> {
    let res = Confirm::new().with_prompt("Do you want to change status?").interact().unwrap();
    
    if res {
        let new_status = !todo.done;
        conn.execute("UPDATE todos SET done = ?1 WHERE userid = ?2 AND name = ?3", params![new_status, user.id, todo.name])?;
        let _ = term.write_all(b"Todo list status updated successfully.\n");
    }

    Ok(())
}

fn todolist_options(term: &mut impl std::io::Write) -> Result<()> {
    let _ = term.write_all(b"1) Show tasks\n");
    let _ = term.write_all(b"2) Edit name\n");
    let _ = term.write_all(b"3) Edit status\n");
    let _ = term.write_all(b"4) Add new task\n");
    let _ = term.write_all(b"5) Delete\n");
    Ok(())
}

fn select_task(user: &User, term: &mut impl std::io::Write, conn: &Connection, todo: &mut TodoList) -> Result<()> {   
    let _ = term.write_all(b"Select task\n");

    let mut option: i32;

    loop {
        option = Input::new().interact_text().unwrap_or(-1);

        if (1..=todo.tasks.len() as i32).contains(&option) {
            break;
        } else {
            let _ = term.write_all(format!("Invalid option. Please enter a number between 1 and {}\n", todo.tasks.len()).as_bytes());
        }
    }
    options_for_tasks(term)?;

    let mut option_task: i32;

    loop {
        option_task = Input::new().interact_text().unwrap_or(-1);

        if (1..=3).contains(&option_task) {
            break;
        } else {
            let _ = term.write_all(b"Invalid option. Please enter a number between 1 and 3\n");
        }
    }

    let _ = match option_task {
        1 => edit_task_name(user, todo, (option-1) as usize, term, conn),
        2 => edit_task_status(user, todo, (option-1) as usize, term, conn),
        3 => delete_task(user, todo, (option-1) as usize, term, conn),
        _ => unreachable!(),
    };

    Ok(())
}

fn edit_task_name(user: &User, todo: &mut TodoList, index: usize, term: &mut impl std::io::Write, conn: &Connection) -> Result<()> {
    let _ = term.write_all(b"Type new name:\n");
    let new_name: String = Input::new().interact_text().unwrap();
    todo.tasks[index].name = new_name;

    let tasks_json = serde_json::to_string(&todo.tasks).unwrap();
    conn.execute("UPDATE todos SET tasks = ?1 WHERE userid = ?2 AND name = ?3", params![tasks_json, user.id, todo.name])?;
    let _ = term.write_all(b"Task name updated successfully.\n");
    Ok(())
}

fn edit_task_status(user: &User, todo: &mut TodoList, index: usize, term: &mut impl std::io::Write, conn: &Connection) -> Result<()> {
    let res = Confirm::new().with_prompt("Do you want to change status?").interact().unwrap();
    
    if res {
        todo.tasks[index].done = !todo.tasks[index].done;

        let tasks_json = serde_json::to_string(&todo.tasks).unwrap();
        conn.execute("UPDATE todos SET tasks = ?1 WHERE userid = ?2 AND name = ?3", params![tasks_json, user.id, todo.name])?;
        let _ = term.write_all(b"Task status updated successfully.\n");
    }

    Ok(())
}

fn delete_task(user: &User, todo: &mut TodoList, index: usize, term: &mut impl std::io::Write, conn: &Connection) -> Result<()> {
    todo.tasks.remove(index);
    
    let tasks_json = serde_json::to_string(&todo.tasks).unwrap();
    conn.execute("UPDATE todos SET tasks = ?1 WHERE userid = ?2 AND name = ?3", params![tasks_json, user.id, todo.name])?;
    let _ = term.write_all(b"Task deleted successfully.\n");
    Ok(())
}

fn delete_user(user: &User, term: &Term, conn: &Connection) -> Result<()> {
    
    conn.execute("DELETE FROM users WHERE id = ?1", params![user.id])?;

    
    term.write_line("Deleted user").unwrap();

    Ok(())
}

fn change_login(user: &User, term: &Term, conn: &Connection) -> Result<()> {
    loop {
        term.write_line("Enter new login").unwrap();
        let new_login: String = Input::new().interact_text().unwrap();

        if !if_login_already_exists(&new_login, conn)? {
            // Username does not exist, proceed with update
            conn.execute("UPDATE users SET login = ?1 WHERE id = ?2", params![new_login, user.id])?;
            term.write_line("Changed username").unwrap();
            break;
        } else {
            term.write_line("This username is already taken").unwrap();
        }
    }

    Ok(())
}

fn if_login_already_exists(login: &str, conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE login = ?1")?;

    let count: i32 = stmt.query_row(params![login], |row| {
        row.get(0)
    })?;

    Ok(count > 0)
}

fn update_password(user: &User, term: &Term, conn: &Connection) -> Result<()> {
    let mut password: String;
    let mut password2: String;

    loop {
        let _ = term.write_line("Enter your new password");
        password = term.read_line().unwrap();

        let _ = term.write_line("Reenter your new password");
        password2 = term.read_line().unwrap();

        if password != password2 {
            let _ = term.write_line("Passwords do not match");
        } else if password == user.password {
            let _ = term.write_line("You already use this password");
        } else {
            let hashed_password = hash(password, DEFAULT_COST).unwrap();
            conn.execute("UPDATE users SET password = ?1 WHERE id = ?2", params![hashed_password, user.id])?;
            let _ = term.write_line("Password updated successfully");
            break;
        }
    }

    Ok(())
}