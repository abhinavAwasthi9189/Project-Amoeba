//project amoeba is a life simulator. a very simple one at that but still it works
//it will have simple grid animals
//we will start with simple things ability to find food and asexual reproduction, later moving towards natural selection and muations

use minifb::{Key, Window, WindowOptions};
use rand::RngExt;
use rand::rng;
use rand::seq::SliceRandom;

const HEIGHT: usize = 32;
const WIDTH: usize = 64;

#[derive(Clone, Copy, PartialEq, Debug)]

pub enum Cell {
    Empty,
    Cyanobacteria {
        x: u16,
        y: u16,
        fulfillment: u8,
        av: u8,
        iq: f32,
    },
    Hydra {
        x: u16,
        y: u16,
        fulfillment: u8,
        age: u16,
        stomach: u16,
        hv: u8,
        iq: f32,
    }, //fulment is the threshold that the animal needs before it makes a baby
    Hydraling {
        x: u16,
        y: u16,
        parent_traits: u8,
        iq: f32,
    },
    Amoeba {
        x: u16,
        y: u16,
        fulfillment: u8,
        age:u8,
        stomach: u16,
        av: u8,
        iq: f32,
    },
}

struct Grid {
    tiles: [Cell; HEIGHT * WIDTH],
    // Storing the cell type along with its current coordinates. this is 1d array so to move a block up -64. for down +64. else if simple.
    animal: Vec<(Cell, u16, u16)>,
}

impl Grid {
    fn new() -> Self {
        Grid {
            tiles: [Cell::Empty; HEIGHT * WIDTH],
            animal: Vec::new(),
        }
    }

    fn add_cyano(&mut self) {
        let mut my_rng = rng();
        for _ in 0..50 {
            let x = my_rng.random_range(0..(WIDTH as u16));
            let y = my_rng.random_range(0..(HEIGHT as u16));
            let av = my_rng.random_range(1..2);
            let iq = my_rng.random_range(1..2) as f32 / 10 as f32;
            let bacteria = Cell::Cyanobacteria {
                x,
                y,
                fulfillment: 0,
                av,
                iq,
            };
            self.add_animal(x, y, bacteria);
        }
    }

    fn spawn_hydras(&mut self, count: u16) {
        let mut my_rng = rng();
        for _ in 0..count {
            let x = my_rng.random_range(0..(WIDTH as u16));
            let y = my_rng.random_range(0..(HEIGHT as u16));
            let hv = my_rng.random_range(1..3);
            let iq = my_rng.random_range(1..3) as f32 / 10 as f32;
            self.add_animal(
                x,
                y,
                Cell::Hydra {
                    x,
                    y,
                    fulfillment: 0,
                    age: 0,
                    stomach: 100,
                    hv,
                    iq,
                },
            );
        }
    }

    fn add_animal(&mut self, x: u16, y: u16, animal_type: Cell) {
        let index = (y as usize * WIDTH) + x as usize;
        if self.tiles[index] == Cell::Empty {
            self.tiles[index] = animal_type;
            self.animal.push((animal_type, x, y));
        }
    }

    fn render_to_buffer(&self, buffer: &mut [u32; WIDTH * HEIGHT]) {
        for (i, tile) in self.tiles.iter().enumerate() {
            buffer[i] = match tile {
                Cell::Empty => 0x1E_1E_24,                // Dark Slate Gray background
                Cell::Cyanobacteria { .. } => 0xFF_D1_66, // Warm Yellow/Gold for bacteria
                Cell::Hydra { .. } => 0xFF_FF_FF,         // White for Hydra
                Cell::Hydraling { .. } => 0xD3_D3_D3,     // Light Gray for baby hydralings!
                Cell::Amoeba { .. } => 0x90_FF_F1,        // Light Blue for Amoeba
            };
        }
    }

    fn animal_update(&mut self) {
        let mut eaten_pellets = Vec::new();
        let mut nursery: Vec<(u16, u16, u8, f32, Cell)> = Vec::new();
        let mut my_rng = rng();

        // Phase 1: Main Simulation Processing Loop
        for (animal, x, y) in self.animal.iter_mut() {
            match animal {
                Cell::Cyanobacteria {
                    x: ax,
                    y: ay,
                    fulfillment,
                    av,
                    iq,
                } => {
                    // Safe integer reference mutation
                    *fulfillment = fulfillment.saturating_add(1);

                    if *fulfillment >= 80 {
                        'spawn: for dy in -1..=1 {
                            for dx in -1..=1 {
                                let tx = *ax as i16 + dx;
                                let ty = *ay as i16 + dy;
                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let target_idx = (ty as usize * WIDTH) + tx as usize;
                                    if self.tiles[target_idx] == Cell::Empty {
                                        *fulfillment = 0;
                                        let mut mutation: i8 = my_rng.random_range(-1..=1);
                                        if (mutation == -1 && *av == 1)
                                            || (mutation == 1 && *av == 2)
                                        {
                                            mutation = 0;
                                        }
                                        let mut qmutation: f32 = my_rng.random_range(-1..=1) as f32;
                                        if (qmutation == -1.0 && *iq == 0.1)
                                            || (qmutation == 1.0 && *iq == 0.25)
                                        {
                                            qmutation = 0.0;
                                        }

                                        let child = Cell::Cyanobacteria {
                                            x: tx as u16,
                                            y: ty as u16,
                                            fulfillment: 0,
                                            av: (*av as i8 + mutation) as u8,
                                            iq: (*iq + qmutation / 10.0),
                                        };

                                        nursery.push((
                                            tx as u16,
                                            ty as u16,
                                            (*av as i8 + mutation) as u8,
                                            *iq + qmutation / 10.0,
                                            child,
                                        ));
                                        self.tiles[target_idx] = child;
                                        break 'spawn;
                                    }
                                }
                            }
                        }
                    }

                    // --- Predator Avoidance Scan ---
                    let mut escape_move: Option<(i16, i16)> = None;

                    if my_rng.random_bool(*iq as f64) {
                        let mut danger_x = 0;
                        let mut danger_y = 0;
                        let mut found_danger = false;

                        for dy in -(*av as i16)..=(*av as i16) {
                            for dx in -(*av as i16)..=(*av as i16) {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let tx = *ax as i16 + dx;
                                let ty = *ay as i16 + dy;

                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let check_idx = (ty as usize * WIDTH) + tx as usize;
                                    match self.tiles[check_idx] {
                                        Cell::Hydra { .. } | Cell::Amoeba { .. } => {
                                            danger_x += dx;
                                            danger_y += dy;
                                            found_danger = true;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        if found_danger {
                            escape_move = Some((-danger_x.signum(), -danger_y.signum()));
                        }
                    }

                    // --- Determine Final Path Step ---
                    let (mx, my) = match escape_move {
                        Some((rx, ry)) => (rx, ry),
                        None => {
                            let think = my_rng.random_range(0..4);
                            match think {
                                0 => (0, -1), // Up
                                1 => (0, 1),  // Down
                                2 => (-1, 0), // Left
                                3 => (1, 0),  // Right
                                _ => (0, 0),
                            }
                        }
                    };

                    let target_x = *ax as i16 + mx;
                    let target_y = *ay as i16 + my;

                    if target_x >= 0
                        && target_x < WIDTH as i16
                        && target_y >= 0
                        && target_y < HEIGHT as i16
                    {
                        let new_index = (target_y as usize * WIDTH) + target_x as usize;

                        if self.tiles[new_index] == Cell::Empty {
                            let old_index = (*ay as usize * WIDTH) + *ax as usize;
                            self.tiles[old_index] = Cell::Empty;

                            *ax = target_x as u16;
                            *ay = target_y as u16;
                            *x = *ax; // Update tracker references
                            *y = *ay;

                            *animal = Cell::Cyanobacteria {
                                x: *ax,
                                y: *ay,
                                fulfillment: *fulfillment,
                                av: *av,
                                iq: *iq,
                            };
                            self.tiles[new_index] = *animal;
                        }
                    }
                }

                Cell::Hydra {
                    x: hx,
                    y: hy,
                    fulfillment,
                    age,
                    stomach,
                    hv,
                    iq,
                } => {
                    *age += 1;
                    *stomach = stomach.saturating_sub(1);
                    if *age >= 200|| *stomach == 0 {
                        let death_index = (*y as usize * WIDTH) + *x as usize;
                        self.tiles[death_index] = Cell::Empty;
                        *animal = Cell::Empty;
                        continue;
                    }

                    'eat: for dy in -(*hv as i16)..=(*hv as i16) {
                        for dx in -(*hv as i16)..=(*hv as i16) {
                            if dx == 0 && dy == 0 {
                                continue;
                            }

                            let tx = *hx as i16 + dx;
                            let ty = *hy as i16 + dy;

                            if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                let target_idx = (ty as usize * WIDTH) + tx as usize;

                                if *fulfillment >= 3 && self.tiles[target_idx] == Cell::Empty {
                                    *fulfillment = 0;
                                    let mut mutation: i8 = my_rng.random_range(-1..=1);
                                    if (mutation == -1 && *hv == 1) || (mutation == 1 && *hv == 5) {
                                        mutation = 0;
                                    }
                                    let mut qmutation: f32 = my_rng.random_range(-1..=1) as f32;
                                    if (qmutation == -1.0 && *iq == 0.1)
                                        || (qmutation == 1.0 && *iq == 0.5)
                                    {
                                        qmutation = 0.0;
                                    }
                                    let baby = Cell::Hydraling {
                                        x: tx as u16,
                                        y: ty as u16,
                                        parent_traits: (*hv as i8 + mutation) as u8,
                                        iq: (*iq + qmutation / 10.0),
                                    };
                                    nursery.push((
                                        tx as u16,
                                        ty as u16,
                                        (*hv as i8 + mutation) as u8,
                                        (*iq + qmutation / 10.0),
                                        baby,
                                    ));
                                    self.tiles[target_idx] = baby;
                                }

                                if let Cell::Cyanobacteria { .. } = self.tiles[target_idx] {
                                    let hunt_probability = ((*iq * 2.0) as f64).clamp(0.0, 1.0);

                                    if my_rng.random_bool(hunt_probability) {
                                        eaten_pellets.push((tx as u16, ty as u16));
                                        self.tiles[target_idx] = Cell::Empty;
                                        *fulfillment += 1;
                                        *stomach = (*stomach + 50).min(150);
                                        break 'eat;
                                    } else {
                                        *stomach = stomach.saturating_sub(1);
                                    }
                                }
                            }
                        }
                    }
                }

                Cell::Hydraling {
                    x: hx,
                    y: hy,
                    parent_traits,
                    iq,
                } => {
                    //---Predator Avoidance Scan ---
                    let mut escape_move: Option<(i16, i16)> = None;

                    if my_rng.random_bool(*iq as f64) {
                        let mut danger_x = 0;
                        let mut danger_y = 0;
                        let mut found_danger = false;

                        for dy in -(*parent_traits as i16)..=(*parent_traits as i16) {
                            for dx in -(*parent_traits as i16)..=(*parent_traits as i16) {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }
                                let tx = *hx as i16 + dx;
                                let ty = *hy as i16 + dy;

                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let check_idx = (ty as usize * WIDTH) + tx as usize;
                                    match self.tiles[check_idx] {
                                        Cell::Hydra { .. } | Cell::Amoeba { .. } => {
                                            danger_x += dx;
                                            danger_y += dy;
                                            found_danger = true;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        if found_danger {
                            escape_move = Some((-danger_x.signum(), -danger_y.signum()));
                        }
                    }

                    //Determine Final Path Step ---
                    let (mx, my) = match escape_move {
                        Some((rx, ry)) => (rx, ry),
                        None => {
                            let think = my_rng.random_range(0..4);
                            match think {
                                0 => (0, -1), // Up
                                1 => (0, 1),  // Down
                                2 => (-1, 0), // Left
                                3 => (1, 0),  // Right
                                _ => (0, 0),
                            }
                        }
                    };

                    let target_x = *hx as i16 + mx;
                    let target_y = *hy as i16 + my;

                    if target_x >= 0
                        && target_x < WIDTH as i16
                        && target_y >= 0
                        && target_y < HEIGHT as i16
                    {
                        let new_index = (target_y as usize * WIDTH) + target_x as usize;

                        if self.tiles[new_index] == Cell::Empty {
                            let old_index = (*hy as usize * WIDTH) + *hx as usize;
                            self.tiles[old_index] = Cell::Empty;

                            *hx = target_x as u16;
                            *hy = target_y as u16;
                            *x = *hx;
                            *y = *hy;

                            if my_rng.random_range(0..10) == 0 {
                                *animal = Cell::Hydra {
                                    x: *hx,
                                    y: *hy,
                                    fulfillment: 0,
                                    age: 0,
                                    stomach: 100,
                                    hv: *parent_traits,
                                    iq: *iq,
                                };
                                self.tiles[new_index] = *animal;
                                continue;
                            }

                            *animal = Cell::Hydraling {
                                x: *hx,
                                y: *hy,
                                parent_traits: *parent_traits,
                                iq: *iq,
                            };
                            self.tiles[new_index] = *animal;
                        }
                    } else {
                        let old_index = (*hy as usize * WIDTH) + *hx as usize;
                        self.tiles[old_index] = Cell::Empty;
                        *animal = Cell::Empty;
                    }
                }

                Cell::Amoeba {
                    x: ax,
                    y: ay,
                    fulfillment,
                    age,
                    stomach,
                    av,
                    iq,
                } => {
                    *stomach = stomach.saturating_sub(1);
                    *age = age.saturating_add(1);

                    if *stomach == 0 || *age >70 {
                        let death_index = (*ay as usize * WIDTH) + *ax as usize;
                        self.tiles[death_index] = Cell::Empty;
                        *animal = Cell::Empty;
                        continue;
                    }

                    if *fulfillment >= 5 {
                        'spawn: for dy in -1..=1 {
                            for dx in -1..=1 {
                                let tx = *ax as i16 + dx;
                                let ty = *ay as i16 + dy;
                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let target_idx = (ty as usize * WIDTH) + tx as usize;
                                    if self.tiles[target_idx] == Cell::Empty {
                                        *fulfillment = 0;
                                        let mut mutation: i8 = my_rng.random_range(-1..=1);
                                        if (mutation == -1 && *av == 1)
                                            || (mutation == 1 && *av == 4)
                                        {
                                            mutation = 0;
                                        }
                                        let mut qmutation: f32 = my_rng.random_range(-1..=1) as f32;
                                        if (qmutation == -1.0 && *iq == 0.2)
                                            || (qmutation == 1.0 && *iq == 0.5)
                                        {
                                            qmutation = 0.0;
                                        }
                                        let baby = Cell::Amoeba {
                                            x: tx as u16,
                                            y: ty as u16,
                                            fulfillment: 0,
                                            age:0,
                                            stomach: 100,
                                            av: (*av as i8 + mutation) as u8,
                                            iq: (*iq + qmutation / 10.0),
                                        };
                                        nursery.push((
                                            tx as u16,
                                            ty as u16,
                                            (*av as i8 + mutation) as u8,
                                            (*iq + qmutation / 10.0),
                                            baby,
                                        ));
                                        self.tiles[target_idx] = baby;
                                        break 'spawn;
                                    }
                                }
                            }
                        }
                    }

                    let mut target_move: Option<(i16, i16)> = None;

                    if *stomach <= 170 {
                        'radar: for dy in -(*av as i16)..=(*av as i16) {
                            for dx in -(*av as i16)..=(*av as i16) {
                                let tx = *ax as i16 + dx;
                                let ty = *ay as i16 + dy;
                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let check_idx = (ty as usize * WIDTH) + tx as usize;
                                    match self.tiles[check_idx] {
                                        Cell::Cyanobacteria { .. } | Cell::Hydraling { .. } => {
                                            target_move = Some((dx.signum(), dy.signum()));
                                            break 'radar;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                    let (mx, my) = match target_move {
                        Some((sx, sy)) => (sx, sy),
                        None => {
                            let think = my_rng.random_range(0..4);
                            match think {
                                0 => (0, -1),
                                1 => (0, 1),
                                2 => (-1, 0),
                                3 => (1, 0),
                                _ => (0, 0),
                            }
                        }
                    };

                    let target_x = *ax as i16 + mx;
                    let target_y = *ay as i16 + my;

                    if target_x >= 0
                        && target_x < WIDTH as i16
                        && target_y >= 0
                        && target_y < HEIGHT as i16
                    {
                        let new_index = (target_y as usize * WIDTH) + target_x as usize;
                        let old_index = (*ay as usize * WIDTH) + *ax as usize;

                        match self.tiles[new_index] {
                            Cell::Empty => {
                                self.tiles[old_index] = Cell::Empty;
                                *ax = target_x as u16;
                                *ay = target_y as u16;
                                *x = *ax;
                                *y = *ay;
                                *animal = Cell::Amoeba {
                                    x: *ax,
                                    y: *ay,
                                    fulfillment: *fulfillment,
                                    age:*age,
                                    stomach: *stomach,
                                    av: *av,
                                    iq: *iq,
                                };
                                self.tiles[new_index] = *animal;
                            }
                            Cell::Cyanobacteria { .. } | Cell::Hydraling { .. } => {
                                // 🎯 Catch success check based on 1.5x IQ
                                let hunt_probability = ((*iq * 2.0) as f64).clamp(0.0, 1.0);

                                if my_rng.random_bool(hunt_probability) {
                                    // HUNT SUCCESSFUL!
                                    eaten_pellets.push((target_x as u16, target_y as u16));
                                    self.tiles[old_index] = Cell::Empty;

                                    *ax = target_x as u16;
                                    *ay = target_y as u16;
                                    *x = *ax;
                                    *y = *ay;
                                    *fulfillment += 1;
                                    *stomach = (*stomach + 40).min(200);

                                    *animal = Cell::Amoeba {
                                        x: *ax,
                                        y: *ay,
                                        fulfillment: *fulfillment,
                                        age:*age,
                                        stomach: *stomach,
                                        av: *av,
                                        iq: *iq,
                                    };
                                    self.tiles[new_index] = *animal;
                                } else {
                                    *stomach = stomach.saturating_sub(1);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // --- Phase 2: Structural Sweep Removal ---
        for (animal, ax, ay) in self.animal.iter_mut() {
            match animal {
                Cell::Cyanobacteria { .. } | Cell::Hydraling { .. } => {
                    if eaten_pellets.contains(&(*ax, *ay)) {
                        *animal = Cell::Empty;
                    }
                }
                _ => {}
            }
        }

        // --- Phase 3: Nursery Matching Dispatch ---
        for (tx, ty, hv, iq, cell) in nursery {
            match cell {
                Cell::Hydraling { .. } => {
                    self.animal.push((
                        Cell::Hydraling {
                            x: tx,
                            y: ty,
                            parent_traits: hv,
                            iq: iq,
                        },
                        tx,
                        ty,
                    ));
                }
                Cell::Amoeba { .. } => {
                    self.animal.push((
                        Cell::Amoeba {
                            x: tx,
                            y: ty,
                            fulfillment: 0,
                            age:0,
                            stomach: 150,
                            av: hv,
                            iq: iq,
                        },
                        tx,
                        ty,
                    ));
                }
                Cell::Cyanobacteria { .. } => {
                    self.animal.push((
                        Cell::Cyanobacteria {
                            x: tx,
                            y: ty,
                            fulfillment: 0,
                            av: hv,
                            iq: iq,
                        },
                        tx,
                        ty,
                    ));
                }
                _ => {}
            }
        }

        // --- Phase 4: Collection & Shuffle ---
        self.animal
            .retain(|(animal_type, _, _)| *animal_type != Cell::Empty);
        self.animal.shuffle(&mut my_rng);
    }
}

fn main() {
    let mut grid = Grid::new();

    // Spawn the food once before entering the main loop
    grid.add_cyano();

    //for testing making a point hydra
    grid.spawn_hydras(3);

    grid.add_animal(
        WIDTH as u16 / 2,
        HEIGHT as u16 / 2,
        Cell::Amoeba {
            x: (WIDTH as u16 / 2),
            y: (HEIGHT as u16 / 2),
            fulfillment: (0),
            age:0,
            stomach: (150),
            av: (2),
            iq: (0.3),
        },
    );
    grid.add_animal(
        WIDTH as u16 / 2,
        HEIGHT as u16 / 2,
        Cell::Amoeba {
            x: (WIDTH as u16).saturating_sub(10),
            y: (HEIGHT as u16).saturating_sub(10),
            fulfillment: (0),
            age:0,
            stomach: (150),
            av: (2),
            iq: (0.3),
        },
    );

    let mut pixel_buffer = [0u32; WIDTH * HEIGHT];

    // 2. Initialize the minifb window
    let mut window = Window::new(
        "Project Amoeba - Simulation",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X8, // This scales our 64x32 window up smoothly
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("Failed to create window: {}", e);
    });

    // Limit the loop to run around 2fps for now, so that some error might not run too fast and burn my pc
    window.set_target_fps(4);

    // 3. The Main Game Loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- UPDATE SIMULATION SPACE ---
        grid.animal_update();

        // --- RENDER SPACE ---
        // Convert the structural layout changes into raw pixel color data
        grid.render_to_buffer(&mut pixel_buffer);

        // updating the screen
        window
            .update_with_buffer(&pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
