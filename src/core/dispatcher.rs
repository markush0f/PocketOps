use crate::core::server_manager::ServerManager;
use crate::executor::ssh::SshExecutor;
use crate::models::command::SystemCommand;

pub async fn dispatch(command: SystemCommand) -> String {
    let manager = ServerManager::new();

    match command {
        SystemCommand::GetStatus => "System status: Operational".to_string(),
        
        SystemCommand::Help => {
            "Available commands:
/status - Check bot status
/servers - List configured servers
/add <alias> <host> <user> - Add a new server
/remove <alias> - Remove a server
/exec <alias> <cmd> - Execute a command on a server"
            .to_string()
        }

        SystemCommand::AddServer { alias, host, user } => {
            manager.add_server(alias.clone(), host, user, 22, None);
            format!("Server '{}' added successfully (Key-based auth assumed).", alias)
        }

        SystemCommand::RemoveServer { alias } => {
            if manager.remove_server(&alias) {
                format!("Server '{}' removed.", alias)
            } else {
                format!("Server '{}' not found.", alias)
            }
        }

        SystemCommand::ListServers => {
            let servers = manager.list_servers();
            if servers.is_empty() {
                "No servers configured.".to_string()
            } else {
                let mut out = "Configured Servers:\n".to_string();
                for (alias, server) in servers {
                    out.push_str(&format!("- {}: {}@{}\n", alias, server.ssh_user, server.hostname));
                }
                out
            }
        }

        SystemCommand::Exec { alias, cmd } => {
            if let Some(server) = manager.get_server(&alias) {
                match SshExecutor::execute(&server, &cmd) {
                    Ok(output) => format!("Output from {}:\n{}", alias, output),
                    Err(e) => format!("Error executing on {}: {}", alias, e),
                }
            } else {
                format!("Server '{}' not found. Use /add to configure it.", alias)
            }
        }

        SystemCommand::Unknown => "Unknown command. Type /help for assistance.".to_string(),
    }
}
