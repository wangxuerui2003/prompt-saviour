use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use ps_core::{build_snapshot, CaptureSource, CaptureSnapshot, SessionContext};
use rdev::{Event, EventType, Key};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy)]
pub enum PasteChord {
    Meta,
    Control,
}

pub struct KeystrokeHost {
    pub running: Arc<AtomicBool>,
    pub on_snapshot: Arc<dyn Fn(CaptureSnapshot) + Send + Sync>,
    pub session: Arc<dyn Fn() -> Option<SessionContext> + Send + Sync>,
    pub clipboard: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    pub request_gui_poll: Arc<dyn Fn() + Send + Sync>,
    pub paste_chord: PasteChord,
}

/// Returns true when `rdev::listen` can be acquired (global keystroke hook).
/// Result is cached for the process lifetime to avoid spawning multiple listeners.
pub fn probe_input_monitoring() -> bool {
    use std::sync::OnceLock;

    static PROBE: OnceLock<bool> = OnceLock::new();
    *PROBE.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let ok = rdev::listen(|_| {}).is_ok();
            let _ = tx.send(ok);
        });

        match rx.recv_timeout(Duration::from_millis(300)) {
            Ok(ok) => ok,
            Err(mpsc::RecvTimeoutError::Timeout) => true,
            Err(mpsc::RecvTimeoutError::Disconnected) => false,
        }
    })
}

struct TapState {
    host: KeystrokeHost,
    buffers: HashMap<String, String>,
    last_session: Option<SessionContext>,
    meta_held: bool,
    alt_held: bool,
    ctrl_held: bool,
}

pub fn run_keystroke_listener(host: KeystrokeHost) {
    let state = Arc::new(Mutex::new(TapState {
        host,
        buffers: HashMap::new(),
        last_session: None,
        meta_held: false,
        alt_held: false,
        ctrl_held: false,
    }));

    info!("keystroke listener starting (rdev)");
    if let Err(err) = rdev::listen({
        let state = Arc::clone(&state);
        move |event| {
            let mut state = state.lock();
            if let Err(err) = handle_event(&mut state, event) {
                warn!(?err, "keystroke handling failed");
            }
        }
    }) {
        error!(?err, "keystroke listener exited");
    }
}

fn handle_event(state: &mut TapState, event: Event) -> anyhow::Result<()> {
    if !state.host.running.load(Ordering::SeqCst) {
        return Ok(());
    }

    match event.event_type {
        EventType::KeyPress(key) => handle_key_press(state, key),
        EventType::KeyRelease(key) => {
            update_modifier_state(state, key, false);
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_key_press(state: &mut TapState, key: Key) -> anyhow::Result<()> {
    if is_modifier_key(key) {
        update_modifier_state(state, key, true);
        return Ok(());
    }

    let session = (state.host.session)().or_else(|| state.last_session.clone());
    let Some(session) = session else {
        return Ok(());
    };
    state.last_session = Some(session.clone());
    let slot_key = session.slot_key();

    if key == Key::Backspace && state.alt_held {
        (state.host.request_gui_poll)();
        delete_word_backward(state.buffers.entry(slot_key).or_default());
        return flush_buffer(state, session);
    }

    if state.meta_held || state.alt_held || state.ctrl_held {
        (state.host.request_gui_poll)();
        if paste_pressed(state, key) {
            if let Some(text) = (state.host.clipboard)() {
                state.buffers.entry(slot_key).or_default().push_str(&text);
                return flush_buffer(state, session);
            }
        }
        return Ok(());
    }

    let buffer = state.buffers.entry(slot_key).or_default();
    apply_key(buffer, key);
    flush_buffer(state, session)
}

fn paste_pressed(state: &TapState, key: Key) -> bool {
    if key != Key::KeyV {
        return false;
    }
    match state.host.paste_chord {
        PasteChord::Meta => state.meta_held,
        PasteChord::Control => state.ctrl_held,
    }
}

fn is_modifier_key(key: Key) -> bool {
    matches!(
        key,
        Key::MetaLeft
            | Key::MetaRight
            | Key::Alt
            | Key::AltGr
            | Key::ControlLeft
            | Key::ControlRight
            | Key::ShiftLeft
            | Key::ShiftRight
    )
}

fn update_modifier_state(state: &mut TapState, key: Key, pressed: bool) {
    match key {
        Key::MetaLeft | Key::MetaRight => state.meta_held = pressed,
        Key::Alt | Key::AltGr => state.alt_held = pressed,
        Key::ControlLeft | Key::ControlRight => state.ctrl_held = pressed,
        _ => {}
    }
}

fn delete_word_backward(buffer: &mut String) {
    while buffer.ends_with(' ') {
        buffer.pop();
    }
    while let Some(ch) = buffer.chars().last() {
        if ch.is_whitespace() {
            break;
        }
        buffer.pop();
    }
    while buffer.ends_with(' ') {
        buffer.pop();
    }
}

fn apply_key(buffer: &mut String, key: Key) {
    match key {
        Key::Backspace => {
            buffer.pop();
        }
        Key::Return => buffer.push('\n'),
        Key::Space => buffer.push(' '),
        Key::Tab => buffer.push('\t'),
        Key::Minus => buffer.push('-'),
        Key::Equal => buffer.push('='),
        Key::LeftBracket => buffer.push('['),
        Key::RightBracket => buffer.push(']'),
        Key::BackSlash => buffer.push('\\'),
        Key::SemiColon => buffer.push(';'),
        Key::Quote => buffer.push('\''),
        Key::Comma => buffer.push(','),
        Key::Dot => buffer.push('.'),
        Key::Slash => buffer.push('/'),
        Key::BackQuote => buffer.push('`'),
        Key::Num1 => buffer.push('1'),
        Key::Num2 => buffer.push('2'),
        Key::Num3 => buffer.push('3'),
        Key::Num4 => buffer.push('4'),
        Key::Num5 => buffer.push('5'),
        Key::Num6 => buffer.push('6'),
        Key::Num7 => buffer.push('7'),
        Key::Num8 => buffer.push('8'),
        Key::Num9 => buffer.push('9'),
        Key::Num0 => buffer.push('0'),
        Key::KeyA => buffer.push('a'),
        Key::KeyB => buffer.push('b'),
        Key::KeyC => buffer.push('c'),
        Key::KeyD => buffer.push('d'),
        Key::KeyE => buffer.push('e'),
        Key::KeyF => buffer.push('f'),
        Key::KeyG => buffer.push('g'),
        Key::KeyH => buffer.push('h'),
        Key::KeyI => buffer.push('i'),
        Key::KeyJ => buffer.push('j'),
        Key::KeyK => buffer.push('k'),
        Key::KeyL => buffer.push('l'),
        Key::KeyM => buffer.push('m'),
        Key::KeyN => buffer.push('n'),
        Key::KeyO => buffer.push('o'),
        Key::KeyP => buffer.push('p'),
        Key::KeyQ => buffer.push('q'),
        Key::KeyR => buffer.push('r'),
        Key::KeyS => buffer.push('s'),
        Key::KeyT => buffer.push('t'),
        Key::KeyU => buffer.push('u'),
        Key::KeyV => buffer.push('v'),
        Key::KeyW => buffer.push('w'),
        Key::KeyX => buffer.push('x'),
        Key::KeyY => buffer.push('y'),
        Key::KeyZ => buffer.push('z'),
        _ => {}
    }
}

fn flush_buffer(state: &mut TapState, session: SessionContext) -> anyhow::Result<()> {
    let slot_key = session.slot_key();
    let Some(content) = state.buffers.get(&slot_key).cloned() else {
        return Ok(());
    };
    if content.trim().chars().count() < 8 {
        return Ok(());
    }
    let snapshot = build_snapshot(session, content, CaptureSource::Keystroke);
    (state.host.on_snapshot)(snapshot);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_word_backward_removes_last_token() {
        let mut buf = "hello world test".to_string();
        delete_word_backward(&mut buf);
        assert_eq!(buf, "hello world");
    }
}
