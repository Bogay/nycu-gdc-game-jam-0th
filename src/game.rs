use rand::prelude::IndexedRandom;
use std::clone;

#[derive(Debug, Default)]
pub enum GameState {
    #[default]
    Init,
    Running,
    Pause,
    End,
}

#[derive(Debug)]
pub struct Board {
    pub ally_grid: Vec<Vec<Option<Ally>>>,
    pub enemies: Vec<Enemy>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ally {
    pub element: AllyElement,
    pub second_element: Option<AllyElement>,
    pub atk: usize,
    pub range: usize,
    pub aoe_range: usize,
    pub level: usize,
    pub atk_speed: f32,
    pub attack_cooldown: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllyElement {
    Basic,
    Slow,
    Aoe,
    Dot,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub hp: usize,
    pub move_speed: f32,
    pub position: f32, // from 0 to 24
    pub dot_list: Vec<Debuff>,
    pub slow_list: Vec<Debuff>,
}

#[derive(Debug, Clone)]
pub struct Debuff {
    pub value: usize,
    pub cooldown: f32,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Game {
    pub level: usize,
    pub game_state: GameState,
    pub board: Board,
    pub cursor: (usize, usize),
    pub selected: Option<(usize, usize)>,
    pub coin: usize,
}

impl Game {
    pub fn new() -> Game {
        Game {
            level: 1,
            cursor: (0, 0),
            selected: None,
            coin: 100,
            game_state: GameState::Init,
            board: Board {
                ally_grid: vec![vec![None; 7]; 3],
                enemies: Vec::new(),
            },
        }
    }

    pub fn init_game() {
        todo!()
    }

    fn state_checkwin() {
        todo!()
    }

    fn state_pause() {
        todo!()
    }

    fn state_resume() {
        todo!()
    }

    // Deduct coins and spawn an ally if possible
    pub fn buy_ally(&mut self) {
        if self.coin >= 10 {
            self.coin -= 10;
            self.ally_spawn();
        }
    }

    // Generate a level 1 ally on a random empty grid
    pub fn ally_spawn(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut empty_cells = Vec::new();
        for (i, row) in self.board.ally_grid.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                if cell.is_none() {
                    empty_cells.push((i, j));
                }
            }
        }
        if let Some(&(i, j)) = empty_cells.choose(&mut thread_rng()) {
            let ally = Ally {
                element: AllyElement::Basic,
                second_element: None,
                atk: 10,
                range: 1,
                aoe_range: 0,
                level: 1,
                atk_speed: 1.0,
                attack_cooldown: 0.0,
            };
            self.board.ally_grid[i][j] = Some(ally);
        }
    }

    //if drop a save level on a allay they will levelup
    // Merge two allies at the given positions (i1, j1) and (i2, j2)
    fn ally_merge(&mut self, ally1: Ally, ally2: Ally) -> Option<Ally> {
        // Check if levels are the same
        if ally1.level != ally2.level {
            return None;
        }

        // To compare AllyElement and Option<AllyElement>, derive PartialEq for AllyElement and Option<AllyElement>
        // (Already derived via #[derive(Debug,Clone)] for AllyElement, but need PartialEq)
        // Let's add PartialEq to AllyElement and Option<AllyElement> in the struct definition (not shown here).

        if ally1.element == ally2.element && ally1.second_element == ally2.second_element {
            Some(Ally {
                element: ally1.element.clone(),
                second_element: None,
                atk: ((ally1.atk as f32) * 1.5) as usize,
                range: ((ally1.range as f32) * 1.5) as usize,
                aoe_range: ((ally1.aoe_range as f32) * 1.5) as usize,
                level: ally1.level + 1,
                atk_speed: ally1.atk_speed * 1.5,
                attack_cooldown: 0.0,
            })
        } else if ally1.second_element.is_none() && ally2.second_element.is_none() {
            // Merge two no second element allies (no upgrade)
            Some(Ally {
                element: ally1.element.clone(),
                second_element: Some(ally2.element.clone()),
                atk: ally1.atk,
                range: ally1.range,
                aoe_range: ally1.aoe_range,
                level: ally1.level,
                atk_speed: ally1.atk_speed,
                attack_cooldown: 0.0,
            })
        } else {
            None
        }
    }

    //todo
    fn ally_attack(&mut self) {
        todo!()
    }

    //handle cursor movement
    fn cursor_move(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                self.cursor.0 -= 1;
            }
            Direction::Down => {
                self.cursor.0 += 1;
            }
            Direction::Left => {
                self.cursor.1 -= 1;
            }
            Direction::Right => {
                self.cursor.1 += 1;
            }
        }
    }

    //select a ally if there is a ally at cursor
    fn cursor_select(&mut self) {
        if self.selected.is_some() {
            return;
        }

        let (i, j) = self.cursor;
        if let Some(Some(_)) = self.board.ally_grid.get(i).and_then(|row| row.get(j)) {
            self.selected = Some((i, j));
        } else {
            self.selected = None;
        }
    }

    //drop the select ally on empty grid or merge on allay
    fn cursor_drop(&mut self) {
        todo!()
    }

    fn enemy_grid_position(ene: Enemy) -> (f32, f32) {
        let grid_position: (f32, f32);
        if ene.position < 8.0 {
            //at top
            grid_position = (ene.position as f32, 0.0)
        } else if ene.position < 12.0 {
            // right
            grid_position = (8.0, ene.position as f32 - 8.0)
        } else if ene.position < 20.0 {
            // bottom
            grid_position = (ene.position as f32 - 12.0, 12.0)
        } else if ene.position < 24.0 {
            // left
            grid_position = (0.0, ene.position as f32 - 20.0)
        } else {
            // out of bounds
            grid_position = (0.0, 0.0)
        }
        grid_position
    }

    fn enemy_getattack(&mut self) {
        todo!()
    }

    fn enemy_die(&mut self) {
        todo!()
    }

    fn enemy_update(&mut self) {
        todo!()
    }

    fn enemy_move(&mut self) {
        todo!()
    }
}
