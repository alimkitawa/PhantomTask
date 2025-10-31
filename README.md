# PhantomTask

A Windows command-line utility for creating and executing scheduled tasks with session-specific control. PhantomTask leverages the Windows Task Scheduler API to create tasks that run in specific user sessions with elevated privileges.

## Features

- **Session Management**: Create tasks targeting specific Windows Terminal Services sessions
- **Session Enumeration**: List all active sessions on the local machine with detailed information
- **Flexible Authentication**: Support for both interactive token and password-based authentication
- **Elevated Privileges**: Tasks can be configured to run with highest privileges
- **Immediate Execution**: Automatically triggers tasks after creation with configurable session targeting
- **Hidden Tasks**: Tasks are created as hidden by default

## Requirements

- Windows operating system
- Rust toolchain (2024 edition)
- Administrator privileges (recommended for full functionality)

## Installation

Clone the repository and build using Cargo:

```powershell
git clone <repository-url>
cd phantomtask
cargo build --release
```

The compiled binary will be available at `target/release/phantomtask.exe`

## Usage

### List Active Sessions

Display all active Windows Terminal Services sessions with their details:

```powershell
phantomtask.exe --list
# or
phantomtask.exe -l
```

Output example:

```
===== Active Sessions =====

SessionID  User                 State           Station              Domain              
=====================================================================================
0          <None>               Active          Services             <Local>            
1          Administrator        Active          Console              WORKSTATION        
```

### Create and Execute a Task

**If you want to execute a task as `SYSTEM` (session 0), change the username in `get_user_from_session()`!** It is localisation dependent (Système, System, ...), and there is no automatic resolution on this account.

Create a scheduled task that runs in a specific session:

```powershell
phantomtask.exe --name "MyTask" --program "C:\path\to\program.exe" --sessionid 1
```

#### Required Arguments

- `-n, --name <TASKNAME>`: Name of the task to create
- `-f, --program <PROGRAM>`: The program to execute
- `-s, --sessionid <SESSIONID>`: Session ID where the task should run

#### Optional Arguments

- `-a, --arguments <ARGUMENTS>`: Arguments to pass to the program
- `-u, --username <USERNAME>`: Username to run the task as
- `-p, --password <PASSWORD>`: Password for the specified username

### Examples

#### Basic Task Creation

```powershell
phantomtask.exe -n "NotepadTask" -f "notepad.exe" -s 1
```

#### Task with Arguments

```powershell
phantomtask.exe -n "CmdTask" -f "cmd.exe" -a "/c dir" -s 1
```

#### Task with Specific User Credentials

```powershell
phantomtask.exe -n "UserTask" -f "C:\Tools\app.exe" -u "DOMAIN\User" -p "Password123" -s 1
```

## How It Works

1. **COM Initialization**: Initializes the Component Object Model (COM) library with multithreaded apartment
2. **Task Scheduler Connection**: Connects to the Windows Task Scheduler service
3. **Task Definition**: Creates a new task definition with:
   - Time trigger (scheduled to run 1 minute after creation)
   - Execution action with specified program and arguments
   - Principal configuration (user and logon type)
   - Security settings (run with highest privileges)
4. **Session Resolution**: Resolves the username associated with the target session ID
5. **Task Registration**: Registers the task in the root folder of Task Scheduler
6. **Immediate Execution**: Triggers the task immediately using the resolved session context

## Project Structure

```
phantomtask/
├── Cargo.toml          # Project dependencies and configuration
├── src/
│   ├── main.rs         # Entry point and CLI argument parsing
│   ├── tasks.rs        # Task creation and registration logic
│   ├── sessions.rs     # Session enumeration and user resolution
│   └── utils.rs        # Utility functions (wide string conversion)
└── README.md
```

## Dependencies

- **windows**: Windows API bindings for Rust (v0.62.2)
  - Task Scheduler COM interfaces
  - Remote Desktop Services API
  - COM and OLE support
- **windows-core**: Core Windows types (v0.62.2)
- **clap**: Command-line argument parser (v4.5.51)

## Limitations

- Windows-only (uses Windows-specific APIs)
- Requires administrator rights for most operations
- Session ID 0 defaults to "Système" user (in french, localization-dependent)

## Error Handling

The application provides detailed console output for:
- COM initialization status
- Task creation progress
- Session user resolution
- Task execution confirmation
- Error messages with context

## Disclaimers

This is an obvious disclaimer because I don't want to be held responsible if someone uses this tool against anyone who hasn't asked for anything.

Usage of anything presented in this repo to attack targets without prior mutual consent is illegal. It's the end user's responsibility to obey all applicable local, state and federal laws. Developers assume no liability and are not responsible for any misuse or damage caused by this program. Only use for educational purposes.