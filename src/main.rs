mod sessions;
mod tasks;
mod utils;

use sessions::list_active_sessions;
use tasks::create_task;

use clap::{Arg, Command};
use windows::Win32::System::Com::CoUninitialize;
use windows::Win32::System::RemoteDesktop::WTSGetActiveConsoleSessionId;

fn main() -> windows_core::Result<()> {
    let args = Command::new("PhantomTask")
        .author("BlackWasp")
        .version("0.1.0")
        .arg(
            Arg::new("list-sessions")
                .short('l')
                .long("list")
                .required(false)
                .exclusive(true)
                .action(clap::ArgAction::SetTrue)
                .help("List active sessions on the local machine"),
        )
        .arg(
            Arg::new("taskname")
                .short('n')
                .long("name")
                .required_unless_present("list-sessions")
                .help("The name of the task to create"),
        )
        .arg(
            Arg::new("program")
                .short('f')
                .long("program")
                .required_unless_present("list-sessions")
                .help("The program to execute"),
        )
        .arg(
            Arg::new("arguments")
                .short('a')
                .long("arguments")
                .help("The arguments to pass to the program"),
        )
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("The username to run the task as"),
        )
        .arg(
            Arg::new("password")
                .short('p')
                .long("password")
                .help("The password for the specified username"),
        )
        .arg(
            Arg::new("sessionid")
                .short('s')
                .long("sessionid")
                .required_unless_present("list-sessions")
                .help("The session ID to run the task in"),
        )
        .get_matches();

    if args.get_flag("list-sessions") {
        list_active_sessions()?;
        return Ok(());
    } else {
        let task_name = args.get_one::<String>("taskname").unwrap();
        let task_path = args.get_one::<String>("program").unwrap();
        let arguments = args.get_one::<String>("arguments").map(|s| s.as_str());
        let user_name = args.get_one::<String>("username").map(|s| s.as_str());
        let password = args.get_one::<String>("password").map(|s| s.as_str());
        let session_id = args
            .get_one::<String>("sessionid")
            .and_then(|s| s.parse().ok())
            .unwrap_or(unsafe { WTSGetActiveConsoleSessionId() });

        println!("Configuration:");
        println!("  TaskName: {}", task_name);
        println!("  Program: {}", task_path);
        if let Some(ref args) = arguments {
            println!("  Arguments: {}", args);
        }
        println!("  Session ID: {}", session_id);
        if let Some(ref user) = user_name {
            println!("  User: {}", user);
        }
        println!();

        match create_task(
            task_name, task_path, arguments, user_name, password, session_id,
        ) {
            Ok(_) => {
                println!("\n[+] Task successfully created and executed!");
            }
            Err(e) => {
                eprintln!("\n[!] Error during task creation or execution: {:?}", e);
            }
        }
    }

    unsafe {
        CoUninitialize();
        println!("[i] COM library uninitialized");
    }

    Ok(())
}
