use crate::{
    event::{AppEvent, Event, EventHandler},
    game::{Ally, AllyElement, Game},
};
use color_eyre::Result;
use rand::seq::IndexedRandom;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use ratatui_image::{
    picker::Picker,
    protocol::{ImageSource, Protocol, StatefulProtocol},
};
use std::{collections::HashMap, fmt::Debug, time::Instant};
use tachyonfx::{Duration, EffectManager};
use tracing::info;
use tui_logger::TuiWidgetState;

/// Workaround to make TuiWidgetState `Debug`
pub struct TuiWidgetStateWrapper(pub TuiWidgetState);

impl Debug for TuiWidgetStateWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TuiWidgetStateWrapper ")?;
        Ok(())
    }
}

pub struct ProtocolWrapper(pub StatefulProtocol);

impl Debug for ProtocolWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProtocolWrapper")
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    pub game: Option<Game>,
    pub mode: AppMode,
    pub log_state: TuiWidgetStateWrapper,
    /// For rendering image
    pub picker: Picker,
    /// Store all images used in game
    pub image_repository: HashMap<String, ProtocolWrapper>,
    pub last_tick: Instant,
    pub effects: Effects,
    pub is_selection_updated: bool,
    pub is_ally_updated: bool,
}

pub struct Effects(pub EffectManager<UniqueEffectId>);

impl Debug for Effects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Effects")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UniqueEffectId {
    #[default]
    Selected,
    Hover,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppMode {
    Menu,
    InGame,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            game: None,
            mode: AppMode::Menu,
            log_state: TuiWidgetStateWrapper(TuiWidgetState::default()),
            picker: Picker::from_query_stdio().expect("failed to init app.picker"),
            image_repository: HashMap::new(),
            effects: Effects(EffectManager::default()),
            last_tick: Instant::now(),
            is_selection_updated: false,
            is_ally_updated: false,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            let duration = self.last_tick.elapsed().into();
            self.last_tick = Instant::now();
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());
                let area = frame.area();
                self.effects
                    .0
                    .process_effects(duration, frame.buffer_mut(), area);
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Increment => self.increment_counter(),
                AppEvent::Decrement => self.decrement_counter(),
                AppEvent::Quit => self.quit(),
                AppEvent::StartGame => {
                    assert_eq!(AppMode::Menu, self.mode);
                    self.game = Some(Game::new());
                    self.game.as_mut().unwrap().init_game();
                    self.init_image_repository()
                        .expect("failed to read image assets");
                    self.mode = AppMode::InGame;
                }
                AppEvent::MoveCursor(direction) => {
                    assert!(self.game.is_some());
                    self.game.as_mut().unwrap().cursor_move(direction);
                }
                AppEvent::ToggleSelection => {
                    assert!(self.game.is_some());
                    self.game.as_mut().unwrap().cursor_select();
                    self.is_selection_updated = true;
                    self.is_ally_updated = true;
                }
                AppEvent::BuyAlly => {
                    assert!(self.game.is_some());
                    self.game.as_mut().unwrap().buy_ally();
                    self.is_ally_updated = true;
                }
            },
        }
        Ok(())
    }

    fn init_image_repository(&mut self) -> Result<()> {
        let image_paths = std::fs::read_dir("assets/avatars/")?
            .map(|r| r.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()?;
        info!(count = image_paths.len(), "load image");
        for p in &image_paths {
            info!(path = p.to_str(), "load single image");
        }
        let image_sources = image_paths
            .iter()
            .map::<Result<image::DynamicImage>, _>(|p| Ok(image::ImageReader::open(p)?.decode()?))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|img| ProtocolWrapper(self.picker.new_resize_protocol(img)));
        assert_eq!(image_paths.len(), image_sources.len());
        self.image_repository.extend(
            image_paths
                .into_iter()
                .map(|e| e.to_string_lossy().to_string())
                .zip(image_sources),
        );
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Enter if matches!(self.mode, AppMode::Menu) => {
                self.events.send(AppEvent::StartGame);
            }
            // Other handlers you could add here.
            _ => {}
        }

        if matches!(self.mode, AppMode::InGame) {
            match key_event.code {
                KeyCode::Up => self
                    .events
                    .send(AppEvent::MoveCursor(crate::game::Direction::Up)),
                KeyCode::Down => self
                    .events
                    .send(AppEvent::MoveCursor(crate::game::Direction::Down)),
                KeyCode::Left => self
                    .events
                    .send(AppEvent::MoveCursor(crate::game::Direction::Left)),
                KeyCode::Right => self
                    .events
                    .send(AppEvent::MoveCursor(crate::game::Direction::Right)),
                KeyCode::Enter => self.events.send(AppEvent::ToggleSelection),
                KeyCode::Char(' ') => {
                    self.events.send(AppEvent::BuyAlly);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        if let Some(game) = self.game.as_mut() {
            game.update();
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
}
