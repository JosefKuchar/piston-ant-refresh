extern crate piston_window;
extern crate opengl_graphics;
extern crate piston;
extern crate image;
#[macro_use]
extern crate clap;

use clap::{App, Arg, Error};
use piston_window::*;
use opengl_graphics::{ OpenGL };
use std::path::Path;
use std::fs::File;
use image::{ImageBuffer, RGBA, Rgba};
use texture::Filter;
use std::process;

struct World {
    ants: Vec<Ant>,
    grid: Grid
}

impl World {
    fn new(width: usize, height: usize) -> World {
        return World {
            ants: Vec::new(),
            grid: Grid::new(width, height)
        }
    }

    fn update(&mut self) {
        for ant in self.ants.iter_mut() {
            ant.update(&mut self.grid)
        }
    }

    fn add_ant(&mut self, ant: Ant) {
        self.ants.push(ant)
    }
}

struct Grid {
    size: Vector,
    tiles: Vec<usize>
}

impl Grid {
    fn new(width: usize, height: usize) -> Grid {
        return Grid {
            tiles: vec![0; width * height],
            size: Vector {
                x: width,
                y: height
            }
        }
    }

    fn set(&mut self, position: &mut IVector, value: usize) {
        self.bound_position(position);

        self.tiles[self.size.x * position.y as usize + position.x as usize] = value;
    }

    fn get(&self, position: &mut IVector) -> usize {
        self.bound_position(position);

        return self.tiles[self.size.x * position.y as usize + position.x as usize];
    }

    fn bound_position(&self, position: &mut IVector) {
        if position.x < 0 {
            position.x += self.size.x as isize;
        }   

        if position.y < 0 {
            position.y += self.size.y as isize;
        }

        if position.x >= self.size.x as isize {
            position.x -= self.size.x as isize;
        }

        if position.y >= self.size.y as isize {
            position.y -= self.size.y as isize;
        }
    }
}

struct IVector {
    x: isize,
    y: isize,
}

impl IVector {
    fn add(&mut self, array: [isize; 2]) {
        self.x += array[0];
        self.y += array[1];
    }
}

struct Vector {
    x: usize,
    y: usize
}

struct Ant {
    position: IVector,
    direction: usize
}

impl Ant {
    fn new(grid_size: &Vector, direction: usize) -> Ant {
        return Ant {
            position: IVector {
                x: (grid_size.x / 2) as isize,
                y: (grid_size.y / 2) as isize
            },
            direction: direction
        }
    }

    fn update(&mut self, world: &mut Grid) {
        static DIRECTIONS: [[isize; 2]; 4] = [[0, -1], [1, 0], [0, 1], [-1, 0]];
        let code = world.get(&mut self.position);

        if get_direction(code) {
            self.direction += 1;
            self.direction %= 4;
            
        } else {
            self.direction += 3;
            self.direction %= 4;
        }

        if code == 15 {
            world.set(&mut self.position, 0);
        } else {
            world.set(&mut self.position, code + 1);
        }
        
        self.position.add(DIRECTIONS[self.direction]);
    }
}

struct Application {
    world: World,
    speed: usize,
    zoom: f64,
}

impl Application {
    fn run(&mut self) {
        let mut canvas = ImageBuffer::new(self.world.grid.size.x as u32, self.world.grid.size.y as u32);
        let opengl = OpenGL::V3_2;
        let mut window: PistonWindow = WindowSettings::new("Piston Ant", [640, 480]).opengl(opengl).build().unwrap();
        let mut texture = Texture::from_image(
            &mut window.factory,
            &canvas,
            &TextureSettings::new().filter(Filter::Nearest)
        ).unwrap();

         while let Some(e) = window.next() {
        
            if let Some(_) = e.update_args() {
                for _ in 0..self.speed {
                    self.world.update();
                }
            }
        
            if let Some(_) = e.render_args() {
            
                for (index, tile) in self.world.grid.tiles.iter().enumerate() {

                    let x = index % self.world.grid.size.x;
                    let y = index / self.world.grid.size.x;
                
                    canvas.put_pixel(x as u32, y as u32, Rgba(get_color(*tile)));
                }

                texture.update(&mut window.encoder, &canvas).unwrap();

                window.draw_2d(&e, |c, g| {
                    clear([0.0; 4], g);
                    image(&texture, c.transform.scale(self.zoom, self.zoom), g);
                });
            }

            e.mouse_scroll(|_, y| {
                self.zoom += y;
                if self.zoom < 1.0 {
                    self.zoom = 1.0;
                }
            });
        }
    }

    fn generate(&mut self, cycles: usize, basepath: &Path) {
        let mut canvas = ImageBuffer::new(self.world.grid.size.x as u32 * self.zoom as u32, self.world.grid.size.y as u32 * self.zoom as u32);

        for cycle in 0..cycles {
            for _ in 0..self.speed {
                self.world.update();
            }

            for (index, tile) in self.world.grid.tiles.iter().enumerate() {
                let x = index % self.world.grid.size.x;
                let y = index / self.world.grid.size.x;

                for i in 0..self.zoom as u32 {
                    for j in 0..self.zoom as u32 {
                        canvas.put_pixel((x as u32 * self.zoom as u32) + i, (y as u32 * self.zoom as u32) + j, Rgba(get_color(*tile)));
                    }
                }
            }

            let path = basepath.join(format!("frame{}.png", cycle));
            File::create(&path).unwrap();
            
            File::create(&path).unwrap();
            image::save_buffer(&path, &canvas, self.world.grid.size.x as u32 * self.zoom as u32, self.world.grid.size.y as u32 * self.zoom as u32, RGBA(8)).unwrap();
        }
    }

    fn add_ants(&mut self) {
        for i in 0..4 {
            let ant = Ant::new(&self.world.grid.size, i);
            self.world.add_ant(ant);
        }
    }
}

fn get_color(code: usize) -> [u8; 4] {
    const COLORS: [[u8; 4]; 16] = [[255, 255, 255, 255], [255, 61, 61, 255], [255, 193, 61, 255], [225, 255, 61, 255],
                                    [148, 255, 61, 255], [61, 255, 103, 255], [61, 255, 225, 255], [61, 190, 255, 255],
                                    [61, 86, 255, 255], [141, 61, 255, 255], [229, 61, 255, 255], [255, 61, 151, 255],
                                    [26, 117, 78, 255], [79, 91, 78, 255], [99, 74, 58, 255], [0, 0, 0, 255]];
    return COLORS[code];
}

fn get_direction(code: usize) -> bool {
    const DIRECTIONS: [bool; 16] = [false, true, false, true, false, false, true, true, false, true, false, true, false, false, true, true];
    return DIRECTIONS[code];
}

fn main() {
    let matches = App::new("piston-ant")
        .version("1.0.0")
        .about("Langton's ant implementation")
        .author("Josef Kuchař")
        .arg(Arg::with_name("generate")
            .short("g")
            .long("generate")
            .value_name("PATH")
            .help("Render images into files")
            .takes_value(true))
        .arg(Arg::with_name("width")
            .short("x")
            .long("width")
            .value_name("INTEGER")
            .help("Width of canvas")
            .default_value("100")
            .takes_value(true))
        .arg(Arg::with_name("height")
            .short("y")
            .long("height")
            .value_name("INTEGER")
            .help("Height of canvas")
            .default_value("100")
            .takes_value(true))
        .arg(Arg::with_name("speed")
            .short("s")
            .long("speed")
            .value_name("INTEGER")
            .help("Iterations per update")
            .default_value("20")
            .takes_value(true))
        .arg(Arg::with_name("cycles")
            .short("c")
            .long("cycles")
            .value_name("INTEGER")
            .help("Number of cycles, only with generate option")
            .default_value("0")
            .takes_value(true))
        .arg(Arg::with_name("zoom")
            .short("z")
            .long("zoom")
            .value_name("INTEGER")
            .help("Initial zoom, also for generate option")
            .default_value("1")
            .takes_value(true))
        .get_matches();

    // Parse arguments
    let width = value_t_or_exit!(matches.value_of("width"), usize);
    let height = value_t_or_exit!(matches.value_of("height"), usize);
    let speed = value_t_or_exit!(matches.value_of("speed"), usize);
    let cycles = value_t_or_exit!(matches.value_of("cycles"), usize);
    let zoom = value_t_or_exit!(matches.value_of("zoom"), usize);

    let mut app = Application {
        world: World::new(width, height),
        speed: speed,
        zoom: zoom as f64
    };

    app.add_ants();

    if matches.value_of("generate").is_some() {
        let path = matches.value_of("generate").unwrap();
        if Path::new(path).exists() {
            app.generate(cycles, Path::new(path));
            process::exit(0);
        } else {
            Error::exit(&Error::with_description("Path is not valid", clap::ErrorKind::InvalidValue));
        }
    } else {
        app.run();
    }
}