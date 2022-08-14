use std::{
    io,
    panic,
    sync::{
        Arc,
        Mutex,
    },
    thread,
    time::Duration,
};

use crossterm::{
    event::{
        DisableMouseCapture,
        EnableMouseCapture,
    },
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use tui::{
    backend::CrosstermBackend,
    widgets::{
        Block,
        Borders,
    },
    Terminal,
};

#[rustfmt::skip]
const MPF_SMALL_ARMS: &[Item] = &[
    Item::new("Booker Storm Rifle Model 838",       None,                       904,    0,      0,      0),
    Item::new("Aalto Storm Rifle 24",               None,                       904,    0,      0,      0),
    Item::new("7.92mm",                             None,                       660,    0,      0,      0),
    Item::new("Malone MK.2",                        None,                       0,      134,    0,      0),
    Item::new("A3 Harpa Fragmentation Grenade",     Some("Harpa"),              550,    0,      110,    0),
    Item::new_useless("Cascadier 837",              None,                       330,    0,      0,      0),
    Item::new_useless("8mm",                        None,                       220,    0,      0,      0),
    Item::new("Cometa T2-9",                        Some("Revolver"),           330,    0,      0,      0),
    Item::new("The Hangman 757",                    Some("Hangman"),            684,    0,      0,      0),
    Item::new("0.44",                               None,                       220,    0,      0,      0),
    Item::new("Sampo Auto-Rifle 77",                Some("Sampo"),              684,    0,      0,      0),
    Item::new("Blakerow 871",                       Some("Blakerow"),           700,    0,      0,      0),
    Item::new("Clancy Cinder M3",                   Some("Clancy Cinder"),      715,    0,      0,      0),
    Item::new("No.2 Loughcaster",                   Some("Loughcaster"),        550,    0,      0,      0),
    Item::new("Clancy-Raca M4",                     Some("Clancy-Raca"),        1100,   0,      79,     0),
    Item::new("7.62",                               None,                       440,    0,      0,      0),
    Item::new("Brasa Shotgun",                      Some("Shotgun"),            440,    0,      0,      0),
    Item::new("Buckshot",                           None,                       440,    0,      0,      0),
    Item::new(r#"No.1 "The Liar" Submachinegun"#,   Some("The Liar"),           660,    0,      0,      0),
    Item::new("Fiddler Submachine Gun Model 868",   Some("Fiddler"),            660,    0,      0,      0),
    Item::new("9mm",                                None,                       440,    0,      0,      0),
    Item::new("PT-815 Smoke Grenade",               Some("Smoke Grenade"),      660,    0,      0,      0),
    Item::new("Green Ash Grenade",                  Some("Green Ash"),          770,    0,      0,      0),
    Item::new("12.7mm",                             None,                       550,    0,      0,      0),
];

#[rustfmt::skip]
const MPF_HEAVY_ARMS: &[Item] = &[
    Item::new("BF5 White Ash Flask Grenade",        Some("White Ash"),          550,    220,    0,      0),
    Item::new("135 Neville Anti-Tank Rifle",        Some("Anti-Tank Rifle"),    825,    0,      0,      0),
    Item::new("20mm",                               None,                       550,    0,      0,      0),
    Item::new("Mounted Bonesaw MK.3",               Some("Mounted Bonesaw"),    550,    0,      24,     0),
    Item::new("Bonesaw MK.3",                       Some("Bonesaw"),            550,    0,      134,    0),
    Item::new("ARC/RPG",                            None,                       330,    409,    0,      0),
    Item::new("Tremola Grenade GPb-1",              Some("Tremola"),            825,    55,     0,      0),
    Item::new("Malone Ratcheter MK.1",              Some("Malone Ratcheter"),   550,    0,      24,     0),
    Item::new("30mm",                               None,                       440,    0,      110,    0),
    Item::new("Cremari Mortar",                     None,                       550,    0,      134,    0),
    Item::new("Mortar Flare Shell",                 None,                       330,    55,     0,      0),
    Item::new("Mortar Shrapnel Shell",              None,                       330,    79,     0,      0),
    Item::new("Mortar Shell",                       None,                       330,    189,    0,      0),
    Item::new("Mammon 91-b",                        Some("Mammon"),             550,    55,     0,      0),
    Item::new("Anti-Tank Sticky Bomb",              Some("Sticky Bomb"),        275,    275,    0,      0),
    Item::new("Cutler Foebreaker",                  Some("Foebreaker"),         550,    0,      24,     0),
    Item::new("RPG Shell",                          None,                       330,    244,    0,      0),
];

#[rustfmt::skip]
const MPF_HEAVY_AMMUNITION: &[Item] = &[
    Item::new("150mm",                              None,                       660,    0,      0,      55),
    Item::new("120mm",                              None,                       330,    0,      0,      0),
    Item::new("250mm",                              None,                       660,    0,      0,      134),
    Item::new("68mm",                               None,                       660,    660,    0,      0),
    Item::new("40mm",                               None,                       880,    660,    0,      0),
];

#[rustfmt::skip]
const MPF_UNIFORMS: &[Item] = &[
    Item::new("Specialist's Overcoat",              None,                       550,    0,      0,      0),
    Item::new("Gunner's Breastplate",               None,                       550,    0,      0,      0),
    Item::new("Sapper Gear",                        None,                       550,    0,      0,      0),
    Item::new("Physician's Jacket",                 None,                       550,    0,      0,      0),
    Item::new("Officer's Regalia",                  None,                       550,    0,      0,      0),
    Item::new("Outrider's Mantle",                  None,                       550,    0,      0,      0),
    Item::new("Caovish Parka",                      None,                       550,    0,      0,      0),
    Item::new("Padded Boiler Suit",                 None,                       550,    0,      0,      0),
];

fn main() {
    let panic_infos = Arc::new(Mutex::new(Vec::new()));
    panic::set_hook({
        let panic_infos = panic_infos.clone();
        Box::new(move |info| {
            panic_infos.lock().unwrap().push((
                info.payload()
                    .downcast_ref::<&str>()
                    .map(ToString::to_string),
                info.location()
                    .map(|location| (location.file().to_owned(), location.line())),
            ));
        })
    });

    crossterm::terminal::enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, size);
        })
        .unwrap();

    // thread::sleep(Duration::from_millis(5000));

    crossterm::terminal::disable_raw_mode().unwrap();
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    if let Err(err) = panic::catch_unwind(|| panic!("nice")) {
        for (message, location) in panic_infos.lock().unwrap().iter() {
            if let Some((file, line)) = location {
                eprintln!(
                    "panic at [{}:{}]: {}",
                    file,
                    line,
                    message
                        .clone()
                        .unwrap_or_else(|| "<no message>".to_string())
                );
            }
        }
        panic::resume_unwind(err);
    }
}

struct Item {
    name: &'static str,
    short_name: Option<&'static str>,
    bmats: u32,
    emats: u32,
    rmats: u32,
    hemats: u32,
    useless: bool,
}

impl Item {
    const fn new(
        name: &'static str,
        short_name: Option<&'static str>,
        bmats: u32,
        emats: u32,
        rmats: u32,
        hemats: u32,
    ) -> Self {
        Self {
            name,
            short_name,
            bmats,
            emats,
            rmats,
            hemats,
            useless: false,
        }
    }

    const fn new_useless(
        name: &'static str,
        short_name: Option<&'static str>,
        bmats: u32,
        emats: u32,
        rmats: u32,
        hemats: u32,
    ) -> Self {
        Self {
            name,
            short_name,
            bmats,
            emats,
            rmats,
            hemats,
            useless: true,
        }
    }
}
