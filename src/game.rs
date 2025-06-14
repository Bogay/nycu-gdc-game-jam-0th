#[derive(Debug)]
pub struct Game {
    pub level: usize,
    pub game_state: GameState,
    pub borad: Board,
    pub cursor: (usize, usize),
    pub selected: Option<(usize, usize)>,
    pub coin: usize,
}

#[derive(Debug, Default)]
enum GameState {
    #[default]
    Init,
    Running,
    Pause,
    End,
}

#[derive(Debug)]
struct Board {
    pub ally_grid: Vec<Vec<Option<Ally>>>,
    pub enemies: Vec<Enemy>,
}

#[derive(Debug)]
struct Ally {
    pub element: AllyElement,
    pub second_element: Option<AllyElement>,
    pub atk: usize,
    pub range: usize,
    pub aoe_range: usize,
    pub level: usize,
    pub atk_speed: f32,
    pub attack_cooldown: f32,
}

#[derive(Debug)]
pub enum AllyElement {
    Basic,
    Slow,
    Aoe,
    Dot,
    Critical,
}

#[derive(Debug)]
struct Enemy {
    pub hp: usize,
    pub move_speed: f32,
    pub position: f32,

    pub dot_list: Vec<Debuff>,
    pub slow_list: Vec<Debuff>,
}

#[derive(Debug)]
pub struct Debuff {
    pub value: usize,
    pub cooldown: f32,
}
