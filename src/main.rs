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
    Pellets {
        x: u16,
        y: u16,
    },
    Hydra {
        x: u16,
        y: u16,
        fulfillment: u8,
        age: u16,
        stomach: u16,
        hv: u8,
    }, //fulment is the threshold that the animal needs before it makes a baby
    Hydraling {
        x: u16,
        y: u16,
        parent_traits: u8,
    },
    Amoeba {
        x: u16,
        y: u16,
        fulfillment: u8,
        stomach: u16,
        av: u8,
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

    fn add_food(&mut self) {
        let mut my_rng = rng();
        for _ in 0..20 {
            let x = my_rng.random_range(0..(WIDTH as u16));
            let y = my_rng.random_range(0..(HEIGHT as u16));
            let pellet = Cell::Pellets { x, y };
            self.add_animal(x, y, pellet);
        }
    }

    fn spawn_hydras(&mut self, count: u16) {
        let mut my_rng = rng();
        for _ in 0..count {
            let x = my_rng.random_range(0..(WIDTH as u16));
            let y = my_rng.random_range(0..(HEIGHT as u16));
            let hv = my_rng.random_range(1..3);
            self.add_animal(
                x,
                y,
                Cell::Hydra {
                    x,
                    y,
                    fulfillment: 0,
                    age: 0,
                    stomach: 100,
                    hv: hv,
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
                Cell::Empty => 0x1E_1E_24,            // Dark Slate Gray background
                Cell::Pellets { .. } => 0xFF_D1_66,   // Warm Yellow/Gold for food
                Cell::Hydra { .. } => 0xFF_FF_FF,     // white for Hydra
                Cell::Hydraling { .. } => 0xD3_D3_D3, // light grey for baby hydralings!
                Cell::Amoeba { .. } => 0x90_FF_F1,    //light blue for amoeba
            };
        }
    }

    fn animal_update(&mut self) {
        let mut eaten_pellets = Vec::new();
        let mut nursery: Vec<(u16, u16, u8, Cell)> = Vec::new();
        let mut my_rng = rng();

        // Phase 1: Main Simulation Processing Loop
        for (animal, x, y) in self.animal.iter_mut() {
            match animal {
                Cell::Pellets { .. } => {
                    let think = my_rng.random_range(0..4);
                    let mut target_x = *x as i16;
                    let mut target_y = *y as i16;

                    match think {
                        0 => target_y -= 1, // Up
                        1 => target_y += 1, // Down
                        2 => target_x -= 1, // Left
                        3 => target_x += 1, // Right
                        _ => {}
                    }

                    if target_x >= 0
                        && target_x < WIDTH as i16
                        && target_y >= 0
                        && target_y < HEIGHT as i16
                    {
                        let new_index = (target_y as usize * WIDTH) + target_x as usize;

                        // Only move if target space is totally clear
                        if self.tiles[new_index] == Cell::Empty {
                            let old_index = (*y as usize * WIDTH) + *x as usize;
                            self.tiles[old_index] = Cell::Empty;

                            *x = target_x as u16;
                            *y = target_y as u16;
                            *animal = Cell::Pellets { x: *x, y: *y };
                            self.tiles[new_index] = *animal;
                        }
                    } else {
                        // Out of bounds: Clear the old space and discard it
                        let old_index = (*y as usize * WIDTH) + *x as usize;
                        self.tiles[old_index] = Cell::Empty;
                        *animal = Cell::Empty;
                    }
                }

                Cell::Hydra {
                    x: _hx,
                    y: _hy,
                    fulfillment,
                    age,
                    stomach,
                    hv,
                } => {
                    *age += 1;
                    *stomach = stomach.saturating_sub(1);
                    if *age >= 1000 || *stomach == 0 {
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

                            let tx = *x as i16 + dx;
                            let ty = *y as i16 + dy;

                            if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                let target_idx = (ty as usize * WIDTH) + tx as usize;

                                // Check if we have enough energy to drop an offspring
                                // and verify if that spot is completely empty before doing so
                                if *fulfillment >= 10 && self.tiles[target_idx] == Cell::Empty {
                                    *fulfillment = 0;
                                    nursery.push((
                                        tx as u16,
                                        ty as u16,
                                        *hv,
                                        Cell::Hydraling {
                                            x: (tx as u16),
                                            y: (ty as u16),
                                            parent_traits: (*hv),
                                        },
                                    ));

                                    // Lock down the tile space immediately so nothing steps here
                                    self.tiles[target_idx] = Cell::Hydraling {
                                        x: tx as u16,
                                        y: ty as u16,
                                        parent_traits: *hv,
                                    };
                                }

                                if let Cell::Pellets { .. } = self.tiles[target_idx] {
                                    eaten_pellets.push((tx as u16, ty as u16));
                                    self.tiles[target_idx] = Cell::Empty;
                                    *fulfillment += 1;
                                    *stomach = (*stomach + 50).min(300);
                                    break 'eat;
                                }
                            }
                        }
                    }
                }

                Cell::Hydraling {
                    x: hx,
                    y: hy,
                    parent_traits,
                } => {
                    let think = my_rng.random_range(0..4);
                    let mut target_x = *hx as i16;
                    let mut target_y = *hy as i16;

                    match think {
                        0 => target_y -= 1,
                        1 => target_y += 1,
                        2 => target_x -= 1,
                        3 => target_x += 1,
                        _ => {}
                    }

                    if target_x >= 0
                        && target_x < WIDTH as i16
                        && target_y >= 0
                        && target_y < HEIGHT as i16
                    {
                        let new_index = (target_y as usize * WIDTH) + target_x as usize;

                        if self.tiles[new_index] == Cell::Empty {
                            let old_index = (*y as usize * WIDTH) + *x as usize;
                            self.tiles[old_index] = Cell::Empty;

                            *x = target_x as u16;
                            *y = target_y as u16;

                            if my_rng.random_range(0..15) == 0 {
                                let mut mutation: i8 = my_rng.random_range(-1..=1);
                                if mutation == -1 && *parent_traits == 1
                                    || mutation == 1 && *parent_traits == 5
                                {
                                    mutation = 0;
                                }
                                *animal = Cell::Hydra {
                                    x: *x,
                                    y: *y,
                                    fulfillment: 0,
                                    age: 0,
                                    stomach: 100,
                                    hv: (*parent_traits as i8 + mutation) as u8,
                                };
                                self.tiles[new_index] = *animal;
                                continue;
                            }

                            *animal = Cell::Hydraling {
                                x: *x,
                                y: *y,
                                parent_traits: *parent_traits,
                            };
                            self.tiles[new_index] = *animal;
                        }
                    } else {
                        let old_index = (*y as usize * WIDTH) + *x as usize;
                        self.tiles[old_index] = Cell::Empty;
                        *animal = Cell::Empty;
                    }
                }
                Cell::Amoeba {
                    x: ax,
                    y: ay,
                    fulfillment,
                    stomach,
                    av,
                } => {
                    *stomach = stomach.saturating_sub(1);

                    if *stomach == 0 {
                        let death_index = (*ay as usize * WIDTH) + *ax as usize;
                        self.tiles[death_index] = Cell::Empty;
                        *animal = Cell::Empty;
                        continue;
                    }

                    if *fulfillment >= 20 {
                        'spawn: for dy in -1..=1 {
                            for dx in -1..=1 {
                                let tx = *ax as i16 + dx;
                                let ty = *ay as i16 + dy;
                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let target_idx = (ty as usize * WIDTH) + tx as usize;
                                    if self.tiles[target_idx] == Cell::Empty {
                                        *fulfillment = 0;
                                        nursery.push((
                                            tx as u16,
                                            ty as u16,
                                            *av,
                                            Cell::Amoeba {
                                                x: (tx as u16),
                                                y: (ty as u16),
                                                fulfillment: (0),
                                                stomach: (100),
                                                av: (*av),
                                            },
                                        ));
                                        self.tiles[target_idx] = Cell::Amoeba {
                                            x: tx as u16,
                                            y: ty as u16,
                                            fulfillment: 0,
                                            stomach: 100,
                                            av: *av,
                                        };
                                        break 'spawn;
                                    }
                                }
                            }
                        }
                    }

                    //food sensor syetem
                    let mut target_move: Option<(i16, i16)> = None;

                    // Check if amoeba hungry
                    if *stomach <= 120 {
                        'radar: for dy in -(*av as i16)..=(*av as i16) {
                            for dx in -(*av as i16)..=(*av as i16) {
                                let tx = *ax as i16 + dx;
                                let ty = *ay as i16 + dy;
                                if tx >= 0 && tx < WIDTH as i16 && ty >= 0 && ty < HEIGHT as i16 {
                                    let check_idx = (ty as usize * WIDTH) + tx as usize;
                                    match self.tiles[check_idx] {
                                        Cell::Pellets { .. } | Cell::Hydraling { .. } => {
                                            let step_x = dx.signum();
                                            let step_y = dy.signum();
                                            target_move = Some((step_x, step_y));
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
                        let old_index = (*ay as usize * WIDTH) + *ax as usize;

                        match self.tiles[new_index] {
                            Cell::Empty => {
                                // Move safely into space
                                self.tiles[old_index] = Cell::Empty;
                                *ax = target_x as u16;
                                *ay = target_y as u16;
                                *animal = Cell::Amoeba {
                                    x: *ax,
                                    y: *ay,
                                    fulfillment: *fulfillment,
                                    stomach: *stomach,
                                    av: *av,
                                };
                                self.tiles[new_index] = *animal;
                            }
                            Cell::Pellets { .. } | Cell::Hydraling { .. } => {
                                // HUNT SUCCESSFUL!
                                eaten_pellets.push((target_x as u16, target_y as u16));
                                self.tiles[old_index] = Cell::Empty;

                                *ax = target_x as u16;
                                *ay = target_y as u16;
                                *fulfillment += 1;
                                *stomach = (*stomach + 40).min(200);

                                *animal = Cell::Amoeba {
                                    x: *ax,
                                    y: *ay,
                                    fulfillment: *fulfillment,
                                    stomach: *stomach,
                                    av: *av,
                                };
                                self.tiles[new_index] = *animal;
                            }
                            _ => {} // Blocked by wall or adult Hydra
                        }
                    }
                }
                _ => {}
            }
        }

        //it removes the eaten animals from the system
        for (animal, ax, ay) in self.animal.iter_mut() {
            if let Cell::Pellets { .. } = animal {
                if eaten_pellets.contains(&(*ax, *ay)) {
                    *animal = Cell::Empty;
                }
            }
        }

        //Safely unload the Nursery without method conflicts
        for (tx, ty, hv, cell) in nursery {
            match cell {
                Cell::Hydraling { .. } => {
                    let baby = Cell::Hydraling {
                        x: tx,
                        y: ty,
                        parent_traits: hv,
                    };
                    self.animal.push((baby, tx, ty));
                }
                Cell::Amoeba { .. } => {
                    let baby = Cell::Amoeba {
                        x: tx,
                        y: ty,
                        fulfillment: 0,
                        stomach: 100,
                        av: hv,
                    };
                    self.animal.push((baby, tx, ty));
                }
                _ => {}
            }
        }

        //garbage collection and shuffle
        self.animal
            .retain(|(animal_type, _, _)| *animal_type != Cell::Empty);
        self.animal.shuffle(&mut my_rng);
    }
}

fn main() {
    let mut grid = Grid::new();

    // Spawn the food once before entering the main loop
    grid.add_food();
    let mut food_buffer = 0;

    //for testing making a point hydra
    grid.spawn_hydras(3);

    grid.add_animal(
        WIDTH as u16 / 2,
        HEIGHT as u16 / 2,
        Cell::Amoeba {
            x: (WIDTH as u16 / 2),
            y: (HEIGHT as u16 / 2),
            fulfillment: (0),
            stomach: (100),
            av: (2),
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
    window.set_target_fps(60);

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

        if food_buffer == 20 {
            grid.add_food();
            food_buffer = 0;
        }
        food_buffer = food_buffer + 1;
    }
}
