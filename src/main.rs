use std::io;
use std::os::unix::io::RawFd;
use std::fs::File;
use std::os::unix::io::FromRawFd;

mod win;
mod st;

use crate::st::Line;
use crate::st::Glyph;

// Constants from the C code
const UTF_INVALID: u32 = 0xFFFD;
const UTF_SIZ: usize = 4;
const ESC_BUF_SIZ: usize = 128 * UTF_SIZ;
const ESC_ARG_SIZ: usize = 16;

#[allow(dead_code)]
const STR_BUF_SIZ: usize = ESC_BUF_SIZ;

#[allow(dead_code)]
const STR_ARG_SIZ: usize = ESC_ARG_SIZ;

#[allow(dead_code)]
// Converted C macros to Rust functions or inline code
fn is_set(flag: u32, term_mode: u32) -> bool {
    (term_mode & flag) != 0
}

#[allow(dead_code)]
fn is_control_c0(c: char) -> bool {
    ('\0'..='\x1f').contains(&c) || c == '\x7f'
}

#[allow(dead_code)]
fn is_control_c1(c: char) -> bool {
    ('\u{80}'..='\u{9f}').contains(&c)
}

#[allow(dead_code)]
fn is_control(c: char) -> bool {
    is_control_c0(c) || is_control_c1(c)
}

#[allow(dead_code)]
fn is_delim(c: char, word_delimiters: &str) -> bool {
    c != '\0' && word_delimiters.contains(c)
}

// Allow dead code for now
#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorMovement {
    Save,
    Load,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorState {
    Default = 0,
    WrapNext = 1,
    Origin = 2,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
// Structs translated to Rust structs
struct TCursor {
    attr: st::Glyph, // current char attributes
    x: i32,          // terminal column
    y: i32,          // terminal row
    state: char,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
// Define the Coordinates struct used in Selection
struct Coordinates {
    x: i32,
    y: i32,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
// CSI Escape sequence struct
struct CSIEscape {
    buf: [char; ESC_BUF_SIZ], // raw string
    len: usize,               // raw string length
    priv_: char,
    arg: [i32; ESC_ARG_SIZ],
    narg: i32,                // nb of args
    mode: [char; 2],
}

#[allow(dead_code)]
// STR Escape sequence struct
struct STREscape {
    escape_type: char,            // ESC type
    buf: String,                  // Allocated raw string
    size: usize,                  // Allocation size
    length: usize,                // Raw string length
    args: Vec<String>,            // Arguments
    num_args: usize,              // Number of arguments
}

// Allow dead code for now
#[allow(dead_code)]
static mut IOFD: i32 = 1;
#[allow(dead_code)]
static mut CMDFD: i32 = 0;  // Initialized to a default value, adjust as needed
#[allow(dead_code)]
static mut PID: libc::pid_t = 0;  // Using libc crate for pid_t type


const UTFBYTE: [u8; UTF_SIZ + 1] = [0x80, 0, 0xC0, 0xE0, 0xF0];
const UTFMASK: [u8; UTF_SIZ + 1] = [0xC0, 0x80, 0xE0, 0xF0, 0xF8];
const UTFMIN: [Rune; UTF_SIZ + 1] = [0, 0, 0x80, 0x800, 0x10000];
const UTFMAX: [Rune; UTF_SIZ + 1] = [0x10FFFF, 0x7F, 0x7FF, 0xFFFF, 0x10FFFF];

// Assuming Rune is an alias for u32
type Rune = u32;


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

// Allow dead code for now
#[allow(dead_code)]
fn utf8_decode(c: &[u8], u: &mut u32) -> usize {
    *u = UTF_INVALID;

    if c.is_empty() {
        return 0;
    }

    let (mut udecoded, len) = utf8_decode_byte(c[0]);
    if !(1..=UTF_SIZ).contains(&len) {
        return 1;
    }

    for (i, &byte) in c.iter().enumerate().take(len).skip(1) {
        let (new_udecoded, type_) = utf8_decode_byte(byte);
        udecoded = (udecoded << 6) | new_udecoded;
        if type_ != 0 {
            return i;
        }
    }

    if c.len() < len {
        return 0;
    }

    *u = udecoded;
    utf8_validate(u, len);

    len
}

// Allow dead code for now
#[allow(dead_code)]
fn utf8_decode_byte(c: u8) -> (Rune, usize) {
    let utfmask: [u8; 4] = [0x80, 0xE0, 0xF0, 0xF8];
    let utfbyte: [u8; 4] = [0x00, 0xC0, 0xE0, 0xF0];

    for (i, (&mask, &byte)) in utfmask.iter().zip(utfbyte.iter()).enumerate() {
        if (c & mask) == byte {
            return ((c & !mask) as Rune, i);
        }
    }

    (0, 0)
}

// Allow dead code for now
#[allow(dead_code)]
fn utf8_encode(u: Rune) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut rune = u;
    let len = utf8_validate(&mut rune, 0);

    if len > UTF_SIZ {
        return encoded;
    }

    for _ in (1..len).rev() {
        encoded.push(utf8_encode_byte(rune, 0));
        rune >>= 6;
    }
    encoded.insert(0, utf8_encode_byte(rune, len));

    encoded
}

// Allow dead code for now
#[allow(dead_code)]
fn utf8_encode_byte(u: Rune, i: usize) -> u8 {
    UTFBYTE[i] | ((u as u8) & !UTFMASK[i])
}

// Allow dead code for now
#[allow(dead_code)]
fn utf8_validate(u: &mut Rune, mut i: usize) -> usize {
    if !((*u >= UTFMIN[i] && *u <= UTFMAX[i]) || (*u >= 0xD800 && *u <= 0xDFFF)) {
        *u = UTF_INVALID;
    }
    while i < UTFMAX.len() && *u > UTFMAX[i] {
        i += 1;
    }

    i
}

#[allow(dead_code)]
fn base64dec_getc(src: &mut std::str::Chars) -> char {
    while let Some(next_char) = src.clone().next() {
        if next_char.is_ascii_graphic() || next_char.is_whitespace() {
            break;
        }
        src.next();  // Advance the iterator if non-printable
    }
    src.next().unwrap_or('=')
}

fn _base64dec_getc(_s: &mut &str) -> char {
    // Function body to be implemented
    '\0'
}


// Allow dead code for now
#[allow(dead_code)]
fn xwrite(fd: RawFd, buf: &[u8]) -> io::Result<usize> {
    let mut len = buf.len();
    let mut written = 0;
    let file = unsafe { File::from_raw_fd(fd) };

    while len > 0 {
        match nix::unistd::write(&file, &buf[written..]) {
            Ok(r) => {
                written += r;
                len -= r;
            },
            Err(e) => {
                return Err(io::Error::from_raw_os_error(e as i32));
            }
        }
    }

    Ok(written)
}

fn main() {
    println!("Hello, world!");
}
