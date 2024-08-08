use libc;
use std::{
  io::{self, Read, Write, IsTerminal},
  sync::Mutex,
  process::{Stdio, Command},
  os::unix::process::CommandExt,
  str,
};

static termios: Mutex<libc::termios> = Mutex::new(libc::termios { c_iflag: 0, c_oflag: 0, c_cflag: 0, c_lflag: 0, c_line: 1, c_cc: [0 as u8; 32], c_ispeed: 1, c_ospeed: 1 });


fn setup_termios() {
  termios.lock().unwrap().c_cflag &= !libc::CSIZE;
  termios.lock().unwrap().c_cflag |= libc::CS8;
  termios.lock().unwrap().c_cc[libc::VMIN] = 1;
}

extern "C" fn disable_raw_mode() {
  unsafe {
    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &(*termios.lock().unwrap()));
  }
}
fn enable_raw_mode() {
  unsafe {
    libc::tcgetattr(libc::STDIN_FILENO, &mut *termios.lock().unwrap());
    libc::atexit(disable_raw_mode);
    let mut raw = *termios.lock().unwrap();
    raw.c_lflag &= !(libc::ECHO | libc::ICANON);
    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw);
  }
}




const enter: char = '\n';
const escape: char = 27 as char;

const SELECTED_COLOR: i32 = 178;
const INACTIVE_COLOR: i32 = 253;
//const SELECTED_BACKGROUND

#[derive(Debug,PartialEq)]
pub struct KeyEvent {
  pub code: KeyCode,
}

#[derive(Debug,PartialEq)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}
#[derive(Debug,PartialEq)]
pub enum KeyCode {
  Escape,
  Enter,
  Arrow(Direction),
  Char(char),
}

fn parse_event(event: KeyEvent, program: &mut Program) {
  match event.code {
    KeyCode::Escape => {
      program.exit = true;
    },
    KeyCode::Enter => {
      println!("{:#?}", Command::new("sh").arg("-c").arg(&format!("{}", program.fokprograms[program.selected_index].runCommand)).exec());
    },
    KeyCode::Arrow(d) => {
      match d {
        Direction::Up => {
          program.fokprograms[program.selected_index].selected = false;
          if program.selected_index as i32 -1 < 0 {
            program.selected_index = program.fokprograms.len()-1;
          } else {
            program.selected_index -= 1;
          }
          program.fokprograms[program.selected_index].selected = true;
        },
        Direction::Down => {
          program.fokprograms[program.selected_index].selected = false;
          if program.selected_index as i32 +1 == program.fokprograms.len() as i32 {
            program.selected_index = 0;
          } else {
            program.selected_index += 1;
          }
          program.fokprograms[program.selected_index].selected = true;
        },
        Direction::Right => {
          println!("{:#?}", Command::new("sh").arg("-c").arg(&format!("{}", program.fokprograms[program.selected_index].runCommand)).exec())
        },
        _ => {}
      }
    },
    KeyCode::Char(c) => {
      match c {
        'q' => {program.exit=true;},
        _ => {},
      }
    },
  }
}

fn getch() -> char {
  io::stdin().bytes().next().unwrap().unwrap() as char
}


fn redraw(program: Program) {
  let mut max = 0;
  for i in program.clone().fokprograms {
    if max<i.name.len() {
      max = i.name.len();
    }
  }
  max*=2;
  print!("{esc}[0G", esc = 27 as char);
  print!("{esc}[2K", esc = 27 as char);
  for i in 0..program.fokprograms.len() {
    print!("{esc}[1A", esc = 27 as char);
    print!("{esc}[2K", esc = 27 as char);
  }
  let mut output = String::new();
  for i in program.fokprograms {
    if i.enabled {
      if i.selected {
        output += &format!("{esc}[38;5;{SELECTED_COLOR}m{line}{}{line}{esc}[0m", i.name, esc = 27 as char, line=vec!['-'; ((max - i.name.len())/2) as usize].into_iter().collect::<String>());
      } else {
        output += &format!("{esc}[38;5;{INACTIVE_COLOR}m{space}{}{space}{esc}[0m", i.name, esc = 27 as char, space=vec![' '; ((max-i.name.len())/2) as usize].into_iter().collect::<String>());
      }
      output += "\n";
    }
  }
  print!("{}", output);
}

#[derive(Debug,Clone)]
struct FokProgram {
  name: String,
  runCommand: String,
  enabled: bool,
  selected: bool,
}

#[derive(Debug,Clone)]
struct Program {
  fokprograms: Vec<FokProgram>,
  selected_index: usize,
  exit: bool,
}
fn main_loop(mut program: Program) {
  
  redraw(program.clone());
  for b in io::stdin().bytes() {
    let c = b.unwrap() as char;
    //let event = KeyEvent {
    //  code: match c {  }
    //};
    let event = KeyEvent {
      code: match c {
        enter => KeyCode::Enter,
        escape => match getch() { '[' => KeyCode::Arrow(match getch() {'A' => Direction::Up, 'B' => Direction::Down, 'C' => Direction::Right, 'D' => Direction::Left, _ => panic!("don't feed me ansi sequences")}),_ => KeyCode::Escape},
        _ => KeyCode::Char(c),
      }
    };
    //println!("{:#?}", event);
    parse_event(event, &mut program);
    redraw(program.clone());
    if program.exit {
      break;
    }
  }
}

fn main() {
  let mut program = Program {fokprograms: vec![], selected_index: 0, exit: false};
  program.fokprograms.push(FokProgram {name: String::from("Chess"), runCommand: String::from("chess"), enabled: true, selected: true});
  program.fokprograms.push(FokProgram {name: String::from("Fok-Quote"), runCommand: String::from("printf \"Quote: \"; read QUOTE;printf \"Author: \";read AUTHOR;fok-quote \"$QUOTE\" \"$AUTHOR\";read NULL"), enabled: true, selected: false});

  for mut i in 0..program.fokprograms.len() {
    program.fokprograms[i].enabled = str::from_utf8(&Command::new("sh").arg("-c").arg(&format!("command -v {}", program.fokprograms[i].name.to_lowercase())).output().unwrap().stdout).unwrap().to_string() != String::new();
    print!("\n");
    //println!("{}", program.fokprograms[i].enabled);
  }

  setup_termios();
  enable_raw_mode();
  main_loop(program);
  disable_raw_mode();
}



