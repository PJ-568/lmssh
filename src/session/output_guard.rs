#[derive(Debug, Clone)]
pub struct OutputGuard {
    max_bytes: usize,
    max_lines: usize,
    max_numbered_lines: usize,

    bytes: usize,
    lines: usize,
    consecutive_numbered: usize,
}

impl OutputGuard {
    pub fn new(max_bytes: usize, max_lines: usize, max_numbered_lines: usize) -> Self {
        Self {
            max_bytes,
            max_lines,
            max_numbered_lines,
            bytes: 0,
            lines: 0,
            consecutive_numbered: 0,
        }
    }

    /// Pushes a chunk and returns whether output should stop.
    pub fn push(&mut self, chunk: &str) -> bool {
        self.bytes = self.bytes.saturating_add(chunk.len());
        if self.bytes >= self.max_bytes {
            return true;
        }

        let mut should_stop = false;
        for line in chunk.split_inclusive('\n') {
            if line.ends_with('\n') {
                self.lines += 1;
                if self.lines >= self.max_lines {
                    should_stop = true;
                }
            }

            if is_numbered_line_start(line) {
                self.consecutive_numbered += 1;
                if self.consecutive_numbered >= self.max_numbered_lines {
                    should_stop = true;
                }
            } else if !line.trim().is_empty() {
                self.consecutive_numbered = 0;
            }

            if should_stop {
                break;
            }
        }

        should_stop
    }
}

fn is_numbered_line_start(line: &str) -> bool {
    let s = line.trim_start();
    let mut digits = 0usize;
    for c in s.chars() {
        if c.is_ascii_digit() {
            digits += 1;
            continue;
        }
        break;
    }
    if digits == 0 {
        return false;
    }
    let rest = &s[digits..];
    rest.starts_with('.') || rest.starts_with(')')
}
