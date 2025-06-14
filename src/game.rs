use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::clone;
use tracing::info;

#[derive(Debug, Default)]
pub enum GameState {
    #[default]
    Init,
    Running,
    Pause,
    End,
}

#[derive(Debug, Default)]
pub struct Board {
    pub ally_grid: Vec<Vec<Option<Ally>>>,
    pub enemies: Vec<Enemy>,
    enemy_ready2spawn: Vec<(Enemy, usize)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
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

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AllyElement {
    #[default]
    Basic,
    Slow,
    Aoe,
    Dot,
    Critical,
}

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub enum Direction {
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
                enemy_ready2spawn: Vec::new(),
            },
        }
    }

    pub fn init_game(&mut self) {
        self.enemy_spawn();
    }

    pub fn update(&mut self) {
        // at 60 FPS, called every frame
        self.ally_update();
        self.enemy_update();
        if self.state_checkwin() {
            self.game_state = GameState::End;
        }
    }

    fn ally_update(&mut self) {
        // Collect positions of allies that are ready to attack after updating cooldowns
        let mut ready_to_attack = Vec::new();

        for (i, row) in self.board.ally_grid.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                if let Some(ally) = cell {
                    // Decrease attack_cooldown if above zero
                    if ally.attack_cooldown > 0.0 {
                        ally.attack_cooldown -= 1.0 / 60.0;
                        if ally.attack_cooldown < 0.0 {
                            ally.attack_cooldown = 0.0;
                        }
                    }
                    // If cooldown is zero or less, mark for attack
                    if ally.attack_cooldown <= 0.0 {
                        ready_to_attack.push((i, j));
                    }
                }
            }
        }

        let mut atk_speeds = Vec::new();
        for &(i, j) in &ready_to_attack {
            if let Some(ally) = self.board.ally_grid[i][j].as_ref() {
                atk_speeds.push((i, j, ally.atk_speed));
            }
        }

        for (i, j, atk_speed) in atk_speeds {
            self.ally_ready2attack((i, j));
            if let Some(ally) = self.board.ally_grid[i][j].as_mut() {
                ally.attack_cooldown = atk_speed;
            }
        }
    }

    fn ally_ready2attack(&mut self, pos: (usize, usize)) {
        let (i, j) = pos;
        if let Some(ally) = self.board.ally_grid[i][j].as_ref() {
            if ally.element == AllyElement::Aoe || ally.second_element == Some(AllyElement::Aoe) {
                self.ally_AOE_damage(pos);
            } else {
                self.ally_damage(pos);
            }
        }
    }

    // Find the nearest enemy within range and attack it
    // The ally position is its (i, j) on the grid (3x7), which is mapped to (x, y) in world space as (j+1, i+1)
    // get the enemys position from
    fn ally_damage(&mut self, _pos: (usize, usize)) {
        let (i, j) = _pos;
        let ally_position = (j as f32 + 1.0, i as f32 + 1.0);

        // Find the nearest enemy within range
        let mut nearest_enemy_idx: Option<usize> = None;
        let mut nearest_dist: f32 = f32::MAX;
        let mut ally_range = 1;
        let mut ally_atk = 0;
        let mut first_element = AllyElement::Basic;
        let mut second_element = None;

        if let Some(ally) = self.board.ally_grid[i][j].as_ref() {
            ally_range = ally.range;
            ally_atk = ally.atk;
            first_element = ally.element.clone();
            second_element = ally.second_element.clone();
        } else {
            return;
        }

        // Use iterator methods to find the nearest enemy within range in a functional style
        nearest_enemy_idx = self
            .board
            .enemies
            .iter()
            .enumerate()
            .filter_map(|(idx, enemy)| {
                let enemy_pos = Game::enemy_grid_position(enemy.clone());
                let dx = ally_position.0 - enemy_pos.0;
                let dy = ally_position.1 - enemy_pos.1;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= ally_range as f32 {
                    Some((idx, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(idx, _)| idx);

        // Prepare damage value (with critical hit if applicable)
        let mut damage = ally_atk;
        if first_element == AllyElement::Critical || second_element == Some(AllyElement::Critical) {
            damage = (damage as f32 * 2.0) as usize;
        }
        if let Some(enemy_idx) = nearest_enemy_idx {
            let enemy = &mut self.board.enemies[enemy_idx];

            // Apply debuffs (first and second element, exclude AOE)
            match first_element {
                AllyElement::Slow => {
                    enemy.slow_list.push(Debuff {
                        value: 1,
                        cooldown: 1.0,
                    });
                }
                AllyElement::Dot => {
                    enemy.dot_list.push(Debuff {
                        value: 2,
                        cooldown: 2.0,
                    });
                }
                _ => {}
            }
            if let Some(second) = &second_element {
                match second {
                    AllyElement::Slow => {
                        enemy.slow_list.push(Debuff {
                            value: 1,
                            cooldown: 1.0,
                        });
                    }
                    AllyElement::Dot => {
                        enemy.dot_list.push(Debuff {
                            value: 2,
                            cooldown: 2.0,
                        });
                    }
                    _ => {}
                }
            }

            // Apply direct damage, with critical hit if applicable

            enemy.hp = enemy.hp.saturating_sub(damage);
        }
    }

    fn ally_AOE_damage(&mut self, _pos: (usize, usize)) {
        let (i, j) = _pos;
        let ally_position = (j as f32 + 1.0, i as f32 + 1.0);

        // Find the nearest enemy within range
        let mut nearest_enemy_idx: Option<usize> = None;
        let mut nearest_dist: f32 = f32::MAX;
        let mut ally_range = 1;
        let mut ally_atk = 0;
        let mut first_element = AllyElement::Basic;
        let mut second_element = None;

        if let Some(ally) = self.board.ally_grid[i][j].as_ref() {
            ally_range = ally.range;
            ally_atk = ally.atk;
            first_element = ally.element.clone();
            second_element = ally.second_element.clone();
        } else {
            return;
        }

        nearest_enemy_idx = self
            .board
            .enemies
            .iter()
            .enumerate()
            .filter_map(|(idx, enemy)| {
                let enemy_pos = Game::enemy_grid_position(enemy.clone());
                let dx = ally_position.0 - enemy_pos.0;
                let dy = ally_position.1 - enemy_pos.1;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= ally_range as f32 {
                    Some((idx, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(idx, _)| idx);

        if let Some(enemy_idx) = nearest_enemy_idx {
            let enemy_pos = {
                let enemy = &self.board.enemies[enemy_idx];
                Game::enemy_grid_position(enemy.clone())
            };

            // Prepare damage value (with critical hit if applicable)
            let mut damage = ally_atk;
            if first_element == AllyElement::Critical
                || second_element == Some(AllyElement::Critical)
            {
                damage = (damage as f32 * 2.0) as usize;
            }

            // For all enemies within aoe_range of the target enemy, apply damage and debuffs
            let aoe_range = if let Some(ally) = self.board.ally_grid[i][j].as_ref() {
                ally.aoe_range
            } else {
                0
            };

            for enemy in self.board.enemies.iter_mut() {
                let pos = Game::enemy_grid_position(enemy.clone());
                let dx = enemy_pos.0 - pos.0;
                let dy = enemy_pos.1 - pos.1;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= aoe_range as f32 {
                    // Apply debuffs (first and second element, exclude AOE)
                    match first_element {
                        AllyElement::Slow => {
                            enemy.slow_list.push(Debuff {
                                value: 1,
                                cooldown: 1.0,
                            });
                        }
                        AllyElement::Dot => {
                            enemy.dot_list.push(Debuff {
                                value: 2,
                                cooldown: 2.0,
                            });
                        }
                        _ => {}
                    }
                    if let Some(second) = &second_element {
                        match second {
                            AllyElement::Slow => {
                                enemy.slow_list.push(Debuff {
                                    value: 1,
                                    cooldown: 1.0,
                                });
                            }
                            AllyElement::Dot => {
                                enemy.dot_list.push(Debuff {
                                    value: 2,
                                    cooldown: 2.0,
                                });
                            }
                            _ => {}
                        }
                    }

                    // Apply damage
                    enemy.hp = enemy.hp.saturating_sub(damage);
                }
            }
        }
    }

    fn enemy_update(&mut self) {
        // Update spawn timers and spawn enemies if ready
        let mut spawned = Vec::new();
        for (idx, &mut (_, ref mut timer)) in self.board.enemy_ready2spawn.iter_mut().enumerate() {
            if *timer > 0 {
                *timer -= 1;
            }
            if *timer == 0 {
                spawned.push(idx);
            }
        }
        // Spawn enemies whose timers reached 0
        for &idx in spawned.iter().rev() {
            let (enemy, _) = self.board.enemy_ready2spawn.remove(idx);
            self.board.enemies.push(enemy);
        }

        // Update all enemies
        for enemy in self.board.enemies.iter_mut() {
            // Apply DOT debuffs
            let mut dot_damage = 0;
            enemy.dot_list.retain_mut(|debuff| {
                if debuff.cooldown > 0.0 {
                    dot_damage += debuff.value;
                    debuff.cooldown -= 1.0 / 60.0;
                    debuff.cooldown > 0.0
                } else {
                    false
                }
            });
            if dot_damage > 0 {
                enemy.hp = enemy.hp.saturating_sub(dot_damage);
            }

            // Apply slow debuffs
            let mut slow_factor = 1.0;
            enemy.slow_list.retain_mut(|debuff| {
                if debuff.cooldown > 0.0 {
                    slow_factor *= 0.5_f32.powi(debuff.value as i32);
                    debuff.cooldown -= 1.0 / 60.0;
                    debuff.cooldown > 0.0
                } else {
                    false
                }
            });

            // Move enemy
            let move_amount = enemy.move_speed * slow_factor * (1.0 / 60.0);
            enemy.position += move_amount;
        }

        // Remove dead enemies
        self.board.enemies.retain(|enemy| enemy.hp > 0);
    }
    fn state_checkwin(&self) -> bool {
        self.board.enemy_ready2spawn.is_empty() && self.board.enemies.is_empty()
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
        } else {
            info!(required = 10, current = self.coin, "coin not enough!");
        }
    }

    // Generate a level 1 ally on a random empty grid
    fn ally_spawn(&mut self) {
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
                atk: 1000,
                range: 5,
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

    //handle cursor movement
    pub fn cursor_move(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.cursor.0 == 0 {
                    self.cursor.0 = 2;
                } else {
                    self.cursor.0 -= 1;
                }
            }
            Direction::Down => {
                if self.cursor.0 == 2 {
                    self.cursor.0 = 0;
                } else {
                    self.cursor.0 += 1;
                }
            }
            Direction::Left => {
                if self.cursor.1 == 0 {
                    self.cursor.1 = 6;
                } else {
                    self.cursor.1 -= 1;
                }
            }
            Direction::Right => {
                if self.cursor.1 == 6 {
                    self.cursor.1 = 0;
                } else {
                    self.cursor.1 += 1;
                }
            }
        }
    }

    //select a ally if there is a ally at cursor
    pub fn cursor_select(&mut self) {
        if self.selected.is_some() {
            self.cursor_drop();
            return;
        }

        let (i, j) = self.cursor;
        if let Some(Some(_)) = self.board.ally_grid.get(i).and_then(|row| row.get(j)) {
            self.selected = Some((i, j));
        } else {
            self.selected = None;
        }
    }

    // Drop the selected ally on an empty grid or merge with an ally at the cursor
    fn cursor_drop(&mut self) {
        if let Some((sel_i, sel_j)) = self.selected {
            let (cur_i, cur_j) = self.cursor;

            if (sel_i, sel_j) == (cur_i, cur_j) {
                return;
            }
            let ally1 = self.board.ally_grid[sel_i][sel_j].take();

            if let Some(ally1) = ally1 {
                if let Some(Some(ally2)) = self
                    .board
                    .ally_grid
                    .get(cur_i)
                    .and_then(|row| row.get(cur_j))
                {
                    if let Some(merged) = self.ally_merge(ally1.clone(), ally2.clone()) {
                        // Place merged ally at cursor, clear selected cell
                        self.board.ally_grid[cur_i][cur_j] = Some(merged);
                        self.selected = None;
                    } else {
                        // Merge failed, return ally1 to its original position
                        self.board.ally_grid[sel_i][sel_j] = Some(ally1);
                        // Optionally, keep selection or clear it
                    }
                } else {
                    // No ally at cursor, move selected ally to cursor position
                    self.board.ally_grid[cur_i][cur_j] = Some(ally1);
                    self.selected = None;
                }
            } else {
                // No ally at selected position, clear selection
                self.selected = None;
            }
        }
    }

    fn enemy_grid_position(ene: Enemy) -> (f32, f32) {
        let grid_position: (f32, f32);
        if ene.position < 8.0 {
            grid_position = (ene.position as f32, 0.0)
        } else if ene.position < 12.0 {
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

    fn enemy_spawn(&mut self) {
        use rand::Rng;
        let mut rng = thread_rng();
        // Push 10 enemies with random spawn times (0..=100 ticks)
        for _ in 0..10 {
            let enemy = Enemy {
                hp: 100,
                move_speed: 1.0,
                position: 0.0,
                dot_list: Vec::new(),
                slow_list: Vec::new(),
            };
            let spawn_time = rng.gen_range(0..=100);
            self.board.enemy_ready2spawn.push((enemy, spawn_time));
        }
    }
}
