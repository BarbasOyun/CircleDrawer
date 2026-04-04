use glam::*;

pub fn circle_pos(radius: f32, progress: f32) -> Vec2 {
    // progress = (progress + 0.01) % (math.pi * 2)
    let x = progress.cos() * radius;
    let y = progress.sin() * radius;
    return Vec2 { x, y };
}

pub fn circle_points(radius: f32, segments: u16) -> Vec<Vec2> {
    let mut points: Vec<Vec2> = vec!{};
    let point_distance = (std::f32::consts::PI * 2.0) / segments as f32;

    for i in 0..segments {
        points.push(circle_pos(radius, point_distance * i as f32));
    }

    return points;
}

// TODO : Draw Circle using Linear Algebra
pub fn vector_basis_circle(radius: f32, segments: u16) -> Vec<Vec2> {
    let mut points: Vec<Vec2> = vec!{};

    let local_point = CVec2 { x: radius, y: 0.0 };
    let angle_step: f32 = (std::f32::consts::PI * 2.0) / segments as f32;
    
    for i in 0..segments {
        // Calculate the rotation of our basis for this segment
        let angle = i as f32 * angle_step;
        
        // Transform the point into the world basis
        let rotation = local_point.rotate(angle);
        points.insert(points.len(), rotation);
        
        // println!("Point {}: x: {:.2}, y: {:.2}", i, world_pos.x, world_pos.y);
    }

    return points;
}

// Custom Vector2
struct CVec2 {
    x: f32,
    y: f32,
}

impl CVec2 {
    fn rotate(&self, rad: f32) -> Vec2 {
        let rotated_x = self.x * rad.cos() - self.y * rad.sin();
        let rotated_y = self.x * rad.sin() + self.y * rad.cos();
        return Vec2 {
            x: rotated_x,
            y: rotated_y,
        };
    }

    // Test
    fn update_rotation(&mut self, radius: f32, angle: f32) {
        self.x = radius * angle.cos();
        self.y = radius * angle.sin();
    }

    fn set_origin(&self, origin: &Vec2) -> Vec2 {
        return Vec2 {
            x: origin.x + self.x,
            y: origin.y + self.y,
        };
    }

    fn to_world_basis(&self, origin: &CVec2, rotation_rad: f32) -> Vec2 {
        // 1. Rotate the point relative to its own (0,0)
        let rotated_x = self.x * rotation_rad.cos() - self.y * rotation_rad.sin();
        let rotated_y = self.x * rotation_rad.sin() + self.y * rotation_rad.cos();

        // 2. Translate it to the "Origin" of the new basis
        Vec2 {
            x: origin.x + rotated_x,
            y: origin.y + rotated_y,
        }
    }
}