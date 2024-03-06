use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use unicode_width::UnicodeWidthChar;

mod win;
mod st;

use crate::st::Line;

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

// Assuming relevant types and constants are defined elsewhere in your Rust code.
type Rune = u32; // Placeholder type, adjust as needed
struct Glyph; // Placeholder struct, define as needed

// Functions translated from C to Rust
fn _execsh(_cmd: *mut char, _args: *mut *mut char) {
    // Function body to be implemented
}

fn _stty(_args: *mut *mut char) {
    // Function body to be implemented
}

fn _sigchld(_signum: i32) {
    // Function body to be implemented
}

fn _tty_writeraw(_s: *const char, _len: usize) {
    // Function body to be implemented
}

fn _csi_dump() {
    // Function body to be implemented
}

fn _csi_handle() {
    // Function body to be implemented
}

fn _csi_parse() {
    // Function body to be implemented
}

fn _csi_reset() {
    // Function body to be implemented
}

fn _osc_color_response(_p1: i32, _p2: i32, _p3: i32) {
    // Function body to be implemented
}

fn _eschandle(_c: u8) -> i32 {
    // Function body to be implemented
    0
}

fn _str_dump() {
    // Function body to be implemented
}

fn _str_handle() {
    // Function body to be implemented
}

fn _str_parse() {
    // Function body to be implemented
}

fn _str_reset() {
    // Function body to be implemented
}

fn _tprinter(_s: *mut char, _len: usize) {
    // Function body to be implemented
}

fn _tdumpsel() {
    // Function body to be implemented
}

fn _tdumpline(_line: i32) {
    // Function body to be implemented
}

fn _tdump() {
    // Function body to be implemented
}

fn _tclearregion(_x1: i32, _y1: i32, _x2: i32, _y2: i32) {
    // Function body to be implemented
}

fn _tcursor(_mode: i32) {
    // Function body to be implemented
}

fn _tdeletechar(_n: i32) {
    // Function body to be implemented
}

fn _tdeleteline(_n: i32) {
    // Function body to be implemented
}

fn _tinsertblank(_n: i32) {
    // Function body to be implemented
}

fn _tinsertblankline(_n: i32) {
    // Function body to be implemented
}

fn _tlinelen(_line: i32) -> i32 {
    // Function body to be implemented
    0
}

fn _tmoveto(_x: i32, _y: i32) {
    // Function body to be implemented
}

fn _tmoveato(_cx: i32, _cy: i32) {
    // Function body to be implemented
}

fn _tnewline(_first_col: i32) {
    // Function body to be implemented
}

fn _tputtab(_one_if_set: i32) {
    // Function body to be implemented
}

fn _tputc(_u: Rune) {
    // Function body to be implemented
}

fn _treset() {
    // Function body to be implemented
}

fn _tscrollup(_top: i32, _bot: i32) {
    // Function body to be implemented
}

fn _tscrolldown(_top: i32, _bot: i32) {
    // Function body to be implemented
}

fn _tsetattr(_attrs: &[i32], _l: i32) {
    // Function body to be implemented
}

fn _tsetchar(_u: Rune, _g: &Glyph, _x: i32, _y: i32) {
    // Function body to be implemented
}

fn _tsetdirt(_top: i32, _bot: i32) {
    // Function body to be implemented
}

fn _tsetscroll(_top: i32, _bot: i32) {
    // Function body to be implemented
}

fn _tswapscreen() {
    // Function body to be implemented
}

fn _tsetmode(_priv_: i32, _set: i32, _args: &[i32], _narg: i32) {
    // Function body to be implemented
}

fn _twrite(_s: *const char, _n: i32, _may_echo: i32) -> i32 {
    // Function body to be implemented
    0
}

fn _tfulldirt() {
    // Function body to be implemented
}

fn _tcontrolcode(_code: u8) {
    // Function body to be implemented
}

fn _tdectest(_c: char) {
    // Function body to be implemented
}

fn _tdefutf8(_mode: char) {
    // Function body to be implemented
}

fn _tdefcolor(_attrs: &[i32], _nattr: &mut i32, _l: i32) -> i32 {
    // Function body to be implemented
    0
}

fn _tdeftran(_c: char) {
    // Function body to be implemented
}

fn _tstrsequence(_c: u8) {
    // Function body to be implemented
}

fn _drawregion(_x1: i32, _y1: i32, _x2: i32, _y2: i32) {
    // Function body to be implemented
}

fn _selnormalize() {
    // Function body to be implemented
}

fn _selscroll(_orig: i32, _n: i32) {
    // Function body to be implemented
}

fn _selsnap(_x: &mut i32, _y: &mut i32, _direction: i32) {
    // Function body to be implemented
}

fn _utf8decode(_s: &str, _u: &mut Rune, _clen: usize) -> usize {
    // Function body to be implemented
    0
}

fn _utf8decodebyte(_c: char, _i: &mut usize) -> Rune {
    // Function body to be implemented
    0
}

fn _utf8encodebyte(_u: Rune, _i: usize) -> char {
    // Function body to be implemented
    '\0'
}

fn _utf8validate(_u: &mut Rune, _i: usize) -> usize {
    // Function body to be implemented
    0
}

fn _base64dec(_s: &str) -> String {
    // Function body to be implemented
    String::new()
}

fn _base64dec_getc(_s: &mut &str) -> char {
    // Function body to be implemented
    '\0'
}

fn xwrite(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    let mut len = buf.len();
    let mut written = 0;

    while len > 0 {
        match nix::unistd::write(fd, &buf[written..]) {
            Ok(r) => {
                written += r;
                len -= r;
            },
            Err(e) => {
                if e.as_errno().unwrap() == nix::errno::Errno::EINTR {
                    // Interrupted by a signal, try again
                    continue;
                }
                return Err(io::Error::from_raw_os_error(e as i32));
            }
        }
    }

    Ok(written)
}

fn main() {
    println!("Hello, world!");
}
