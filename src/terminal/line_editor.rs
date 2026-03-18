#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LineEditorOutput {
  pub to_send: Vec<u8>,
  pub commands: Vec<String>,
}

#[derive(Debug, Default)]
pub struct LineEditor {
  buffer: String,
  cursor: usize,
  history: Vec<String>,
  history_index: usize,
  saved_line: String,
  state: EscapeState,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum EscapeState {
  #[default]
  Normal,
  Esc,
  Csi,
}

impl LineEditor {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn process_bytes(&mut self, data: &[u8]) -> LineEditorOutput {
    let mut out = LineEditorOutput::default();

    for &b in data {
      match self.state {
        EscapeState::Normal => self.process_normal(b, &mut out),
        EscapeState::Esc => self.process_esc(b, &mut out),
        EscapeState::Csi => self.process_csi(b, &mut out),
      }
    }

    out
  }

  fn process_normal(&mut self, b: u8, out: &mut LineEditorOutput) {
    match b {
      0x1b => self.state = EscapeState::Esc,
      b'\r' => {
        // echo Enter 本身只会换行；回显字符在输入时已发送。
        out.to_send.extend_from_slice(b"\r\n");

        let cmd = self.buffer.clone();
        self.buffer.clear();
        self.cursor = 0;

        if !cmd.trim().is_empty() {
          self.history.push(cmd.clone());
        }
        self.history_index = self.history.len();
        self.saved_line.clear();
        out.commands.push(cmd);
      }
      b'\n' => {}
      0x7f | 0x08 => {
        if self.cursor == 0 {
          return;
        }
        // 最小实现：仅支持行尾退格。
        if self.cursor == self.buffer.len() {
          self.buffer.pop();
          self.cursor = self.cursor.saturating_sub(1);
          out.to_send.extend_from_slice(b"\x08 \x08");
        } else {
          // 复杂的中间删除后续再补；现在先退化为不处理。
        }
      }
      x if (0x20..0x7f).contains(&x) => {
        let c = x as char;
        self.buffer.push(c);
        self.cursor = self.buffer.len();
        out.to_send.push(x);
      }
      _ => {}
    }
  }

  fn process_esc(&mut self, b: u8, _out: &mut LineEditorOutput) {
    if b == b'[' {
      self.state = EscapeState::Csi;
    } else {
      self.state = EscapeState::Normal;
    }
  }

  fn process_csi(&mut self, b: u8, out: &mut LineEditorOutput) {
    self.state = EscapeState::Normal;
    match b {
      b'A' => self.history_prev(out),
      b'B' => self.history_next(out),
      _ => {}
    }
  }

  fn history_prev(&mut self, _out: &mut LineEditorOutput) {
    if self.history.is_empty() || self.history_index == 0 {
      return;
    }
    if self.history_index == self.history.len() {
      self.saved_line = self.buffer.clone();
    }
    self.history_index -= 1;
    self.replace_line(self.history[self.history_index].clone());
  }

  fn history_next(&mut self, _out: &mut LineEditorOutput) {
    if self.history_index >= self.history.len() {
      return;
    }
    self.history_index += 1;
    if self.history_index == self.history.len() {
      self.replace_line(self.saved_line.clone());
    } else {
      self.replace_line(self.history[self.history_index].clone());
    }
  }

  fn replace_line(&mut self, new_line: String) {
    self.buffer = new_line;
    self.cursor = self.buffer.len();
  }
}
