use core_model::VaultItem;
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SafetyVault {
    conn: Connection,
    vault_dir: PathBuf,
}

impl SafetyVault {
    pub fn new<P: AsRef<Path>>(vault_base_dir: P) -> Result<Self, String> {
        let vault_dir = vault_base_dir.as_ref().to_path_buf();
        if !vault_dir.exists() {
            fs::create_dir_all(&vault_dir).map_err(|e| e.to_string())?;
        }

        let db_path = vault_dir.join("vault_journal.db");
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vault_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_path TEXT NOT NULL,
                vault_path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                deleted_timestamp_secs INTEGER NOT NULL,
                expiry_timestamp_secs INTEGER NOT NULL
            );",
            [],
        )
        .map_err(|e| e.to_string())?;

        Ok(Self { conn, vault_dir })
    }

    pub fn quarantine_file<P: AsRef<Path>>(&self, file_path: P) -> Result<VaultItem, String> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Err("Target file does not exist".into());
        }

        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        let size = metadata.len();

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Default 7 days retention
        let expiry_secs = now_secs + (7 * 24 * 3600);

        let file_name = path
            .file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy();
        let unique_vault_name = format!("{}_{}", now_secs, file_name);
        let target_vault_path = self.vault_dir.join(&unique_vault_name);

        fs::rename(path, &target_vault_path).map_err(|e| e.to_string())?;

        let orig_str = path.to_string_lossy().to_string();
        let vault_str = target_vault_path.to_string_lossy().to_string();

        self.conn
            .execute(
                "INSERT INTO vault_items (original_path, vault_path, size_bytes, deleted_timestamp_secs, expiry_timestamp_secs)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![orig_str, vault_str, size as i64, now_secs as i64, expiry_secs as i64],
            )
            .map_err(|e| e.to_string())?;

        let item_id = self.conn.last_insert_rowid();

        Ok(VaultItem {
            id: item_id,
            original_path: orig_str,
            vault_path: vault_str,
            size_bytes: size,
            deleted_timestamp_secs: now_secs,
            expiry_timestamp_secs: expiry_secs,
        })
    }

    pub fn restore_file(&self, item_id: i64) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT original_path, vault_path FROM vault_items WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        let row = stmt
            .query_row(params![item_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;

        let orig_path = PathBuf::from(&row.0);
        let vault_path = PathBuf::from(&row.1);

        if !vault_path.exists() {
            return Err("Quarantined file missing from vault disk".into());
        }

        if let Some(parent) = orig_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        fs::rename(&vault_path, &orig_path).map_err(|e| e.to_string())?;

        self.conn
            .execute("DELETE FROM vault_items WHERE id = ?1", params![item_id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn list_items(&self) -> Result<Vec<VaultItem>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, original_path, vault_path, size_bytes, deleted_timestamp_secs, expiry_timestamp_secs FROM vault_items ORDER BY deleted_timestamp_secs DESC")
            .map_err(|e| e.to_string())?;

        let items_iter = stmt
            .query_map([], |r| {
                Ok(VaultItem {
                    id: r.get(0)?,
                    original_path: r.get(1)?,
                    vault_path: r.get(2)?,
                    size_bytes: r.get::<_, i64>(3)? as u64,
                    deleted_timestamp_secs: r.get::<_, i64>(4)? as u64,
                    expiry_timestamp_secs: r.get::<_, i64>(5)? as u64,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut res = Vec::new();
        for item in items_iter {
            if let Ok(i) = item {
                res.push(i);
            }
        }
        Ok(res)
    }
}
