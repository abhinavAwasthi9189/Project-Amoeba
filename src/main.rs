//project amoeba is a life simulator. a very simple one at that but still it works
//it will have simple grid animals
//we will start with simple things ability to find food and asexual reproduction, later moving towards natural selection and muations

use minifb::{Key, Window, WindowOptions};
use rand::RngExt;
use rand::rng;
use rand::seq::SliceRandom;

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
        stomach:u16,
    }, //fulment is the threshold that the animal needs before it makes a baby
    Hydraling {
        x: u16,
        y: u16,
    },
}

struct Grid {
    tiles: [Cell; 64 * 32],
    // Storing the cell type along with its current coordinates. this is 1d array so to move a block up -64. for down +64. else if simple.
    animal: Vec<(Cell, u16, u16)>,
}

impl Grid {
    fn new() -> Self {
        Grid {
            tiles: [Cell::Empty; 64 * 32],
            animal: Vec::new(),
        }
    }

    fn add_food(&mut self) {
        let mut my_rng = rng();
        for _ in 0..20 {
            let x = my_rng.random_range(0..64);
            let y = my_rng.random_range(0..32);
            let pellet = Cell::Pellets { x, y };
            self.add_animal(x, y, pellet);
        }
    }

    fn spawn_hydras(&mut self, count: u16) {
        let mut my_rng = rng();
        for _ in 0..count {
            let x = my_rng.random_range(0..64);
            let y = my_rng.random_range(0..32);
            self.add_animal(
                x,
                y,
                Cell::Hydra {
                    x,
                    y,
                    fulfillment: 0,
                    age: 0,
                    stomach:100,
                },
            );
        }
    }



    fn add_animal(&mut self, x: u16, y: u16, animal_type: Cell) {
        let index = (y as usize * 64) + x as usize;
        if self.tiles[index] == Cell::Empty {
            self.tiles[index] = animal_type;
            self.animal.push((animal_type, x, y));
        }
    }

    fn render_to_buffer(&self, buffer: &mut [u32; 64 * 32]) {
        for (i, tile) in self.tiles.iter().enumerate() {
            buffer[i] = match tile {
                Cell::Empty => 0x1E_1E_24,            // Dark Slate Gray background
                Cell::Pellets { .. } => 0xFF_D1_66,   // Warm Yellow/Gold for food
                Cell::Hydra { .. } => 0x06_D6_A0,     // Vibrant Mint Green for Hydra
                Cell::Hydraling { .. } => 0x72_EF_DD, // Bright Neon Turquoise for baby hydralings!
            };
        }
    }

    fn animal_update(&mut self) {
        let mut eaten_pellets = Vec::new();
        let mut nursery = Vec::new();
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

                    if target_x >= 0 && target_x < 64 && target_y >= 0 && target_y < 32 {
                        let new_index = (target_y as usize * 64) + target_x as usize;

                        // Only move if target space is totally clear
                        if self.tiles[new_index] == Cell::Empty {
                            let old_index = (*y as usize * 64) + *x as usize;
                            self.tiles[old_index] = Cell::Empty;

                            *x = target_x as u16;
                            *y = target_y as u16;
                            *animal = Cell::Pellets { x: *x, y: *y };
                            self.tiles[new_index] = *animal;
                        }
                    } else {
                        // Out of bounds: Clear the old space and discard it
                        let old_index = (*y as usize * 64) + *x as usize;
                        self.tiles[old_index] = Cell::Empty;
                        *animal = Cell::Empty;
                    }
                }

                Cell::Hydra {
                    x: hx,
                    y: hy,
                    fulfillment,
                    age,
                    stomach
                } => {


                    *age += 1;
                    *stomach -=1;
                    if *age >= 1000 || *stomach==0{
                        let death_index = (*hy as usize * 64) + *hx as usize;
                        self.tiles[death_index] = Cell::Empty;
                        *animal = Cell::Empty;
                        continue;
                    }

                    'eat: for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }

                            let tx = *x as i16 + dx;
                            let ty = *y as i16 + dy;

                            if tx >= 0 && tx < 64 && ty >= 0 && ty < 32 {
                                let target_idx = (ty as usize * 64) + tx as usize;

                                // Check if we have enough energy to drop an offspring
                                // and verify if that spot is completely empty before doing so
                                if *fulfillment >= 10 && self.tiles[target_idx] == Cell::Empty {
                                    *fulfillment = 0;
                                    nursery.push((tx as u16, ty as u16));

                                    // Lock down the tile space immediately so nothing steps here
                                    self.tiles[target_idx] = Cell::Hydraling {
                                        x: tx as u16,
                                        y: ty as u16,
                                    };
                                }

                                if let Cell::Pellets { .. } = self.tiles[target_idx] {
                                    eaten_pellets.push((tx as u16, ty as u16));
                                    self.tiles[target_idx] = Cell::Empty;
                                    *fulfillment += 1;
                                    *stomach +=50;
                                    break 'eat;
                                }
                            }
                        }
                    }
                }

                Cell::Hydraling { .. } => {
                    let think = my_rng.random_range(0..4);
                    let mut target_x = *x as i16;
                    let mut target_y = *y as i16;

                    match think {
                        0 => target_y -= 1,
                        1 => target_y += 1,
                        2 => target_x -= 1,
                        3 => target_x += 1,
                        _ => {}
                    }

                    if target_x >= 0 && target_x < 64 && target_y >= 0 && target_y < 32 {
                        let new_index = (target_y as usize * 64) + target_x as usize;

                        if self.tiles[new_index] == Cell::Empty {
                            let old_index = (*y as usize * 64) + *x as usize;
                            self.tiles[old_index] = Cell::Empty;

                            *x = target_x as u16;
                            *y = target_y as u16;

                            if my_rng.random_range(0..15) == 0 {
                                *animal = Cell::Hydra {
                                    x: *x,
                                    y: *y,
                                    fulfillment: 0,
                                    age:0,
                                    stomach:100
                                };
                                self.tiles[new_index] = *animal;
                                continue;
                            }

                            *animal = Cell::Hydraling { x: *x, y: *y };
                            self.tiles[new_index] = *animal;
                        }
                    } else {
                        let old_index = (*y as usize * 64) + *x as usize;
                        self.tiles[old_index] = Cell::Empty;
                        *animal = Cell::Empty;
                    }
                }
                _ => {}
            }
        }

        // Phase 2: Sweep and remove eaten food items
        for (animal, ax, ay) in self.animal.iter_mut() {
            if let Cell::Pellets { .. } = animal {
                if eaten_pellets.contains(&(*ax, *ay)) {
                    *animal = Cell::Empty;
                }
            }
        }

        // Phase 3: Safely unload the Nursery without method conflicts
        for (tx, ty) in nursery {
            let baby = Cell::Hydraling { x: tx, y: ty };
            self.animal.push((baby, tx, ty));
        }

        // Phase 4: Garbage collection and shuffle
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

    let mut pixel_buffer = [0u32; 64 * 32];

    // 2. Initialize the minifb window
    let mut window = Window::new(
        "Project Amoeba - Simulation",
        64,
        32,
        WindowOptions {
            scale: minifb::Scale::X8, // This scales our 64x32 window up smoothly
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("Failed to create window: {}", e);
    });

    // Limit the loop to run around 2fps for now, so that some error might not run too fast and burn my pc
    window.set_target_fps(10);

    // 3. The Main Game Loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- UPDATE SIMULATION SPACE ---
        grid.animal_update();

        // --- RENDER SPACE ---
        // Convert the structural layout changes into raw pixel color data
        grid.render_to_buffer(&mut pixel_buffer);

        // updating the screen
        window.update_with_buffer(&pixel_buffer, 64, 32).unwrap();

        if food_buffer == 20 {
            grid.add_food();
            food_buffer = 0;
        }
        food_buffer = food_buffer + 1;
    }
}
