use std::{collections, ops::SubAssign};

use crate::app::App;
use crate::game::AllyElement;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};

impl Widget for &mut App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Brainrot TD")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        let inner_block = block.inner(area);
        block.render(area, buf);

        let [left_area, info_panel_area] =
            Layout::horizontal([Constraint::Ratio(3, 4), Constraint::Percentage(0)])
                .areas(inner_block);
        let [grid_area, action_panel_area] =
            Layout::vertical([Constraint::Ratio(3, 4), Constraint::Percentage(0)]).areas(left_area);

        self.render_grid(buf, grid_area);
    }
}

impl App {
    fn render_grid(&mut self, buf: &mut Buffer, grid_area: Rect) {
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

        // render all cells first
        for row in &grid {
            for cell in row {
                let p = Paragraph::new("")
                    .block(Block::bordered())
                    .style(Style::new().gray());
                p.render(cell.clone(), buf);
            }
        }

        // render ally grid
        let game = self.game.as_ref().unwrap();
        for row_i in 1..GRID_HEIGHT - 1 {
            for col_i in 1..GRID_WIDTH - 1 {
                let ally = &game.board.ally_grid[row_i - 1][col_i - 1];
                let text = match ally {
                    Some(a) => a.level.to_string(),
                    None => "".to_string(),
                };
                let style = match ally.as_ref().map(|a| &a.element) {
                    Some(AllyElement::Basic) => Style::new().bg(Color::White),
                    Some(AllyElement::Slow) => Style::new().bg(Color::LightBlue),
                    Some(AllyElement::Dot) => Style::new().bg(Color::LightGreen),
                    Some(AllyElement::Aoe) => Style::new().bg(Color::LightRed),
                    Some(AllyElement::Critical) => Style::new().bg(Color::Yellow),
                    None => Style::new().bg(Color::Gray),
                };
                let block = Block::bordered().style(style);
                let p = Paragraph::new(text)
                    .block(block)
                    .alignment(Alignment::Center);

                let rect = grid[row_i][col_i].clone();
                p.render(rect, buf);
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
            let pos_i = e.position.floor() as usize;
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
    }
}
