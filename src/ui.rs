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

        const GRID_WIDTH: usize = 9;
        const GRID_HEIGHT: usize = 5;

        let row_constraints = vec![Constraint::Max(10); GRID_HEIGHT];
        let grid_layouts = Layout::vertical(row_constraints)
            .split(inner_block)
            .iter()
            .map(|&a| {
                let col_constrains = vec![Constraint::Max(20); GRID_WIDTH];
                Layout::horizontal(col_constrains).split(a).to_vec()
            })
            .collect::<Vec<_>>();
        assert_eq!(GRID_HEIGHT, grid_layouts.len());
        assert_eq!(GRID_WIDTH, grid_layouts[0].len());

        // render all cells first
        for row in &grid_layouts {
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
                let ally = &game.borad.ally_grid[row_i - 1][col_i - 1];
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
                let p = Paragraph::new(text).block(block);

                let rect = grid_layouts[row_i][col_i].clone();
                p.render(rect, buf);
            }
        }
    }
}
