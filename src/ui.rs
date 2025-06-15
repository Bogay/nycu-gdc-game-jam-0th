use crate::app::UniqueEffectId;
use crate::color_cycle::RepeatingColorCycle;
use crate::fx::effect;
// use crate::fx;
use crate::game::AllyElement;
use crate::styling::Catppuccin;
use crate::{app::App, game::Ally};
use color_eyre::eyre::{OptionExt, Result};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    prelude::StatefulWidget,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};
use ratatui_image::{Resize, StatefulImage};
use tachyonfx::{
    ColorSpace, Duration, EffectTimer, HslConvertable, Interpolation, Motion, ToRgbComponents, fx,
};
use tracing::info;
use tui_big_text::BigText;
use tui_logger::TuiLoggerWidget;

const APP_NAME: &str = "Brainrot TD";

impl Widget for &mut App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            crate::app::AppMode::Menu => {
                let big_text = BigText::builder()
                    .style(Style::new().blue())
                    .lines(vec![APP_NAME.into()])
                    .centered()
                    .build();
                big_text.render(area, buf);
            }
            crate::app::AppMode::InGame => {
                let block = Block::bordered()
                    .title(APP_NAME)
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded);
                let inner_block = block.inner(area);
                block.render(area, buf);

                let [left_area, info_panel_area] =
                    Layout::horizontal([Constraint::Ratio(3, 4), Constraint::Fill(1)])
                        .areas(inner_block);
                let [grid_area, merge_panel_area] =
                    Layout::vertical([Constraint::Ratio(3, 4), Constraint::Fill(1)])
                        .areas(left_area);

                self.render_grid(grid_area, buf);
                self.render_info_panel(info_panel_area, buf);
                self.render_merge_panel(merge_panel_area, buf);
            }
        }
    }
}

impl App {
    // fn selected_area(&self) -> Option<Rect> {
    //     self.game.and_then(|g| g.selected).map(|sele| {})
    // }

    fn render_info_panel(&mut self, area: Rect, buf: &mut Buffer) {
        let [status_panel_area, events_panel_area] =
            Layout::vertical([Constraint::Max(3 + 2), Constraint::Fill(1)]).areas(area);
        self.render_status_panel(status_panel_area, buf);
        self.render_events_panel(events_panel_area, buf);
    }

    fn render_status_panel(&mut self, area: Rect, buf: &mut Buffer) {
        let game = self.game.as_ref().unwrap();
        let block = Block::bordered().title("Status");
        let inner_block = block.inner(area);
        block.render(area, buf);
        Paragraph::new(vec![
            Line::raw(format!("Coin: {}", game.coin)),
            Line::raw(format!("Level: {}", game.level)),
            Line::raw(format!(
                "Remain Enemy: {}",
                game.board.enemy_ready2spawn.len()
            )),
        ])
        .render(inner_block, buf);
    }

    fn render_events_panel(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title("Events");
        let inner_block = block.inner(area);
        block.render(area, buf);
        TuiLoggerWidget::default()
            .state(&mut self.log_state.0)
            .render(inner_block, buf);
    }

    fn render_merge_panel(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Merge Italian Brainrot")
            .padding(Padding::horizontal(2));
        let inner_block = block.inner(area);
        block.render(area, buf);

        let [ally_lhs, plus, ally_rhs, eq, ally_output] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Max(3),
            Constraint::Fill(1),
            Constraint::Max(3),
            Constraint::Fill(1),
        ])
        .areas(inner_block);

        let [plus_mid] = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .areas(plus);
        let [eq_mid] = Layout::vertical([Constraint::Length(3)])
            .flex(Flex::Center)
            .areas(eq);

        Paragraph::new("+").render(plus_mid, buf);
        Paragraph::new("=").render(eq_mid, buf);

        let (selected_ally, hovered_ally) = {
            let game = self.game.as_ref().unwrap();
            let selected_ally = game
                .selected
                .and_then(|(y, x)| game.board.ally_grid[y][x].clone());
            let hovered_ally = game.board.ally_grid[game.cursor.0][game.cursor.1].clone();
            (selected_ally, hovered_ally)
        };
        match (selected_ally, hovered_ally) {
            (Some(lhs), Some(rhs)) => {
                self.render_ally(&lhs, ally_lhs, buf)
                    .expect("failed to render lhs ally");
                self.render_ally(&rhs, ally_rhs, buf)
                    .expect("failed to render lhs ally");
                if let Some(output) = self
                    .game
                    .as_mut()
                    .unwrap()
                    .ally_merge(lhs.clone(), rhs.clone())
                {
                    self.render_ally(&output, ally_output, buf)
                        .expect("failed to render output ally");
                }
            }
            (Some(lhs), None) | (None, Some(lhs)) => {
                self.render_ally(&lhs, ally_lhs, buf)
                    .expect("failed to render lhs ally");
            }
            (None, None) => {}
        }
    }

    fn render_ally(&mut self, ally: &Ally, area: Rect, buf: &mut Buffer) -> Result<()> {
        let [avatar_rect, name_rect] =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).areas(area);
        let ally_image = self
            .image_repository
            .get_mut(ally.avatar_path())
            .ok_or_eyre("failed to get ally image")?;
        let [avatar_rect_mid] = Layout::horizontal([Constraint::Length(16)])
            .flex(Flex::Center)
            .areas(avatar_rect);
        let image = StatefulImage::new().resize(Resize::Fit(None));
        image.render(avatar_rect_mid, buf, &mut ally_image.0);
        Paragraph::new(ally.name())
            .bg(Color::Black)
            .alignment(Alignment::Center)
            .render(name_rect, buf);
        Ok(())
    }

    fn render_grid(&mut self, grid_area: Rect, buf: &mut Buffer) {
        let game = self.game.as_ref().unwrap();

        const GRID_WIDTH: usize = 9;
        const GRID_HEIGHT: usize = 5;

        let row_constraints = vec![Constraint::Max(10); GRID_HEIGHT];
        let grid = Layout::vertical(row_constraints)
            .split(grid_area)
            .iter()
            .map(|&a| {
                let col_constrains = vec![Constraint::Max(20); GRID_WIDTH];
                Layout::horizontal(col_constrains).split(a).to_vec()
            })
            .collect::<Vec<_>>();
        assert_eq!(GRID_HEIGHT, grid.len());
        assert_eq!(GRID_WIDTH, grid[0].len());

        if self.is_selection_updated {
            self.is_selection_updated = false;

            if let Some((sele_y, sele_x)) = game.selected {
                let sele_cell = grid[sele_y + 1][sele_x + 1].clone();
                self.effects.0.add_unique_effect(
                    UniqueEffectId::Selected,
                    effect::selected_category(Color::Cyan, sele_cell.clone()),
                );
            } else {
                self.effects.0.unique(
                    UniqueEffectId::Selected,
                    effect::selected_category(Color::Cyan, Rect::ZERO),
                );
            }
        }

        // render all cells first
        // for row in &grid {
        //     for cell in row {
        //         let p = Paragraph::new("")
        //             .block(Block::bordered())
        //             .style(Style::new().gray());
        //         p.render(cell.clone(), buf);
        //     }
        // }

        // render ally grid
        for row_i in 1..GRID_HEIGHT - 1 {
            for col_i in 1..GRID_WIDTH - 1 {
                let ally = &game.board.ally_grid[row_i - 1][col_i - 1];
                let text = match ally {
                    Some(a) => a.level.to_string(),
                    None => "".to_string(),
                };

                let style = calculate_ally_style(ally);
                let block = Block::bordered().style(style);
                let p = Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center);

                let rect = grid[row_i][col_i].clone();
                p.render(rect, buf);
            }
        }

        // update fx
        if self.is_ally_updated {
            self.is_ally_updated = false;
            for row_i in 1..GRID_HEIGHT - 1 {
                for col_i in 1..GRID_WIDTH - 1 {
                    let ally = &game.board.ally_grid[row_i - 1][col_i - 1];
                    if let Some((e0, e1)) = ally
                        .as_ref()
                        .and_then(|a| a.second_element.map(|e1| (a.element, e1)))
                    {
                        let c0 = ally_element_color(e0);
                        let c1 = ally_element_color(e1);
                        let rect = grid[row_i][col_i].clone();
                        let fx =
                            effect::color_cycle_bg(mixed_element_color(c0, c1, 3), 66, |_| true)
                                .with_area(rect);
                        self.effects.0.add_effect(fx);
                    }
                }
            }
        }

        // render enemies
        let grid_indices = (0..GRID_WIDTH)
            .map(|x| (0, x))
            .chain((1..GRID_HEIGHT).map(|y| (y, GRID_WIDTH - 1)))
            .chain((0..GRID_WIDTH - 1).rev().map(|x| (GRID_HEIGHT - 1, x)))
            .chain((1..GRID_HEIGHT - 1).rev().map(|y| (y, 0)))
            .collect::<Vec<_>>();
        let mut counts = [[0; GRID_WIDTH]; GRID_HEIGHT];
        for e in &game.board.enemies {
            let pos_i = e.position.floor() as usize % grid_indices.len();
            let (grid_y, grid_x) = grid_indices[pos_i];
            counts[grid_y][grid_x] += 1;
        }
        for &(grid_y, grid_x) in &grid_indices {
            let cell = grid[grid_y][grid_x];
            let text = match counts[grid_y][grid_x] {
                0 => "".to_string(),
                c @ _ => format!("{c}"),
            };
            let p = Paragraph::new(text)
                .block(Block::bordered())
                .alignment(Alignment::Center)
                .style(Style::new().gray());
            p.render(cell.clone(), buf);
        }

        // render cursor and selected
        let (cursor_y, cursor_x) = game.cursor;
        let cursor_cell = grid[cursor_y + 1][cursor_x + 1].clone();
        let block = Block::bordered().border_style(Style::new().magenta());
        block.render(cursor_cell, buf);
    }
}

fn calculate_ally_style(ally: &Option<Ally>) -> Style {
    match ally.as_ref().map(|a| a.element) {
        Some(elem) => Style::new().bg(ally_element_color(elem)),
        None => Style::new().bg(Color::Black),
    }
}

fn ally_element_color(elem: AllyElement) -> Color {
    match elem {
        AllyElement::Basic => Catppuccin::new().yellow,
        AllyElement::Slow => Color::LightBlue,
        AllyElement::Dot => Color::LightGreen,
        AllyElement::Aoe => Color::LightRed,
        AllyElement::Critical => Color::Gray,
    }
}

fn mixed_element_color(c0: Color, c1: Color, step: usize) -> RepeatingColorCycle {
    let color_step: usize = 7 * step;

    let (h0, s0, l0) = c0.to_hsl_f32();
    let (h1, s1, l1) = c1.to_hsl_f32();

    let color_l0 = Color::from_hsl_f32(h0, s0, 80.0);
    let color_d0 = Color::from_hsl_f32(h0, s0, 40.0);
    let color_l1 = Color::from_hsl_f32(h1, s1, 80.0);
    let color_d1 = Color::from_hsl_f32(h1, s1, 40.0);

    RepeatingColorCycle::new(
        c0,
        &[
            (4 * step, color_d0),
            (2 * step, color_l0),
            (
                4 * step,
                Color::from_hsl_f32((h0 - 25.0) % 360.0, s0, (l0 + 10.0).min(100.0)),
            ),
            (
                color_step,
                Color::from_hsl_f32(h0, (s0 - 20.0).max(0.0), (l0 + 10.0).min(100.0)),
            ),
            (
                color_step,
                Color::from_hsl_f32((h0 + 25.0) % 360.0, s0, (l0 + 10.0).min(100.0)),
            ),
            (
                color_step,
                Color::from_hsl_f32(h0, (s0 + 20.0).max(0.0), (l0 + 10.0).min(100.0)),
            ),
            (
                color_step,
                Color::from_hsl_f32(h1, (s1 + 20.0).max(0.0), (l1 + 10.0).min(100.0)),
            ),
            (
                color_step,
                Color::from_hsl_f32((h1 + 25.0) % 360.0, s1, (l1 + 10.0).min(100.0)),
            ),
            (
                color_step,
                Color::from_hsl_f32(h1, (s1 - 20.0).max(0.0), (l1 + 10.0).min(100.0)),
            ),
            (
                4 * step,
                Color::from_hsl_f32((h1 - 25.0) % 360.0, s1, (l1 + 10.0).min(100.0)),
            ),
            (2 * step, color_l1),
            (4 * step, color_d1),
        ],
    )
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    a + ((b - a) as f32 * t).floor() as u8
}
