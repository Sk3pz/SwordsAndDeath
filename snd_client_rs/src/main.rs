mod ui;

use std::io::{stdout, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use better_term::{flush_styles, read_input, yesno_prompt};
use crossterm::event::{Event as CEvent, KeyCode};
use crossterm::{event, execute};
use crossterm::terminal::{ClearType, disable_raw_mode, enable_raw_mode};
use regex::Regex;
use rpassword::read_password;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Style;
use tui::Terminal;
use tui::text::Span;
use tui::widgets::{Block, Borders, BorderType, Paragraph};
use snd_network_lib::client_event::{write_client_disconnect, write_client_drop_item, write_client_inspect_item, write_client_keepalive, write_client_open_inv, write_client_step};
use snd_network_lib::encounter_data::EncounterData;
use snd_network_lib::enemy_data::EnemyData;
use snd_network_lib::entry_point_io::{write_entry_login_attempt, write_entry_point_ver};
use snd_network_lib::entry_response::read_entry_response;
use snd_network_lib::item_data::ItemData;
use snd_network_lib::login_data::LoginData;
use snd_network_lib::player_data::PlayerData;
use snd_network_lib::server_event::{read_server_event, ServerEvent};
use crate::ui::{draw_home, Event};

fn get_ip() -> String {
    let ip_pattern =
        Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$")
            .expect("Failed to init regex");
    let port_pattern =
        Regex::new(r"^((6553[0-5])|(655[0-2][0-9])|(65[0-4][0-9]{2})|(6[0-4][0-9]{3})|([1-5][0-9]{4})|([0-5]{0,5})|([0-9]{1,4}))$")
            .expect("Failed to init regex");

    let mut ip = read_input!("Input the ip of the server: ");
    let mut port = read_input!("Input the port of the server: ");

    while !ip_pattern.is_match(ip.as_str()) {
        eprintln!("Invalid ip! please enter a valid ip!");
        ip = read_input!("Input the ip of the server: ");
    }

    while !port_pattern.is_match(port.as_str()) {
        eprintln!("Invalid entry for the connection port. Please enter the port ranging from 0 to 65535");
        port = read_input!("Input the port of the server: ");
    }

    format!("{}:{}", ip, port)
}

fn get_login() -> LoginData {
    println!("Login:");
    let signup = yesno_prompt!("Are you signing up?");
    let username = read_input!("Username: ");
    print!("Password: ");
    stdout().flush().expect("failed to flush stdout!");
    let passwd = read_password().expect("Failed to get password");
    LoginData {
        signup, username, passwd, client_ver: VERSION.to_string()
    }
}

fn print_item(id: &ItemData) {
    println!("==={}===\
            \nRarity: {}\
            \nLevel:  {}\
            \n{}\
            \n{}",
             id.name, id.rarity, id.level,
             if id.defense.is_some() { format!("Defense: {}", id.defense.unwrap()) }
             else { format!("Damage: {}", id.damage.unwrap()) },
             "=".repeat(6 + id.name.len()));
}

fn print_enemy(ed: &EnemyData) {
    println!("==={}===\
            \nHealth: {}\
            \nRace:   {}\
            \nLevel:  {}\
            \n{}",
             ed.name, ed.health, ed.race, ed.level,
             "=".repeat(6 + ed.name.len()));
}

struct BuffWrapper<T> {
    wrapped: T
}

impl<T> BuffWrapper<T> {
    pub fn new(content: T) -> Self {
        Self {
            wrapped: content
        }
    }

    pub fn set(&mut self, new: T) {
        self.wrapped = new;
    }

    pub fn get(&self) -> &T {
        &self.wrapped
    }
}

pub(crate) struct Output<const N: usize = 5> {
    lines: [String;N],
}

impl Output {
    pub fn new() -> Self {
        Self {
            lines: Default::default()
        }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        if self.lines.len() <= index {
            return None;
        }
        Some(&self.lines[index])
    }

    pub fn set<S: Into<String>>(&mut self, index: usize, value: S) {
        if index >= self.lines.len() {
            return;
        }
        self.lines[index] = value.into();
    }

    pub fn one<S: Into<String>>(&mut self, value: S) {
        let v = value.into();
        for x in 0..self.lines.len() {
            if x == self.lines.len() / 2 {
                self.lines[x] = v.clone();
                continue;
            }
            self.lines[x] = "".to_string();
        }
    }

    pub fn append<S: Into<String>>(&mut self, index: usize, app: S) {
        if self.lines.len() <= index {
            return;
        }
        self.lines[index].push_str(app.into().as_str());
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    //let ip = get_ip();
    let ip = format!("127.0.0.1:2277");

    // ping loop
    loop {
        // get the connection
        let ping_stream = TcpStream::connect(ip.clone());
        if ping_stream.is_err() {
            eprintln!("Failed to connect to server.");
            continue;
        }

        // validate the connection
        let ps = ping_stream.unwrap();

        // write version to server for ping
        if let Err(e) = write_entry_point_ver(&ps, VERSION.to_string()) {
            eprintln!("Failed to write ping to server to check version. Error: {}", e);
            return;
        }

        // read response
        let (_, version, error) = read_entry_response(&ps);

        if let Some(err) = error {
            eprintln!("{}", err);
            return;
        }

        if version.is_none() {
            eprintln!("Unknown issue occurred getting version from the server.");
            return;
        }

        // if it was an error, print the message and see if the user wants to continue or exit
        if error.is_some() {
            eprintln!("Error from the server: {}", error.unwrap());
        }

        drop(ps);
        break;
    }

    // get login data
    // todo(eric): have a system for inputting login information
    // todo(eric): have a system to cache login data
    let login = get_login();

    // connect to server and send login data
    let stream_res = TcpStream::connect(ip);
    if let Err(e) = stream_res {
        eprintln!("Failed to connect to server to login! Error: {}", e);
        return;
    }
    let stream = stream_res.unwrap();

    if let Err(e) = write_entry_login_attempt(&stream, login) {
        eprintln!("Failed to write a login attempt to the server: {}", e);
        return;
    }

    let (motd, _, error) = read_entry_response(&stream);

    if error.is_some() {
        eprintln!("Error from server: {}", error.unwrap());
        return;
    }

    if motd.is_none() {
        eprintln!("unexpected error: invalid response from server");
        return;
    }

    // todo(eric): get player update

    // initialize the terminal for the UI
    enable_raw_mode().expect("Failed to enable raw mode; is this terminal supported?");

    let mut stdout = stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen, event::EnableMouseCapture).expect("Failed to setup terminal; Is this terminal supported?");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to setup terminal; Is this terminal supported?");
    terminal.clear().expect("Failed to clear the terminal");

    let terminate = Arc::new(AtomicBool::new(false));

    // Initialize the event loop for the UI
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // input handling thread
    let aterm = Arc::clone(&terminate);
    let handler = thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            if aterm.load(Ordering::SeqCst) {
                break;
            }
            let timeout = tick_rate.checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Failed to poll events") {
                if let CEvent::Key(key) = event::read().expect("Failed to read events.") {
                    tx.send(Event::Input(key)).expect("Failed to send event to main thread");
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).expect("Failed to send tick update");
                    last_tick = Instant::now();
                }
            }
        }
    });

    let output = Arc::new(Mutex::new(Output::new()));
    let mut ending_output = Arc::new(Mutex::new(BuffWrapper::new(String::new())));

    let mut encounter: Arc<Mutex<BuffWrapper<Option<EncounterData>>>> = Arc::new(Mutex::new(BuffWrapper::new(None)));

    let tarc = Arc::clone(&terminate);
    let op = Arc::clone(&output);
    let enctr = Arc::clone(&encounter);
    let eop = Arc::clone(&ending_output);
    let stream2 = stream.try_clone().expect("Failed to clone stream for server handler");
    let handler2 = thread::spawn(move || {
        loop {
            if tarc.load(Ordering::SeqCst) {
                break;
            }
            let server_event = read_server_event(&stream2);
            match server_event {
                ServerEvent::Disconnect => {
                    break;
                }
                ServerEvent::Keepalive(_) => {
                    if let Err(e) = write_client_keepalive(&stream2) {
                        eop.lock().unwrap().set(format!("Error writing keepalive: {}", e));
                        break;
                    }
                }
                ServerEvent::Event(s) => {
                    (op.lock().unwrap()).one(format!("{}", s));
                }
                ServerEvent::GainExp(amt) => {
                    (op.lock().unwrap()).one(format!("You gained {} exp!", amt));
                    flush_styles();
                }
                ServerEvent::FindItem(id) => {
                    (op.lock().unwrap()).set(0, "You found an item!");
                    (op.lock().unwrap()).set(1, format!("Name:   {}", id.name));
                    let itype = match id.itype {
                        1 => "Shield",
                        2 => "Helmet",
                        3 => "Chestplate",
                        4 => "Leggings",
                        5 => "Boots",
                        _ => "Sword",
                    };
                    let rarity = match id.rarity {
                        1 => "Rare",
                        2 => "Epic",
                        3 => "Legendary",
                        _ => "Common",
                    };
                    (op.lock().unwrap()).set(2, format!("Type:   {}", itype));
                    (op.lock().unwrap()).set(3, format!("Rarity: {}", rarity));
                    (op.lock().unwrap()).set(4, format!("Enter 'inspect {}' to view more about this item!", id.name));
                }
                ServerEvent::Update(pd) => {
                    // todo(eric)
                }
                ServerEvent::ItemView(id) => {
                    let itype = match id.itype {
                        1 => "Shield",
                        2 => "Helmet",
                        3 => "Chestplate",
                        4 => "Leggings",
                        5 => "Boots",
                        _ => "Sword",
                    };
                    let rarity = match id.rarity {
                        1 => "Rare",
                        2 => "Epic",
                        3 => "Legendary",
                        _ => "Common",
                    };
                    (op.lock().unwrap()).set(0, format!("Name:   {}", id.name));
                    (op.lock().unwrap()).set(1, format!("Type:   {}", itype));
                    (op.lock().unwrap()).set(2, format!("Level:  {}", id.level));
                    (op.lock().unwrap()).set(3, format!("Rarity: {}", rarity));
                    if id.defense.is_some() {
                        (op.lock().unwrap()).set(4, format!("Defense: {}", id.defense.unwrap()));
                    } else {
                        (op.lock().unwrap()).set(4, format!("Damage: {}", id.damage.unwrap()));
                    }
                }
                ServerEvent::Inventory(items) => {
                    (op.lock().unwrap()).set(0, "INVENTORY | To view an item, enter 'inspect <item name>'");
                    (op.lock().unwrap()).set(1, "");
                    (op.lock().unwrap()).set(2, if items.is_empty() { "Your inventory is empty" } else { "" });
                    (op.lock().unwrap()).set(3, "");
                    (op.lock().unwrap()).set(4, "");
                    let line_size = items.len() / 4;
                    let mut line = 1;
                    for x in 0..items.len() {
                        let current = items.get(x).unwrap();
                        let mut l = format!("'{}'", current.name);
                        if x != items.len() - 1 {
                            l.push_str(", ");

                        }
                        (op.lock().unwrap()).set(line, format!("{}", l));
                        if x == line_size * line { line = (line + 1).min(4) }
                    }
                }
                ServerEvent::Encounter(ed) => {
                    // todo(eric): handle different encounter actions that are received here.
                    (op.lock().unwrap()).set(0, "You encountered an enemy!");
                    (op.lock().unwrap()).set(1, format!("Name: {}", ed.enemy.name));
                    (op.lock().unwrap()).set(2, format!("Level: {}", ed.enemy.level));
                    (op.lock().unwrap()).set(3, format!("Race: {}", ed.enemy.race));
                    (op.lock().unwrap()).set(4, format!("Health: {}", ed.enemy.health));
                    enctr.lock().unwrap().set(Some(ed));
                }
                ServerEvent::Error(ed) => {
                    (op.lock().unwrap()).one(format!("Error from the server: {}", ed.msg));
                    if ed.disconnect {
                        break;
                    }
                }
            }
        }
        tarc.store(true, Ordering::SeqCst);
    });

    let mut input_mode = false;
    let mut input_ready = false;
    let mut user_input = String::new();

    loop {
        if terminate.load(Ordering::SeqCst) {
            break;
        }
        // draw the UI
        let footer = Paragraph::new(format!("Swords and Death v{} by Eric Shreve", VERSION))
            .style(Style::default().fg(tui::style::Color::Red))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(tui::style::Color::White))
                    .title("Info")
                    .border_type(BorderType::Plain)
            );
        terminal.draw(|mut rect| {
            // setup the layout
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Min(2),
                    Constraint::Length(3),
                ].as_ref())
                .split(size);

            // handle the main page
            draw_home(&mut rect, &chunks, &output);

            if input_mode {
                let input = Paragraph::new(user_input.clone())
                    .style(Style::default().fg(tui::style::Color::Gray))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(tui::style::Color::White))
                            .title("Input")
                            .border_type(BorderType::Double)
                    );
                rect.set_cursor(
                    chunks[1].x + user_input.len() as u16 + 1,
                    chunks[1].y + 1,
                );
                rect.render_widget(input, chunks[1]);
            }
            if !input_mode {
                rect.render_widget(footer, chunks[1]);
            }
        }).expect("Failed to draw frame with TUI");

        // handle keypresses for the UI
        let event_poll = rx.recv_timeout(Duration::from_millis(200));
        if event_poll.is_ok() {
            match event_poll.unwrap() {
                Event::Input(event) => {
                    if event.code == KeyCode::Char('c') {
                        if event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            // add ctrl+c functionality
                            // if the program is processing for a long time this wont complete until it's done processing
                            break;
                        }
                    }
                    if input_mode {
                        match event.code {
                            KeyCode::Char(c) => {
                                user_input.push(c);
                            }
                            KeyCode::Enter => {
                                input_ready = true;
                                input_mode = !input_mode;
                            }
                            KeyCode::Esc => {
                                input_mode = !input_mode;
                            }
                            KeyCode::Backspace => {
                                if user_input.len() > 0 {
                                    user_input.remove(user_input.len() - 1);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match event.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Enter => input_mode = true,
                            _ => {}
                        }
                    }
                }
                Event::Tick => {}
            }
        }

        // handle input
        if input_ready {
            let mut args = user_input.split(" ").collect::<Vec<&str>>();
            let cmd = args.remove(0);
            match cmd.to_ascii_lowercase().as_str() {
                "step" => {
                    if let Err(e) = write_client_step(&stream) {
                        ending_output.lock().unwrap().set(format!("Failed to send packet to server: {}", e));
                        return;
                    }
                }
                "inv" => {
                    if let Err(e) = write_client_open_inv(&stream) {
                        ending_output.lock().unwrap().set(format!("Failed to send packet to server: {}", e));
                        return;
                    }
                }
                "drop" => {
                    // make sure there is an item name specified
                    if args.len() < 1 {
                        (output.lock().unwrap()).one("You must specify which item to drop! 'drop <item name>'");
                        input_ready = false;
                        user_input.clear();
                        continue;
                    }
                    // get the item name
                    let item = args.join(" ");
                    // ensure the user wants to drop the item
                    if let Err(e) = write_client_drop_item(&stream, item) {
                        ending_output.lock().unwrap().set(format!("Failed to send packet to server: {}", e));
                        return;
                    }
                }
                "inspect" => {
                    // make sure there is an item name specified
                    if args.len() < 1 {
                        (output.lock().unwrap()).one("You must specify which item to drop! 'drop <item name>'");
                        input_ready = false;
                        user_input.clear();
                        continue;
                    }
                    // get the item name
                    let item = args.join(" ");
                    if let Err(e) = write_client_inspect_item(&stream, item) {
                        ending_output.lock().unwrap().set(format!("Failed to send packet to server: {}", e));
                        return;
                    }
                }
                _ => {
                    if !user_input.is_empty() {
                        (output.lock().unwrap()).one("Invalid Action!");
                    }
                }
            }
            input_ready = false;
            user_input.clear();
            continue;
        }
    }

    let _ = write_client_disconnect(&stream);

    // restore terminal
    disable_raw_mode().expect("Failed to restore terminal");
    terminal.clear().expect("Failed to restore terminal");
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        event::DisableMouseCapture,
        crossterm::cursor::MoveTo(0,0),
        crossterm::terminal::Clear(ClearType::All)
    ).expect("Failed to restore terminal");
    terminal.show_cursor().expect("Failed to restore terminal");
    terminate.store(true, Ordering::SeqCst);
    handler.join();
    handler2.join();
    println!("{}", ending_output.lock().unwrap().get());
}
