use std::collections::HashMap;

use crate::vfs::init_data::{SeedFile, SEED_FILES, SYSTEM_DIRS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsNodeType {
  File,
  Directory,
}

#[derive(Debug, Clone)]
pub struct VfsNode {
  pub path: String,
  pub node_type: VfsNodeType,
  pub content: String,
  pub permissions: String,
  pub owner: String,
  pub group: String,
  pub is_user_created: bool,
}

impl VfsNode {
  fn new_dir(path: String, permissions: &str, owner: &str) -> Self {
    Self {
      group: if owner == "root" {
        "root".to_string()
      } else {
        owner.to_string()
      },
      path,
      node_type: VfsNodeType::Directory,
      content: String::new(),
      permissions: permissions.to_string(),
      owner: owner.to_string(),
      is_user_created: false,
    }
  }

  fn new_file(path: String, content: &str, permissions: &str, owner: &str) -> Self {
    Self {
      group: if owner == "root" {
        "root".to_string()
      } else {
        owner.to_string()
      },
      path,
      node_type: VfsNodeType::File,
      content: content.to_string(),
      permissions: permissions.to_string(),
      owner: owner.to_string(),
      is_user_created: false,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
  Overwrite,
  Append,
}

#[derive(Debug, Default)]
pub struct VirtualFileSystem {
  nodes: HashMap<String, VfsNode>,
}

impl VirtualFileSystem {
  pub fn new(username: &str) -> Self {
    let mut vfs = Self {
      nodes: HashMap::new(),
    };
    vfs.seed(username);
    vfs
  }

  pub fn exists(&self, path: &str) -> bool {
    let p = normalize_path(path);
    self.nodes.contains_key(&p)
  }

  pub fn is_dir(&self, path: &str) -> bool {
    let p = normalize_path(path);
    self
      .nodes
      .get(&p)
      .is_some_and(|n| n.node_type == VfsNodeType::Directory)
  }

  pub fn read_file(&self, path: &str) -> Option<&str> {
    let p = normalize_path(path);
    let n = self.nodes.get(&p)?;
    if n.node_type != VfsNodeType::File {
      return None;
    }
    Some(&n.content)
  }

  pub fn write_file(&mut self, path: &str, content: &str, mode: WriteMode) {
    let p = normalize_path(path);
    self.ensure_parent_dirs(&p);

    match self.nodes.get_mut(&p) {
      Some(node) if node.node_type == VfsNodeType::File => match mode {
        WriteMode::Overwrite => node.content = content.to_string(),
        WriteMode::Append => node.content.push_str(content),
      },
      _ => {
        let mut n = VfsNode::new_file(p.clone(), content, "rw-r--r--", "root");
        n.is_user_created = true;
        self.nodes.insert(p, n);
      }
    }
  }

  pub fn create_dir(&mut self, path: &str, recursive: bool, owner: &str) {
    let p = normalize_path(path);
    if recursive {
      self.ensure_all_dirs(&p, owner);
      return;
    }

    // 非 -p 情况：父目录不存在则忽略。
    if let Some(parent) = parent_dir(&p)
      && !self.is_dir(&parent)
    {
      return;
    }

    self.insert_dir_if_missing(&p, owner);
  }

  pub fn touch(&mut self, path: &str, owner: &str) {
    let p = normalize_path(path);
    self.ensure_parent_dirs(&p);
    if self.nodes.contains_key(&p) {
      return;
    }
    let mut n = VfsNode::new_file(p.clone(), "", "rw-r--r--", owner);
    n.is_user_created = true;
    self.nodes.insert(p, n);
  }

  pub fn delete_node(&mut self, path: &str) {
    let p = normalize_path(path);
    let prefix = if p == "/" {
      "/".to_string()
    } else {
      format!("{p}/")
    };

    let to_remove: Vec<String> = self
      .nodes
      .keys()
      .filter(|k| *k == &p || k.starts_with(&prefix))
      .cloned()
      .collect();
    for k in to_remove {
      self.nodes.remove(&k);
    }
  }

  fn seed(&mut self, username: &str) {
    for d in SYSTEM_DIRS {
      self.insert_dir_if_missing(&normalize_path(d), "root");
    }

    if username != "root" {
      let home = format!("/home/{username}");
      self.insert_dir_if_missing(&normalize_path(&home), username);
    }

    for SeedFile {
      path,
      content,
      permissions,
    } in SEED_FILES
    {
      let p = normalize_path(path);
      self.ensure_parent_dirs(&p);
      self.nodes.insert(
        p.clone(),
        VfsNode::new_file(p, content, permissions, "root"),
      );
    }
  }

  fn ensure_parent_dirs(&mut self, path: &str) {
    if let Some(parent) = parent_dir(path) {
      self.ensure_all_dirs(&parent, "root");
    }
  }

  fn ensure_all_dirs(&mut self, path: &str, owner: &str) {
    let mut current = String::from("/");
    self.insert_dir_if_missing("/", owner);

    for seg in path.split('/').filter(|s| !s.is_empty()) {
      if current == "/" {
        current.push_str(seg);
      } else {
        current.push('/');
        current.push_str(seg);
      }
      self.insert_dir_if_missing(&current, owner);
    }
  }

  fn insert_dir_if_missing(&mut self, path: &str, owner: &str) {
    if self.nodes.contains_key(path) {
      return;
    }
    let mut n = VfsNode::new_dir(path.to_string(), "rwxr-xr-x", owner);
    if owner != "root" {
      n.permissions = "rwx------".to_string();
    }
    self.nodes.insert(path.to_string(), n);
  }
}

fn normalize_path(path: &str) -> String {
  let raw = if path.is_empty() { "/" } else { path };
  let raw = if raw.starts_with('/') {
    raw.to_string()
  } else {
    format!("/{raw}")
  };

  let mut stack: Vec<&str> = vec![];
  for part in raw.split('/') {
    if part.is_empty() || part == "." {
      continue;
    }
    if part == ".." {
      stack.pop();
      continue;
    }
    stack.push(part);
  }

  if stack.is_empty() {
    return "/".to_string();
  }

  let mut out = String::new();
  for seg in stack {
    out.push('/');
    out.push_str(seg);
  }
  out
}

fn parent_dir(path: &str) -> Option<String> {
  let p = normalize_path(path);
  if p == "/" {
    return None;
  }
  let idx = p.rfind('/')?;
  if idx == 0 {
    Some("/".to_string())
  } else {
    Some(p[..idx].to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn seed_has_key_paths() {
    let vfs = VirtualFileSystem::new("root");
    assert!(vfs.exists("/etc/os-release"));
    assert!(vfs.exists("/etc/hostname"));
    assert!(vfs.exists("/proc/cpuinfo"));

    assert!(vfs.is_dir("/etc"));
    assert!(!vfs.is_dir("/etc/os-release"));
  }

  #[test]
  fn read_file_returns_content() {
    let vfs = VirtualFileSystem::new("root");
    let hostname = vfs.read_file("/etc/hostname").unwrap();
    assert_eq!(hostname, "debian-srv");
  }
}
