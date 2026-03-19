use lmssh::terminal::LineEditor;

#[test]
fn typing_and_enter_emits_command() {
    let mut ed = LineEditor::new();
    let out = ed.process_bytes(b"ls\r");

    assert_eq!(out.commands, vec!["ls".to_string()]);
    assert_eq!(out.to_send, b"ls\r\n".to_vec());
}

#[test]
fn backspace_deletes_last_char() {
    let mut ed = LineEditor::new();
    let out = ed.process_bytes(b"ab\x7f\r");

    assert_eq!(out.commands, vec!["a".to_string()]);
    assert_eq!(out.to_send, b"ab\x08 \x08\r\n".to_vec());
}

#[test]
fn up_arrow_replaces_line_from_history() {
    let mut ed = LineEditor::new();
    let _ = ed.process_bytes(b"foo\r");

    let out = ed.process_bytes(b"\x1b[A\r");
    assert_eq!(out.commands, vec!["foo".to_string()]);
}
