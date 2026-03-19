use crate::vfs::tree::{VirtualFileSystem, WriteMode};

pub fn apply_side_effects(vfs: &mut VirtualFileSystem, cmd: &str) {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return;
    }

    if let Some((left, op, right)) = split_redirection(trimmed) {
        let path = right.split_whitespace().next().unwrap_or("");
        if path.is_empty() {
            return;
        }

        let mode = if op == ">>" {
            WriteMode::Append
        } else {
            WriteMode::Overwrite
        };

        let content = echo_payload(left);
        vfs.write_file(path, &content, mode);
        return;
    }

    let mut tokens = trimmed.split_whitespace();
    let Some(cmd0) = tokens.next() else {
        return;
    };

    match cmd0 {
        "mkdir" => {
            let mut recursive = false;
            let mut paths = vec![];
            for t in tokens {
                if t == "-p" {
                    recursive = true;
                    continue;
                }
                if t.starts_with('-') {
                    continue;
                }
                paths.push(t);
            }

            for p in paths {
                vfs.create_dir(p, recursive, "root");
            }
        }
        "touch" => {
            for t in tokens {
                if t.starts_with('-') {
                    continue;
                }
                vfs.touch(t, "root");
            }
        }
        "rm" => {
            for t in tokens {
                if t.starts_with('-') {
                    continue;
                }
                vfs.delete_node(t);
            }
        }
        _ => {}
    }
}

fn split_redirection(input: &str) -> Option<(&str, &str, &str)> {
    if let Some((l, r)) = input.split_once(">>") {
        return Some((l.trim_end(), ">>", r.trim_start()));
    }
    if let Some((l, r)) = input.split_once('>') {
        return Some((l.trim_end(), ">", r.trim_start()));
    }
    None
}

fn echo_payload(left: &str) -> String {
    let l = left.trim();
    let Some(rest) = l.strip_prefix("echo ") else {
        return String::new();
    };

    let mut s = rest.trim().to_string();
    if let Some(stripped) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')) {
        s = stripped.to_string();
    }
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_effects_mkdir_touch_rm_and_redirect() {
        let mut vfs = VirtualFileSystem::new("root");
        apply_side_effects(&mut vfs, "mkdir -p /tmp/a/b");
        assert!(vfs.is_dir("/tmp/a"));
        assert!(vfs.is_dir("/tmp/a/b"));

        apply_side_effects(&mut vfs, "touch /tmp/a/b/file.txt");
        assert!(vfs.exists("/tmp/a/b/file.txt"));

        apply_side_effects(&mut vfs, "echo hi > /tmp/a/b/out.txt");
        assert_eq!(vfs.read_file("/tmp/a/b/out.txt").unwrap(), "hi\n");

        apply_side_effects(&mut vfs, "rm -rf /tmp/a/b/file.txt");
        assert!(!vfs.exists("/tmp/a/b/file.txt"));
    }
}
