pub const BLACKLISTED_COMMANDS: &[&str] = &[
  // editors
  "vim",
  "vi",
  "nano",
  "emacs",
  // pagers
  "less",
  "more",
  // multiplexers
  "tmux",
  "screen",
  // monitors
  "top",
  "htop",
];

pub fn is_blacklisted(cmd0: &str) -> bool {
  BLACKLISTED_COMMANDS.iter().any(|c| *c == cmd0)
}
