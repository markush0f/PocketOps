use crate::core::server_manager::ServerManager;
use crate::executor::ssh::SshExecutor;
use crate::models::command::SystemCommand;

pub async fn dispatch(command: SystemCommand) -> String {
    let manager = ServerManager::new();

    match command {
        SystemCommand::GetStatus => "System status: Operational".to_string(),

        SystemCommand::Help => {
            let mut help_msg = "Available commands:\n".to_string();
            for (cmd, desc) in SystemCommand::all_commands_info() {
                help_msg.push_str(&format!("  {} - {}\n", cmd, desc));
            }
            help_msg
        }

        SystemCommand::AddServer { alias, host, user } => {
            manager.add_server(alias.clone(), host, user, 22, None);
            format!(
                "Server '{}' added successfully (Key-based auth assumed).",
                alias
            )
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
                    out.push_str(&format!(
                        "- {}: {}@{}\n",
                        alias, server.ssh_user, server.hostname
                    ));
                }
                out
            }
        }

        SystemCommand::Exec { alias, cmd } => {
            println!("Dispatcher: Executing '{}' on '{}'", cmd, alias);
            if let Some(server) = manager.get_server(&alias) {
                println!("Dispatcher: Server found. Connecting...");
                match SshExecutor::execute(&server, &cmd) {
                    Ok(output) => {
                        println!("Dispatcher: Execution successful.");
                        format!("Output from {}:\n{}", alias, output)
                    }
                    Err(e) => {
                        println!("Dispatcher: Execution failed: {}", e);
                        format!("Error executing on {}: {}", alias, e)
                    }
                }
            } else {
                format!("Server '{}' not found. Use /add to configure it.", alias)
            }
        }

        SystemCommand::Ask { question } => format!(
            "AI processing is not yet implemented. You asked: {}",
            question
        ),
        SystemCommand::Unknown => "Unknown command. Type /help for assistance.".to_string(),
    }
}
