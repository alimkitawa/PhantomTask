use crate::utils::from_wide_string;

use std::ptr::null_mut;

use windows::Win32::System::RemoteDesktop::{
    WTSActive, WTSConnectQuery, WTSConnected, WTSDisconnected, WTSDomainName, WTSDown,
    WTSEnumerateSessionsW, WTSFreeMemory, WTSIdle, WTSInit,
    WTSListen, WTSQuerySessionInformationW, WTSReset, WTSShadow, WTSUserName,
    WTS_CURRENT_SERVER_HANDLE, WTS_SESSION_INFOW,
};

use windows_core::BSTR;

pub fn list_active_sessions() -> windows::core::Result<()> {
    println!("\n===== Active Sessions =====\n");
    println!(
        "{:<10} {:<20} {:<15} {:<20} {:<20}",
        "SessionID", "User", "State", "Station", "Domain"
    );
    println!("{}", "=".repeat(85));

    unsafe {
        let mut session_info: *mut WTS_SESSION_INFOW = null_mut();
        let mut count: u32 = 0;

        WTSEnumerateSessionsW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            0,
            1,
            &mut session_info,
            &mut count,
        )?;
        let sessions = std::slice::from_raw_parts(session_info, count as usize);

        for session in sessions {
            let session_id = session.SessionId;

            //println!("[i] Processing Session ID: {}", session_id);

            // Récupérer le nom d'utilisateur
            let mut user_name: *mut u16 = null_mut();
            let mut user_size: u32 = 0;

            let user_str = match WTSQuerySessionInformationW(
                Some(WTS_CURRENT_SERVER_HANDLE),
                session_id,
                WTSUserName,
                &mut user_name as *mut _ as *mut _,
                &mut user_size,
            ) {
                Ok(_) => {
                    let name = from_wide_string(user_name as *const u16);
                    if name.is_empty() {
                        "<None>".to_string()
                    } else {
                        name
                    }
                }
                Err(_) => "<Error>".to_string(),
            };

            WTSFreeMemory(user_name as *mut _);

            // Get the domain name
            let mut domain_name: *mut u16 = null_mut();
            let mut domain_size: u32 = 0;

            let domain_str = match WTSQuerySessionInformationW(
                Some(WTS_CURRENT_SERVER_HANDLE),
                session_id,
                WTSDomainName,
                &mut domain_name as *mut _ as *mut _,
                &mut domain_size,
            ) {
                Ok(_) => {
                    let name = from_wide_string(domain_name as *const u16);
                    if name.is_empty() {
                        "<Local>".to_string()
                    } else {
                        name
                    }
                }
                Err(_) => "N/A".to_string(),
            };

            WTSFreeMemory(domain_name as *mut _);

            // State
            #[allow(non_upper_case_globals)]
            let state_str = match session.State {
                WTSActive => "Active",
                WTSConnected => "Connected",
                WTSConnectQuery => "ConnectQuery",
                WTSShadow => "Shadow",
                WTSDisconnected => "Disconnected",
                WTSIdle => "Idle",
                WTSListen => "Listen",
                WTSReset => "Reset",
                WTSDown => "Down",
                WTSInit => "Init",
                _ => "Unknown",
            };

            let station_name = from_wide_string(session.pWinStationName.as_ptr());

            println!(
                "{:<10} {:<20} {:<15} {:<20} {:<20}",
                session_id, user_str, state_str, station_name, domain_str
            );
        }

        WTSFreeMemory(session_info as *mut _);
    }

    Ok(())
}

pub fn get_user_from_session(session_id: u32) -> Result<Option<BSTR>, windows::core::Error> {
    unsafe {
        let mut user_name: *mut u16 = null_mut();
        let mut user_size: u32 = 0;

        if session_id == 0 {
            return Ok(Some(BSTR::from("Système"))); // Change this according to your location (system on english systems, ...)
        }

        match WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            session_id,
            WTSUserName,
            &mut user_name as *mut _ as *mut _,
            &mut user_size,
        ) {
            Ok(_) => {
                let name = from_wide_string(user_name as *const u16);
                WTSFreeMemory(user_name as *mut _);

                if !name.is_empty() {
                    return Ok(Some(BSTR::from(name)));
                } else {
                    return Ok(Some(BSTR::from("")));
                }
            }
            Err(_) => return Ok(Some(BSTR::from("[!] WTSQuerySessionInformationW Error"))),
        };
    }
}