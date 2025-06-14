use crate::{
    event::{AppEvent, Event, EventHandler},
    game::{Ally, AllyElement, Game},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

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
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
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
                    self.mode = AppMode::InGame;
                }
                AppEvent::MoveCursor(direction) => {
                    assert!(self.game.is_some());
                    self.game.as_mut().unwrap().cursor_move(direction);
                }
                AppEvent::ToggleSelection => {
                    assert!(self.game.is_some());
                    self.game.as_mut().unwrap().cursor_select();
                }
                AppEvent::BuyAlly => {
                    assert!(self.game.is_some());
                    self.game.as_mut().unwrap().buy_ally();
                }
            },
        }
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
