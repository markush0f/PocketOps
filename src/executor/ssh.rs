use crate::models::ManagedServer;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

pub struct SshExecutor;

impl SshExecutor {
    /// Connects to a server and executes a command via SSH.
    /// Returns the standard output if successful.
    pub fn execute(server: &ManagedServer, command: &str) -> Result<String, String> {
        //Establish TCP connection
        let address = format!("{}:{}", server.ip_address, server.port);
        let tcp = TcpStream::connect(&address)
            .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;

        // Initialize SSH Session
        let mut sess =
            Session::new().map_err(|e| format!("Failed to create SSH session: {}", e))?;
        sess.set_tcp_stream(tcp);
        sess.handshake()
            .map_err(|e| format!("SSH handshake failed: {}", e))?;
        println!("SSH: Handshake successful.");

        // Authenticate
        // Try to authenticate using the SSH agent first
        if sess.userauth_agent(&server.ssh_user).is_err() {
            // Fallback: Try using the local id_rsa key if agent fails
            let _ = sess
                .userauth_pubkey_file(
                    &server.ssh_user,
                    None,
                    Path::new(&format!(
                        "{}/.ssh/id_rsa",
                        std::env::var("HOME").unwrap_or_default()
                    )),
                    None,
                )
                .map_err(|e| format!("Authentication failed (Agent & key): {}", e))?;
        }

        // Fallback: Try password if keys fail and password is provided
        if !sess.authenticated() {
            if let Some(pwd) = &server.password {
                sess.userauth_password(&server.ssh_user, pwd)
                    .map_err(|e| format!("Password authentication failed: {}", e))?;
            }
        }

        if !sess.authenticated() {
            return Err(
                "Authentication failed: Unable to authenticate via Agent, Key, or Password."
                    .to_string(),
            );
        }

        // Create Channel and Execute Command
        let mut channel = sess
            .channel_session()
            .map_err(|e| format!("Failed to create channel: {}", e))?;
        channel
            .exec(command)
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        // Read Output
        let mut stdout = String::new();
        channel
            .read_to_string(&mut stdout)
            .map_err(|e| format!("Failed to read stdout: {}", e))?;

        // Also try to read stderr
        let mut stderr = String::new();
        channel
            .stderr()
            .read_to_string(&mut stderr)
            .unwrap_or_default();

        channel
            .wait_close()
            .map_err(|e| format!("Failed to close channel: {}", e))?;

        // Check exit status
        let exit_status = channel.exit_status().unwrap_or(-1);
        if exit_status != 0 {
            return Err(format!(
                "Command exited with status {}.\nStdout: {}\nStderr: {}",
                exit_status, stdout, stderr
            ));
        }

        if stdout.is_empty() && !stderr.is_empty() {
            Ok(format!("(stderr): {}", stderr))
        } else {
            Ok(stdout)
        }
    }
}
