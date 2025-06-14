use crate::app::App;
use crate::game::AllyElement;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};
use tui_big_text::{BigText, PixelSize};
use tui_logger::{TuiLoggerSmartWidget, TuiLoggerWidget};

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
                let [grid_area, merge_panel] =
                    Layout::vertical([Constraint::Ratio(3, 4), Constraint::Fill(1)])
                        .areas(left_area);

                self.render_grid(grid_area, buf);
                self.render_logs(info_panel_area, buf);
                self.render_merge_panel(merge_panel, buf);
            }
        }
    }
}

impl App {
    fn render_logs(&mut self, area: Rect, buf: &mut Buffer) {
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

        let game = self.game.as_ref().unwrap();
        let selected_ally = game
            .selected
            .and_then(|(y, x)| game.board.ally_grid[y][x].as_ref());
        let hovered_ally = game.board.ally_grid[game.cursor.0][game.cursor.1].as_ref();

        match (selected_ally, hovered_ally) {
            (Some(lhs), Some(rhs)) => {
                Paragraph::new(lhs.name())
                    .alignment(Alignment::Center)
                    .render(ally_lhs, buf);
                Paragraph::new(rhs.name())
                    .alignment(Alignment::Center)
                    .render(ally_rhs, buf);
                // let output = todo!();
            }
            (Some(lhs), None) | (None, Some(lhs)) => {
                Paragraph::new(lhs.name())
                    .alignment(Alignment::Center)
                    .render(ally_lhs, buf);
            }
            (None, None) => {}
        }
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
                    None => Style::new().bg(Color::Black),
                };
                let block = Block::new().style(style);
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
        if let Some((sele_y, sele_x)) = game.selected {
            let sele_cell = grid[sele_y + 1][sele_x + 1].clone();
            let block = Block::bordered().border_style(Style::new().magenta());
            block.render(sele_cell, buf);
        }
    }
}
