use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub parent: u32,
    pub first_child: u32,
    pub next_sibling: u32,
    pub name_offset: u32,
    pub name_len: u16,
    pub attrs: u32,
    pub size_bytes: u64,
    pub allocated_bytes: u64,
    pub created_secs: i64,
    pub modified_secs: i64,
    pub file_count: u32,
    pub dir_count: u32,
}

impl Node {
    pub fn is_dir(&self) -> bool {
        (self.attrs & 0x10) != 0
    }

    pub fn is_hidden(&self) -> bool {
        (self.attrs & 0x02) != 0
    }

    pub fn is_system(&self) -> bool {
        (self.attrs & 0x04) != 0
    }

    pub fn is_reparse_point(&self) -> bool {
        (self.attrs & 0x400) != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeIndex {
    pub volume_letter: char,
    pub volume_name: String,
    pub nodes: Vec<Node>,
    pub names_pool: String,
    pub scanned_timestamp: i64,
    pub total_size_bytes: u64,
    pub total_files_count: u64,
    pub total_dirs_count: u64,
}

impl VolumeIndex {
    pub fn new(volume_letter: char, volume_name: String) -> Self {
        let mut idx = Self {
            volume_letter,
            volume_name,
            nodes: Vec::with_capacity(100_000),
            names_pool: String::with_capacity(1_000_000),
            scanned_timestamp: chrono::Utc::now().timestamp(),
            total_size_bytes: 0,
            total_files_count: 0,
            total_dirs_count: 0,
        };

        // Root node at index 0
        idx.add_node(
            u32::MAX,
            &format!("{}:\\", volume_letter),
            0x10, // FILE_ATTRIBUTE_DIRECTORY
            0,
            0,
            idx.scanned_timestamp,
            idx.scanned_timestamp,
        );

        idx
    }

    pub fn add_node(
        &mut self,
        parent: u32,
        name: &str,
        attrs: u32,
        size_bytes: u64,
        allocated_bytes: u64,
        created_secs: i64,
        modified_secs: i64,
    ) -> u32 {
        let name_offset = self.names_pool.len() as u32;
        let name_len = name.len() as u16;
        self.names_pool.push_str(name);

        let new_idx = self.nodes.len() as u32;
        let node = Node {
            parent,
            first_child: u32::MAX,
            next_sibling: u32::MAX,
            name_offset,
            name_len,
            attrs,
            size_bytes,
            allocated_bytes,
            created_secs,
            modified_secs,
            file_count: if (attrs & 0x10) == 0 { 1 } else { 0 },
            dir_count: if (attrs & 0x10) != 0 && new_idx != 0 { 1 } else { 0 },
        };

        self.nodes.push(node);

        if parent != u32::MAX && (parent as usize) < self.nodes.len() {
            let prev_first = self.nodes[parent as usize].first_child;
            self.nodes[new_idx as usize].next_sibling = prev_first;
            self.nodes[parent as usize].first_child = new_idx;
        }

        new_idx
    }

    pub fn get_name(&self, node_idx: usize) -> &str {
        if node_idx >= self.nodes.len() {
            return "";
        }
        let node = &self.nodes[node_idx];
        let start = node.name_offset as usize;
        let end = start + node.name_len as usize;
        if end <= self.names_pool.len() {
            &self.names_pool[start..end]
        } else {
            ""
        }
    }

    pub fn build_full_path(&self, node_idx: usize) -> String {
        let mut parts = Vec::new();
        let mut curr = node_idx as u32;

        while curr != u32::MAX && (curr as usize) < self.nodes.len() {
            let name = self.get_name(curr as usize);
            parts.push(name);
            curr = self.nodes[curr as usize].parent;
        }

        parts.reverse();
        if parts.is_empty() {
            return String::new();
        }

        let root = parts[0];
        if parts.len() == 1 {
            return root.to_string();
        }

        let sub_path = parts[1..].join("\\");
        format!("{}{}", root, sub_path)
    }

    /// Bottom-up aggregation: calculates directory sizes and counts from children
    pub fn aggregate_sizes(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        // Post-order / reverse traversal guarantees children are aggregated before their parents
        for idx in (1..self.nodes.len()).rev() {
            let node = self.nodes[idx].clone();
            let parent = node.parent;

            if parent != u32::MAX && (parent as usize) < self.nodes.len() {
                self.nodes[parent as usize].size_bytes += node.size_bytes;
                self.nodes[parent as usize].allocated_bytes += node.allocated_bytes;
                self.nodes[parent as usize].file_count += node.file_count;
                self.nodes[parent as usize].dir_count += node.dir_count;
            }
        }

        self.total_size_bytes = self.nodes[0].size_bytes;
        self.total_files_count = self.nodes[0].file_count as u64;
        self.total_dirs_count = self.nodes[0].dir_count as u64;
    }

    /// Returns direct children of a given node
    pub fn get_children(&self, parent_idx: u32) -> Vec<u32> {
        let mut children = Vec::new();
        if (parent_idx as usize) >= self.nodes.len() {
            return children;
        }

        let mut curr = self.nodes[parent_idx as usize].first_child;
        while curr != u32::MAX && (curr as usize) < self.nodes.len() {
            children.push(curr);
            curr = self.nodes[curr as usize].next_sibling;
        }
        children
    }

    /// Returns top items sorted by size or age
    pub fn get_top_items(&self, limit: usize) -> Vec<core_model::FileRecord> {
        let now = chrono::Utc::now().timestamp();
        let mut sorted_indices: Vec<usize> = (1..self.nodes.len()).collect();
        // Sort descending by size
        sorted_indices.sort_unstable_by(|&a, &b| {
            self.nodes[b].size_bytes.cmp(&self.nodes[a].size_bytes)
        });

        sorted_indices
            .into_iter()
            .take(limit)
            .map(|idx| {
                let node = &self.nodes[idx];
                let path = self.build_full_path(idx);
                let name = self.get_name(idx).to_string();
                let is_dir = node.is_dir();

                let age_secs = (now - node.created_secs).max(0);
                let age_days = age_secs / 86400;
                let age_years = age_days / 365;
                let age_months = (age_days % 365) / 30;

                let age_label = if age_years > 0 {
                    format!("{} ปี {} เดือน", age_years, age_months)
                } else if age_months > 0 {
                    format!("{} เดือน", age_months)
                } else {
                    format!("{} วัน", age_days.max(1))
                };

                let created_year = chrono::DateTime::from_timestamp(node.created_secs, 0)
                    .map(|dt| chrono::Datelike::year(&dt))
                    .unwrap_or(2024);

                let lower_name = name.to_lowercase();
                let (label, reason) = if lower_name.ends_with(".iso") || lower_name.ends_with(".zip") || lower_name.ends_with(".rar") || lower_name.ends_with(".7z") {
                    ("Review".to_string(), "ไฟล์บีบอัดหรือ Installer เก่าขนาดใหญ่".to_string())
                } else if lower_name.contains("cache") || lower_name.contains("temp") {
                    ("Safe".to_string(), "แคชหรือไฟล์ชั่วคราว สามารถล้างได้".to_string())
                } else if is_dir {
                    ("Review".to_string(), "โฟลเดอร์ขนาดใหญ่ในไดรฟ์".to_string())
                } else {
                    ("Keep".to_string(), "ไฟล์ข้อมูลของผู้ใช้".to_string())
                };

                core_model::FileRecord {
                    id: idx.to_string(),
                    name,
                    path,
                    size_bytes: node.size_bytes,
                    created_year,
                    age_label,
                    is_dir,
                    label,
                    reason,
                }
            })
            .collect()
    }
}
