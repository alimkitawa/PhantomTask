use crate::sessions::get_user_from_session;

use windows::core::{Interface, HRESULT};
use windows::Win32::Foundation::{S_OK, VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::TaskScheduler::{
    IExecAction, IPrincipal, IRegisteredTask, IRegistrationInfo, ITaskDefinition, ITaskFolder,
    ITaskService, ITimeTrigger, TaskScheduler, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE,
    TASK_INSTANCES_IGNORE_NEW, TASK_LOGON_INTERACTIVE_TOKEN, TASK_LOGON_PASSWORD,
    TASK_RUNLEVEL_HIGHEST, TASK_RUN_NO_FLAGS, TASK_RUN_USE_SESSION_ID, TASK_TRIGGER_TIME,
};
use windows::Win32::System::Variant::VARIANT;
use windows_core::BSTR;

pub fn create_task(
    task_name: &str,
    task_path: &str,
    arguments: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    session_id: u32,
) -> windows_core::Result<()> {
    // Task creation logic goes here

    unsafe {
        let h_result: HRESULT;

        // Initialize COM library
        h_result = CoInitializeEx(None, COINIT_MULTITHREADED);
        if h_result != S_OK {
            panic!("Failed to initialize COM library: HRESULT = 0x{}", h_result);
        }

        // Initialize COM security
        println!("[i] Initializing COM security");
        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        )?;

        // Create Task Scheduler instance
        println!("[i] Creating Task Scheduler instance");
        let task_scheduler: ITaskService =
            CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;

        // Connection to the Task Scheduler service
        println!("[i] Connecting to Task Scheduler service");
        task_scheduler.Connect(
            &VARIANT::default(),
            &VARIANT::default(),
            &VARIANT::default(),
            &VARIANT::default(),
        )?;

        // Get the root folder
        const ROOT_FOLDER: &str = "\\";
        let root_folder: ITaskFolder = task_scheduler.GetFolder(&BSTR::from(ROOT_FOLDER))?;

        // New task definition and registration info
        println!("[i] Creating new task definition");
        let task: ITaskDefinition = task_scheduler.NewTask(0)?;
        let registration_info: IRegistrationInfo = task.RegistrationInfo()?;
        registration_info.SetAuthor(&BSTR::from("BlackWasp"))?;
        registration_info.SetDescription(&BSTR::from("Yennefer better than Triss"))?;

        // Set principal for the task
        let principal: IPrincipal = task.Principal()?;
        if let (Some(user), Some(pass)) = (username, password) {
            if !user.is_empty() && !pass.is_empty() {
                principal.SetUserId(&BSTR::from(user))?;
                principal.SetLogonType(TASK_LOGON_PASSWORD)?;
            }
        } else {
            principal.SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN)?;
        }

        // Set privileges
        principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?; // TASK_RUNLEVEL_HIGHEST

        println!("[i] Setting task's parameters");
        let settings = task.Settings()?;
        settings.SetStartWhenAvailable(VARIANT_TRUE)?;
        settings.SetDisallowStartIfOnBatteries(VARIANT_FALSE)?;
        settings.SetStopIfGoingOnBatteries(VARIANT_FALSE)?;
        settings.SetAllowDemandStart(VARIANT_TRUE)?;
        settings.SetMultipleInstances(TASK_INSTANCES_IGNORE_NEW)?;
        settings.SetHidden(VARIANT_TRUE)?;

        // Set trigger
        println!("[i] Creating trigger");
        let trigger_collection = task.Triggers()?;
        let trigger = trigger_collection.Create(TASK_TRIGGER_TIME)?;
        let time_trigger: ITimeTrigger = trigger.cast()?;

        // Start time - one minute from now
        let mut st: windows::Win32::Foundation::SYSTEMTIME =
            windows::Win32::System::SystemInformation::GetLocalTime();
        st.wMinute += 1;
        if st.wMinute >= 60 {
            st.wMinute -= 60;
            st.wHour += 1;
        }

        let start_boundary = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
        );
        time_trigger.SetStartBoundary(&BSTR::from(start_boundary))?;
        time_trigger.SetEnabled(VARIANT_TRUE)?;

        // Set the action to execute
        println!("[i] Creating the action to execute");
        let action_collection = task.Actions()?;
        let action = action_collection.Create(TASK_ACTION_EXEC)?;
        let exec_action: IExecAction = action.cast()?;

        exec_action.SetPath(&BSTR::from(task_path))?;
        if let Some(args) = arguments {
            exec_action.SetArguments(&BSTR::from(args))?;
        }

        // Register the task
        let registered_task = if let (Some(user), Some(pass)) = (username, password) {
            if !user.is_empty() && !pass.is_empty() {
                println!("[i] Registering the task with username and password");
                root_folder.RegisterTaskDefinition(
                    &BSTR::from(task_name),
                    &task,
                    TASK_CREATE_OR_UPDATE.0,
                    &VARIANT::from(user),
                    &VARIANT::from(pass),
                    TASK_LOGON_PASSWORD,
                    &VARIANT::default(),
                )?
            } else {
                register_with_session_id(&root_folder, task_name, &task, session_id)?
            }
        } else {
            register_with_session_id(&root_folder, task_name, &task, session_id)?
        };

        println!("[+] Task {} successfully created", task_name);

        // Execute the task
        let resolved_user = get_user_from_session(session_id)?;
        if username.is_some() && resolved_user != username.map(|u| BSTR::from(u)) {
            println!(
                "[!] Resolved user for session {}: {}, does not match specified user: {}",
                session_id,
                resolved_user
                    .as_ref()
                    .map_or("<None>".to_string(), |b| b.to_string()),
                username.unwrap()
            );
            println!(
                "[!] The specified username does not match the resolved user for the session ID."
            );
            println!(
                "[!] We will not execute the task automatically and we will wait for the trigger or a manual execution."
            );
        } else {
            println!("[i] Triggering the task '{}'", task_name);
            let flags = if password.is_some() && !password.unwrap().is_empty() {
                TASK_RUN_NO_FLAGS
            } else {
                TASK_RUN_USE_SESSION_ID
            };

            if session_id != 0 {
                registered_task.RunEx(
                    &VARIANT::default(),
                    flags.0,
                    session_id as i32,
                    &resolved_user.unwrap_or(BSTR::from("")),
                )?;
            } else {
                registered_task.Run(&VARIANT::default())?;
            }

            println!("[+] Task {} successfully executed", task_name);
        }
    }

    Ok(())
}

fn register_with_session_id(
    task_folder: &ITaskFolder,
    task_name: &str,
    task_def: &ITaskDefinition,
    session_id: u32,
) -> Result<IRegisteredTask, windows::core::Error> {
    println!("[i] Registering the task with Session ID");
    let user_variant: VARIANT;

    let user_name = get_user_from_session(session_id)?;

    user_variant = if let Some(user_bstr) = user_name {
        println!("[+] Username resolved: {}", user_bstr.to_string());
        VARIANT::from(user_bstr)
    } else {
        println!("[i] No username resolved, your current username will be used");
        VARIANT::default()
    };

    unsafe {
        task_folder.RegisterTaskDefinition(
            &BSTR::from(task_name),
            task_def,
            TASK_CREATE_OR_UPDATE.0,
            &user_variant,
            &VARIANT::default(),
            TASK_LOGON_INTERACTIVE_TOKEN,
            &VARIANT::default(),
        )
    }
}
