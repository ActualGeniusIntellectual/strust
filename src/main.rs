use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use nix::pty::{openpty, OpenptyResult};
use nix::unistd::{close, dup2, execvp, fork, ForkResult};
use unicode_width::UnicodeWidthChar;

mod win;

// Constants from the C code
const UTF_INVALID: u32 = 0xFFFD;
const UTF_SIZ: usize = 4;
const ESC_BUF_SIZ: usize = 128 * UTF_SIZ;
const ESC_ARG_SIZ: usize = 16;
const STR_BUF_SIZ: usize = ESC_BUF_SIZ;
const STR_ARG_SIZ: usize = ESC_ARG_SIZ;

// Converted C macros to Rust functions or inline code
fn is_set(flag: u32, term_mode: u32) -> bool {
    (term_mode & flag) != 0
}

fn is_control_c0(c: char) -> bool {
    ('\0'..='\x1f').contains(&c) || c == '\x7f'
}

fn is_control_c1(c: char) -> bool {
    ('\u{80}'..='\u{9f}').contains(&c)
}

fn is_control(c: char) -> bool {
    is_control_c0(c) || is_control_c1(c)
}

fn is_delim(c: char, word_delimiters: &str) -> bool {
    c != '\0' && word_delimiters.contains(c)
}

// Enums converted to Rust enums
#[derive(Clone, Copy, PartialEq, Eq)]
enum TermMode {
    Wrap = 1 << 0,
    Insert = 1 << 1,
    Altscreen = 1 << 2,
    Crlf = 1 << 3,
    Echo = 1 << 4,
    Print = 1 << 5,
    Utf8 = 1 << 6,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorMovement {
    Save,
    Load,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorState {
    Default = 0,
    WrapNext = 1,
    Origin = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Charset {
    Graphic0,
    Graphic1,
    Uk,
    Usa,
    Multi,
    Ger,
    Fin,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EscapeState {
    Start = 1,
    Csi = 2,
    Str = 4,
    AltCharset = 8,
    StrEnd = 16,
    Test = 32,
    Utf8 = 64,
}

// Structs translated to Rust structs
struct TCursor {
    attr: st::Glyph, // current char attributes
    x: i32,          // terminal column
    y: i32,          // terminal row
    state: char,
}

// Define a Rust struct corresponding to the C Selection struct
struct Selection {
    mode: i32,
    selection_type: i32, // Renamed from `type` to `selection_type` to avoid Rust keyword conflict
    snap: i32,

    // The nested struct is now a distinct Rust struct
    nb: Coordinates, // normalized coordinates of the beginning of the selection
    ne: Coordinates, // normalized coordinates of the end of the selection
    ob: Coordinates, // original coordinates of the beginning of the selection
    oe: Coordinates, // original coordinates of the end of the selection

    alt: i32,
}

// Define the Coordinates struct used in Selection
struct Coordinates {
    x: i32,
    y: i32,
}


struct Term {
    row: i32,           // Number of rows
    col: i32,           // Number of columns
    line: Vec<Line>,    // Screen buffer
    alt: Vec<Line>,     // Alternate screen buffer
    dirty: Vec<bool>,   // Dirtiness of lines
    cursor: TCursor,    // Cursor
    ocx: i32,           // Old cursor column
    ocy: i32,           // Old cursor row
    top: i32,           // Top scroll limit
    bot: i32,           // Bottom scroll limit
    mode: i32,          // Terminal mode flags
    esc: i32,           // Escape state flags
    trantbl: [char; 4], // Charset table translation
    charset: i32,       // Current charset
    icharset: i32,      // Selected charset for sequence
    tabs: Vec<bool>,    // Tab stops
    lastc: Rune,        // Last printed character (0 if control character)
}

// CSI Escape sequence struct
struct CSIEscape {
    buf: [char; ESC_BUF_SIZ], // raw string
    len: usize,               // raw string length
    priv_: char,
    arg: [i32; ESC_ARG_SIZ],
    narg: i32,                // nb of args
    mode: [char; 2],
}

// STR Escape sequence struct
struct STREscape {
    escape_type: char,            // ESC type
    buf: String,                  // Allocated raw string
    size: usize,                  // Allocation size
    length: usize,                // Raw string length
    args: Vec<String>,            // Arguments
    num_args: usize,              // Number of arguments
}


fn main() {
    println!("Hello, world!");
}
