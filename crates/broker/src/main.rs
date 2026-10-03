use platform_win::{enable_privilege, is_elevated};

fn main() {
    println!("Lumen Privileged Broker starting...");
    if is_elevated() {
        println!("Running with Administrator privileges.");
        let _ = enable_privilege("SeBackupPrivilege");
        let _ = enable_privilege("SeRestorePrivilege");
    } else {
        println!("Warning: Broker launched without elevation.");
    }
}
