use broker::BrokerServer;
use platform_win::{enable_privilege, is_elevated};

fn main() {
    println!("Lumen Privileged Broker starting...");
    if is_elevated() {
        println!("Running with Administrator privileges.");
        let _ = enable_privilege("SeBackupPrivilege");
        let _ = enable_privilege("SeRestorePrivilege");
        let _ = enable_privilege("SeProfileSingleProcessPrivilege");
    } else {
        println!("Warning: Broker launched without elevation. Privileged commands may fail.");
    }

    let server = BrokerServer::new(None);
    println!("Named Pipe server listening on LumenBrokerPipe...");
    if let Err(e) = server.run() {
        eprintln!("Broker server error: {}", e);
    }
}
