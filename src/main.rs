use macroquad::{
    input::is_key_down,
    prelude::{
        draw_arc, draw_circle, draw_circle_lines, draw_rectangle, is_key_pressed, next_frame,
        screen_height, screen_width, Color, KeyCode, Vec2, BLACK, GREEN, RED, WHITE,
    },
};
use rand::{thread_rng, Rng};
use std::collections::HashMap;

const SHIP_SIZE: f32 = 10.0;
const NUM_ASTEROIDS: usize = 10;
const ARC_GROWTH_RATE: f32 = 1.0;
const SCAN_SPEED: usize = 7;
const ANGLE_PRECISION: usize = 100;
const CLOSENESS_TOLERANCE: f32 = 0.3;

// struct Radar {}
// impl Iterator for Radar {
// TODO: for pixel in Radar
//          if pixel also in an asteroid
//              collisions.push(asteroid)
//       for asteroid in collisions:
//          draw asteroid
// }

#[derive(Debug)]
struct Line {
    a: Vec2,
    b: Vec2,
}

impl Line {
    fn new(a: Vec2, b: Vec2) -> Self {
        if a.x == b.x && a.y == b.y {
            panic!("unable to form lines, a and b are the same point");
        }
        Self { a, b }
    }

    fn near(&self, point: Vec2, tolerance: f32) -> bool {
        // Calculate the cross product to ensure the point is on the infinite line
        let cross_product = (point.y - self.a.y) * (self.b.x - self.a.x)
            - (point.x - self.a.x) * (self.b.y - self.a.y);
        if cross_product.abs() > tolerance {
            return false; // Not collinear
        }

        // Check if the point lies within the bounds of the segment
        let dot_product = (point.x - self.a.x) * (self.b.x - self.a.x)
            + (point.y - self.a.y) * (self.b.y - self.a.y);
        if dot_product < tolerance {
            return false; // Point is before `a`
        }

        let squared_length = (self.b.x - self.a.x).powi(2) + (self.b.y - self.a.y).powi(2);
        if dot_product > squared_length {
            return false; // Point is after `b`
        }

        true
    }
}

#[derive(Debug)]
struct Asteroid {
    pos: Vec2,
    sides: u8,
    radius: f32,
    rotation: f32,
}

impl Asteroid {
    fn random_asteroid() -> Self {
        let mut rng = thread_rng();

        Self {
            pos: Vec2::new(
                rng.gen_range(0.0..=screen_width()),
                rng.gen_range(0.0..=screen_height()),
            ),
            sides: rng.gen_range(3..8),
            radius: rng.gen_range(5.0..40.0),
            rotation: rng.gen_range(0.0..360.0),
        }
    }

    fn vertices(&self) -> Vec<Vec2> {
        let mut vertices = Vec::new();
        for i in 0..self.sides {
            let angle = self.rotation + i as f32 * (2.0 * std::f32::consts::PI / self.sides as f32);
            let x = self.pos.x + self.radius * angle.cos();
            let y = self.pos.y + self.radius * angle.sin();
            vertices.push(Vec2::new(x, y));
        }
        vertices
    }

    fn edges(&self) -> Vec<Line> {
        let vertices = self.vertices();
        let mut edges = Vec::new();

        for i in 0..vertices.len() {
            let start = vertices[i];
            let end = vertices[(i + 1) % vertices.len()]; // Wrap around to the first vertex
            edges.push(Line::new(start, end));
        }

        edges
    }
}

fn polar2euclidean(center: Vec2, radius: f32, angle: f32) -> Vec2 {
    Vec2::new(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    )
}

fn pixels_in_circle(
    center: Vec2,
    radius: f32,
    excluded_angles: &HashMap<usize, usize>,
) -> Vec<(Vec2, usize)> {
    let mut pixels = Vec::new();
    for angle in 0..=360 * ANGLE_PRECISION {
        if excluded_angles.contains_key(&angle) {
            continue;
        }
        pixels.push((
            polar2euclidean(center, radius, angle as f32 / ANGLE_PRECISION as f32),
            angle,
        ));
    }
    pixels
}

fn draw_circle_except_angles(
    center: Vec2,
    radius: f32,
    thickness: f32,
    color: Color,
    excluded_angles: &HashMap<usize, usize>, // (angle in degrees, radius in units)
) {
    // Draw the base circle first
    draw_circle_lines(center.x, center.y, radius, thickness, color);

    // Iterate over excluded angles and overlay "blackout" regions
    for (&rotation, &angle_radius) in excluded_angles.iter() {
        // Convert the angle and radius to radians and restrict the range
        let arc = rotation as f32 / ANGLE_PRECISION as f32 + (radius / angle_radius as f32);

        draw_arc(
            center.x,
            center.y,
            0,
            radius,
            rotation as f32,
            1.0,
            arc * ARC_GROWTH_RATE,
            BLACK,
        );
    }
}

async fn circle_render(edges: &Vec<Line>, center: Vec2) {
    let mut excluded_angles: HashMap<usize, usize> = HashMap::new(); // (angle, radius)
    let mut drawn_pixels: Vec<Vec2> = Vec::new();

    for scan_radius in (SHIP_SIZE as usize..screen_width() as usize).step_by(SCAN_SPEED) {
        draw_circle_except_angles(center, scan_radius as f32, 0.5, GREEN, &excluded_angles);
        // TODO: glitter background
        for (pixel, angle) in pixels_in_circle(center, scan_radius as f32, &excluded_angles) {
            for edge in edges {
                if edge.near(pixel, CLOSENESS_TOLERANCE) {
                    // draw pixel
                    drawn_pixels.push(pixel);
                    excluded_angles.insert(angle, scan_radius);
                    break;
                }
            }
        }
        for pixel in &drawn_pixels {
            draw_rectangle(pixel.x, pixel.y, 1.0, 1.0, RED);
        }
        draw_circle(center.x, center.y, SHIP_SIZE, WHITE);
        next_frame().await;
    }
}

#[macroquad::main("Wavey")]
async fn main() {
    let mut ship = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
    let mut asteroids: Vec<Asteroid> = Vec::new();
    for _i in 0..NUM_ASTEROIDS {
        let asteroid = Asteroid::random_asteroid();
        asteroids.push(asteroid);
    }
    let mut edges = Vec::new();
    for asteroid in &asteroids {
        edges.extend(asteroid.edges());
    }

    loop {
        draw_circle(ship.x, ship.y, SHIP_SIZE, WHITE);
        // if J pressed
        if is_key_pressed(KeyCode::J) {
            circle_render(&edges, ship).await;
        }
        if is_key_down(KeyCode::W) && ship.y >= 0.0 + SHIP_SIZE {
            // up
            ship.y -= 1.0;
        }
        if is_key_down(KeyCode::S) && ship.y <= screen_height() - SHIP_SIZE {
            // down
            ship.y += 1.0;
        }
        if is_key_down(KeyCode::A) && ship.x >= 0.0 + SHIP_SIZE {
            // left
            ship.x -= 1.0;
        }
        if is_key_down(KeyCode::D) && ship.x <= screen_width() - SHIP_SIZE {
            // right
            ship.x += 1.0;
        }
        next_frame().await;
    }
}
