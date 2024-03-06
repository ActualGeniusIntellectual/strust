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

// Assuming relevant types and constants are defined elsewhere in your Rust code.
type Rune = u32; // Placeholder type, adjust as needed
struct Glyph; // Placeholder struct, define as needed

// Functions translated from C to Rust
fn execsh(cmd: *mut char, args: *mut *mut char) {
    // Function body to be implemented
}

fn stty(args: *mut *mut char) {
    // Function body to be implemented
}

fn sigchld(signum: i32) {
    // Function body to be implemented
}

fn tty_writeraw(s: *const char, len: usize) {
    // Function body to be implemented
}

fn csi_dump() {
    // Function body to be implemented
}

fn csi_handle() {
    // Function body to be implemented
}

fn csi_parse() {
    // Function body to be implemented
}

fn csi_reset() {
    // Function body to be implemented
}

fn osc_color_response(p1: i32, p2: i32, p3: i32) {
    // Function body to be implemented
}

fn eschandle(c: u8) -> i32 {
    // Function body to be implemented
    0
}

fn str_dump() {
    // Function body to be implemented
}

fn str_handle() {
    // Function body to be implemented
}

fn str_parse() {
    // Function body to be implemented
}

fn str_reset() {
    // Function body to be implemented
}

fn tprinter(s: *mut char, len: usize) {
    // Function body to be implemented
}

fn tdumpsel() {
    // Function body to be implemented
}

fn tdumpline(line: i32) {
    // Function body to be implemented
}

fn tdump() {
    // Function body to be implemented
}

fn tclearregion(x1: i32, y1: i32, x2: i32, y2: i32) {
    // Function body to be implemented
}

fn tcursor(mode: i32) {
    // Function body to be implemented
}

fn tdeletechar(n: i32) {
    // Function body to be implemented
}

fn tdeleteline(n: i32) {
    // Function body to be implemented
}

fn tinsertblank(n: i32) {
    // Function body to be implemented
}

fn tinsertblankline(n: i32) {
    // Function body to be implemented
}

fn tlinelen(line: i32) -> i32 {
    // Function body to be implemented
    0
}

fn tmoveto(x: i32, y: i32) {
    // Function body to be implemented
}

fn tmoveato(cx: i32, cy: i32) {
    // Function body to be implemented
}

fn tnewline(first_col: i32) {
    // Function body to be implemented
}

fn tputtab(one_if_set: i32) {
    // Function body to be implemented
}

fn tputc(u: Rune) {
    // Function body to be implemented
}

fn treset() {
    // Function body to be implemented
}

fn tscrollup(top: i32, bot: i32) {
    // Function body to be implemented
}

fn tscrolldown(top: i32, bot: i32) {
    // Function body to be implemented
}

fn tsetattr(attrs: &[i32], l: i32) {
    // Function body to be implemented
}

fn tsetchar(u: Rune, g: &Glyph, x: i32, y: i32) {
    // Function body to be implemented
}

fn tsetdirt(top: i32, bot: i32) {
    // Function body to be implemented
}

fn tsetscroll(top: i32, bot: i32) {
    // Function body to be implemented
}

fn tswapscreen() {
    // Function body to be implemented
}

fn tsetmode(priv_: i32, set: i32, args: &[i32], narg: i32) {
    // Function body to be implemented
}

fn twrite(s: *const char, n: i32, may_echo: i32) -> i32 {
    // Function body to be implemented
    0
}

fn tfulldirt() {
    // Function body to be implemented
}

fn tcontrolcode(code: u8) {
    // Function body to be implemented
}

fn tdectest(c: char) {
    // Function body to be implemented
}

fn tdefutf8(mode: char) {
    // Function body to be implemented
}

fn tdefcolor(attrs: &[i32], nattr: &mut i32, l: i32) -> i32 {
    // Function body to be implemented
    0
}

fn tdeftran(c: char) {
    // Function body to be implemented
}

fn tstrsequence(c: u8) {
    // Function body to be implemented
}

fn drawregion(x1: i32, y1: i32, x2: i32, y2: i32) {
    // Function body to be implemented
}

fn selnormalize() {
    // Function body to be implemented
}

fn selscroll(orig: i32, n: i32) {
    // Function body to be implemented
}

fn selsnap(x: &mut i32, y: &mut i32, direction: i32) {
    // Function body to be implemented
}

fn utf8decode(s: &str, u: &mut Rune, clen: usize) -> usize {
    // Function body to be implemented
    0
}

fn utf8decodebyte(c: char, i: &mut usize) -> Rune {
    // Function body to be implemented
    0
}

fn utf8encodebyte(u: Rune, i: usize) -> char {
    // Function body to be implemented
    '\0'
}

fn utf8validate(u: &mut Rune, i: usize) -> usize {
    // Function body to be implemented
    0
}

fn base64dec(s: &str) -> String {
    // Function body to be implemented
    String::new()
}

fn base64dec_getc(s: &mut &str) -> char {
    // Function body to be implemented
    '\0'
}

fn xwrite(fd: i32, buf: &[u8], size: usize) -> isize {
    // Function body to be implemented
    0
}

fn main() {
    println!("Hello, world!");
}
