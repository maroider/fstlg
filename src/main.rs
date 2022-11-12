#![feature(strict_provenance)]

use std::{
    fmt::Write,
    io,
    iter,
    panic,
    sync::{
        Arc,
        Mutex,
    },
    time::{
        Duration,
        Instant,
    },
};

use crossterm::{
    event::{
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode,
    },
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use tui::{
    backend::{
        Backend,
        CrosstermBackend,
    },
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    style::{
        Color,
        Modifier,
        Style,
    },
    widgets::{
        Block,
        Borders,
        List,
        ListItem,
        ListState,
        Paragraph,
    },
    Frame,
    Terminal,
};

#[rustfmt::skip]
const MPF_SMALL_ARMS: &[Item] = &[
    Item::new("Booker Storm Rifle Model 838",       None,                       904,    0,      0,      0,  W),
    Item::new("Aalto Storm Rifle 24",               None,                       904,    0,      0,      0,  W),
    Item::new(r#""Dusk" ce.III"#,                    None,                       904,    0,      0,      0,  C),
    Item::new("7.92mm",                             None,                       660,    0,      0,      0,  None),
    Item::new("Malone MK.2",                        None,                       0,      0,      134,    0,  W),
    Item::new("A3 Harpa Fragmentation Grenade",     Some("Harpa"),              550,    110,    0,      0,  W),
    Item::new_useless("Cascadier 837",              None,                       330,    0,      0,      0,  W),
    Item::new("Catara mo.II",                       Some("Catara"),             904,    0,      0,      0,  C),
    Item::new("KRN886-127 Gast Machine Gun",        Some("Gast"),               0,      0,      134,    0,  C),
    Item::new("Bombastone Grenade",                 Some("Bombastone"),         550,    110,    0,      0,  C),
    Item::new_useless("8mm",                        None,                       220,    0,      0,      0,  None),
    Item::new("Cometa T2-9",                        Some("Revolver"),           330,    0,      0,      0,  None),
    Item::new("The Hangman 757",                    Some("Hangman"),            684,    0,      0,      0,  W),
    Item::new("0.44",                               None,                       220,    0,      0,      0,  None),
    Item::new("Sampo Auto-Rifle 77",                Some("Sampo"),              684,    0,      0,      0,  W),
    Item::new("Blakerow 871",                       Some("Blakerow"),           770,    0,      0,      0,  W),
    Item::new("Clancy Cinder M3",                   Some("Clancy Cinder"),      715,    0,      0,      0,  W),
    Item::new("No.2 Loughcaster",                   Some("Loughcaster"),        550,    0,      0,      0,  W),
    Item::new("Clancy-Raca M4",                     Some("Clancy-Raca"),        1100,   0,      79,     0,  W),
    Item::new("Argenti r.II Rifle",                 Some("Argenti"),            550,    0,      0,      0,  C),
    Item::new("Volta r.I Repeater",                 Some("Volta"),              550,    0,      0,      0,  C),
    Item::new("Fuscina pi.I",                       Some("Fuscina"),            770,    0,      0,      0,  C),
    Item::new("KRR2-790 Omen",                      Some("Omen"),               849,    0,      0,      0,  C),
    Item::new("KRR3-792 Auger",                     Some("Auger"),              1100,   134,    0,      0,  C),
    Item::new("7.62",                               None,                       440,    0,      0,      0,  None),
    Item::new("Brasa Shotgun",                      Some("Shotgun"),            440,    0,      0,      0,  None),
    Item::new("Buckshot",                           None,                       440,    0,      0,      0,  None),
    Item::new(r#"No.1 "The Liar" Submachinegun"#,   Some("The Liar"),           660,    0,      0,      0,  W),
    Item::new("Fiddler Submachine Gun Model 868",   Some("Fiddler"),            660,    0,      0,      0,  W),
    Item::new(r#""The Pitch Gun" mc. V"#,           Some("Pitch Gun"),          440,    0,      0,      0,  C),
    Item::new(r#""Lionclaw" mc.VIII"#,              Some("Lionclaw"),           660,    0,      0,      0,  C),
    Item::new("9mm",                                None,                       440,    0,      0,      0,  None),
    Item::new("PT-815 Smoke Grenade",               Some("Smoke Grenade"),      660,    0,      0,      0,  None),
    Item::new("Green Ash Grenade",                  Some("Green Ash"),          770,    0,      0,      0,  None),
    Item::new("12.7mm",                             None,                       550,    0,      0,      0,  None),
];

#[rustfmt::skip]
const MPF_HEAVY_ARMS: &[Item] = &[
    Item::new("135 Neville Anti-Tank Rifle",        Some("Anti-Tank Rifle"),    825,    0,      0,      0,  W),
    Item::new(r#"Typhon ra.XII"#,                   Some("Typhon"),             550,    0,      24,     0,  C),
    Item::new("20mm",                               None,                       550,    0,      0,      0,  None),
    Item::new("Mounted Bonesaw MK.3",               Some("Mounted Bonesaw"),    550,    0,      24,     0,  W),
    Item::new("Bonesaw MK.3",                       Some("Bonesaw"),            550,    0,      134,    0,  W),
    Item::new("Venom c.II 35",                      Some("Venom"),              550,    0,      79,     0,  C),
    Item::new("Bane 45",                            Some("Bane"),               825,    0,      220,    0,  C),
    Item::new("AP/RPG",                             None,                       330,    409,    0,      0,  C),
    Item::new("ARC/RPG",                            None,                       330,    409,    0,      0,  None),
    Item::new("Willow's Bane Model 845",            Some("Flamethrower"),       904,    0,      165,    0,  W),
    Item::new(r#""Molten Wind" v.II Flame Torch"#,  Some("Flamethrower"),       1014,   0,      134,    0,  C),
    Item::new("KLG91-2 Lunaire F",                  Some("Lunaire"),            275,    0,      24,     0,  C),
    Item::new("Tremola Grenade GPb-1",              Some("Tremola"),            825,    55,     0,      0,  None),
    Item::new("Malone Ratcheter MK.1",              Some("Malone Ratcheter"),   550,    0,      24,     0,  W),
    Item::new("Lamentum mm.IV",                     Some("Lamentum"),           550,    0,      24,     0,  C),
    Item::new("Daucus isg.III",                     Some("ISG"),                550,    0,      24,     0,  C),
    Item::new("30mm",                               None,                       440,    110,    0,      0,  None),
    Item::new("Cremari Mortar",                     None,                       550,    0,      134,    0,  None),
    Item::new("Mortar Flare Shell",                 None,                       330,    55,     0,      0,  None),
    Item::new("Mortar Shrapnel Shell",              None,                       330,    79,     0,      0,  None),
    Item::new("Mortar Shell",                       None,                       330,    189,    0,      0,  None),
    Item::new("BF5 White Ash Flask Grenade",        Some("White Ash"),          550,    220,    0,      0,  W),
    Item::new("Ignifist 30",                        Some("Ignifist"),           464,    189,    0,      0,  C),
    Item::new("Mammon 91-b",                        Some("Mammon"),             550,    55,     0,      0,  None),
    Item::new("Anti-Tank Sticky Bomb",              Some("Sticky Bomb"),        275,    275,    0,      0,  None),
    Item::new("Cutler Foebreaker",                  Some("Foebreaker"),         550,    0,      24,     0,  W),
    Item::new("Cutler Launcher 4",                  None,                       550,    0,      189,    0,  W),
    Item::new("RPG Shell",                          None,                       330,    244,    0,      0,  None),
];

#[rustfmt::skip]
const MPF_HEAVY_AMMUNITION: &[Item] = &[
    Item::new("150mm",                              None,                       660,    0,      0,      55,     None),
    Item::new("120mm",                              None,                       330,    79,     0,      0,      None),
    Item::new("250mm",                              None,                       660,    0,      0,      134,    None),
    Item::new("68mm",                               None,                       660,    660,    0,      0,      None),
    Item::new("40mm",                               None,                       880,    660,    0,      0,      None),
];

#[rustfmt::skip]
const MPF_UNIFORMS: &[Item] = &[
    Item::new("Specialist's Overcoat",              None,                       550,    0,      0,      0,  W),
    Item::new("Gunner's Breastplate",               None,                       550,    0,      0,      0,  W),
    Item::new("Sapper Gear",                        None,                       550,    0,      0,      0,  W),
    Item::new("Physician's Jacket",                 None,                       550,    0,      0,      0,  W),
    Item::new("Officer's Regalia",                  None,                       550,    0,      0,      0,  W),
    Item::new("Outrider's Mantle",                  None,                       550,    0,      0,      0,  W),
    Item::new("Caovish Parka",                      None,                       550,    0,      0,      0,  W),
    Item::new("Padded Boiler Suit",                 None,                       550,    0,      0,      0,  W),
    Item::new("Velian Flak Vest",                   None,                       550,    0,      0,      0,  C),
    Item::new("Fabri Rucksack",                     None,                       550,    0,      0,      0,  C),
    Item::new("Grenadier's Baldric",                None,                       550,    0,      0,      0,  C),
    Item::new("Medic Fatigues",                     None,                       550,    0,      0,      0,  C),
    Item::new("Officialis' Attire",                 None,                       550,    0,      0,      0,  C),
    Item::new("Legionary's Oilcoat",                None,                       550,    0,      0,      0,  C),
    Item::new("Recon Camo",                         None,                       550,    0,      0,      0,  C),
    Item::new("Heavy Topcoat",                      None,                       550,    0,      0,      0,  C),
    Item::new("Tankman's Coveralls",                None,                       550,    0,      0,      0,  C),
];

#[derive(Debug, PartialEq, Eq)]
enum Faction {
    Warden,
    Colonial,
}
const W: Option<Faction> = Some(Faction::Warden);
const C: Option<Faction> = Some(Faction::Colonial);

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
    let terminal = Arc::new(Mutex::new(Terminal::new(backend).unwrap()));

    let result = panic::catch_unwind({
        let terminal = terminal.clone();
        || {
            run_app(terminal);
        }
    });

    let mut terminal = terminal.lock().unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    if let Err(err) = result {
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

fn run_app<B: Backend>(terminal: Arc<Mutex<Terminal<B>>>) {
    let mut app = App::new();
    let mut terminal = terminal.lock().unwrap();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    loop {
        terminal
            .draw(|f| {
                ui(f, &mut app);
            })
            .unwrap();

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));
        if crossterm::event::poll(timeout).unwrap() {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                match app.selected_list {
                    0 => match key.code {
                        KeyCode::Right => {
                            app.main_list.unselect();
                            app.selected_list = 1;
                            app.todolist.select_next();
                        }
                        KeyCode::Up => app.main_list.select_previous(),
                        KeyCode::Down => app.main_list.select_next(),
                        KeyCode::Enter => {
                            app.add_to_todolist();
                        }
                        _ => {}
                    },
                    1 => match key.code {
                        KeyCode::Left => {
                            app.todolist.unselect();
                            app.selected_list = 0;
                            app.main_list.select_next();
                        }
                        KeyCode::Up => app.todolist.select_previous(),
                        KeyCode::Down => app.todolist.select_next(),
                        KeyCode::Enter => {
                            app.remove_from_todolist();
                        }
                        _ => {}
                    },
                    _ => {
                        app.selected_list = 0;
                    }
                }
                match key.code {
                    KeyCode::Char('q') => return,
                    KeyCode::Char('w') => app.write_output(),
                    KeyCode::Char('f') => app.swap_faction(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            // app.on_tick()
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    if let [left, right, ..] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size())
    {
        let items: Vec<ListItem> = app
            .todolist
            .items
            .iter()
            .enumerate()
            .map(|(n, item)| ListItem::new(format_todolist_entry(item, n, true)))
            .collect();
        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Todolist"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );
        if let [right_top, right_bottom, ..] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(right)
        {
            f.render_widget(
                Paragraph::new(format!("Faction: {:?}", app.faction))
                    .block(Block::default().borders(Borders::ALL).title("Settings")),
                right_top,
            );

            f.render_stateful_widget(items, right_bottom, &mut app.todolist.state);

            let items: Vec<ListItem> = app
                .main_list
                .items
                .iter()
                .filter(|item| {
                    if let DividedListItem::Item(item) = item {
                        item.faction
                            .as_ref()
                            .map(|faction| *faction == app.faction)
                            .unwrap_or(true)
                    } else {
                        true
                    }
                })
                .map(|item| match item {
                    DividedListItem::Divider(name) => ListItem::new(name.clone())
                        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::ITALIC)),
                    DividedListItem::Item(item) => ListItem::new(item.name),
                })
                .collect();
            let items = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Add"))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(items, left, &mut app.main_list.state);
        }
    }
}

fn format_todolist_entry(item: &Item, n: usize, letter_width_hack: bool) -> String {
    let format_material_amount =
        |out: &mut String, amount: u32, name: &str, crated_amount: u32, comma: bool| {
            if amount > 0 {
                let comma = if comma { ", " } else { "" };
                let crates = (amount as f32 / crated_amount as f32).ceil() as u32;
                write!(out, "{comma}{amount} {name} ({crates}📦)").unwrap();
                true
            } else {
                false | comma
            }
        };
    let format_material_amounts = |item: &Item| {
        let mut out = String::new();
        let mut comma = false;
        comma = format_material_amount(&mut out, item.bmats, "Bmats", 100, comma);
        comma = format_material_amount(&mut out, item.emats, "Emats", 20, comma);
        comma = format_material_amount(&mut out, item.rmats, "Rmats", 20, comma);
        let _ = format_material_amount(&mut out, item.hemats, "HEmats", 20, comma);
        out
    };
    format!(
        "{}{}・1 Queue of {}・{}",
        char::from_u32(0x1F1E6 + n as u32).unwrap_or('X'),
        if letter_width_hack { " " } else { "" },
        item.short_name.unwrap_or(item.name),
        format_material_amounts(item)
    )
}

struct App {
    main_list: DividedList<&'static Item>,
    todolist: StatefulList<&'static Item>,
    selected_list: usize,
    faction: Faction,
}

impl App {
    fn new() -> Self {
        let main_items = [
            ("Small Arms", MPF_SMALL_ARMS),
            ("Heavy Arms", MPF_HEAVY_ARMS),
            ("Heavy Ammunition", MPF_HEAVY_AMMUNITION),
            ("Uniforms", MPF_UNIFORMS),
        ]
        .into_iter()
        .flat_map(|(name, category)| {
            iter::once(DividedListItem::Divider(name.to_string()))
                .chain(category.into_iter().map(DividedListItem::Item))
        })
        .collect();
        Self {
            main_list: DividedList::with_items(main_items),
            todolist: StatefulList::with_items(Vec::new()),
            selected_list: 0,
            faction: Faction::Warden,
        }
    }

    fn add_to_todolist(&mut self) {
        if let Some(selected) = self.main_list.state.selected() {
            if let DividedListItem::Item(item) = self.main_list.items.get(selected).unwrap() {
                self.todolist.push(item);

                self.todolist.items.sort_by(|this, other| {
                    let find_category = |item: &'static Item| {
                        for (n, category) in [
                            MPF_SMALL_ARMS,
                            MPF_HEAVY_ARMS,
                            MPF_HEAVY_AMMUNITION,
                            MPF_UNIFORMS,
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            if category.as_ptr_range().contains(&(item as *const Item)) {
                                return n;
                            }
                        }
                        return usize::MAX;
                    };

                    find_category(this)
                        .cmp(&find_category(other))
                        .then((*this as *const Item).cmp(&(*other as *const Item)))
                });
            }
        }
    }

    fn remove_from_todolist(&mut self) {
        if let Some(selected) = self.todolist.state.selected() {
            let _ = self.todolist.remove(selected);
        }
    }

    fn write_output(&self) {
        let mut output = String::new();
        for (n, item) in self.todolist.items.iter().enumerate() {
            writeln!(output, "{}", format_todolist_entry(item, n, false)).unwrap();
        }
        std::fs::write("output.txt", output.as_bytes()).unwrap();
    }

    fn swap_faction(&mut self) {
        self.faction = match self.faction {
            Faction::Warden => Faction::Colonial,
            Faction::Colonial => Faction::Warden,
        };
    }
}

struct StatefulList<T> {
    state: ListState,
    last_pos: Option<usize>,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            last_pos: None,
            items,
        }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn remove(&mut self, idx: usize) -> T {
        if let Some(selected) = self.state.selected() {
            if selected == self.items.len() - 1 {
                self.state.select(selected.checked_sub(1));
            }
        }
        self.items.remove(idx)
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_pos.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_pos.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.last_pos = self.state.selected();
        self.state.select(None);
    }
}

struct DividedList<T> {
    state: ListState,
    last_pos: Option<usize>,
    items: Vec<DividedListItem<T>>,
}

enum DividedListItem<T> {
    Divider(String),
    Item(T),
}

impl<T> DividedList<T> {
    fn with_items(items: Vec<DividedListItem<T>>) -> Self {
        Self {
            state: ListState::default(),
            last_pos: None,
            items,
        }
    }

    fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(mut i) => loop {
                i = if i >= self.items.len() - 1 { 0 } else { i + 1 };
                if matches!(self.items.get(i), Some(DividedListItem::Item(_))) {
                    break i;
                }
            },
            None => self.last_pos.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let i = match self.state.selected() {
            Some(mut i) => loop {
                i = if i == 0 { self.items.len() - 1 } else { i - 1 };
                if matches!(self.items.get(i), Some(DividedListItem::Item(_))) {
                    break i;
                }
            },
            None => self.last_pos.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.last_pos = self.state.selected();
        self.state.select(None);
    }
}

struct Item {
    name: &'static str,
    short_name: Option<&'static str>,
    bmats: u32,
    emats: u32,
    rmats: u32,
    hemats: u32,
    #[allow(dead_code)]
    useless: bool,
    faction: Option<Faction>,
}

impl Item {
    const fn new(
        name: &'static str,
        short_name: Option<&'static str>,
        bmats: u32,
        emats: u32,
        rmats: u32,
        hemats: u32,
        faction: Option<Faction>,
    ) -> Self {
        Self {
            name,
            short_name,
            bmats,
            emats,
            rmats,
            hemats,
            useless: false,
            faction,
        }
    }

    const fn new_useless(
        name: &'static str,
        short_name: Option<&'static str>,
        bmats: u32,
        emats: u32,
        rmats: u32,
        hemats: u32,
        faction: Option<Faction>,
    ) -> Self {
        Self {
            name,
            short_name,
            bmats,
            emats,
            rmats,
            hemats,
            useless: true,
            faction,
        }
    }
}
