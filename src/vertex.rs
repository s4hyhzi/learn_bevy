#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [i8; 4],
    pub normal: [i8; 4],
}

pub fn vertex(pos: [i8; 3], nor: [i8; 3]) -> Vertex {
    Vertex {
        position: [pos[0], pos[1], pos[2], 1],
        normal: [nor[0], nor[1], nor[2], 0],
    }
}

pub fn create_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        vertex([-1, -1, -1], [0, 0, -1]),
        vertex([1, -1, -1], [0, 0, -1]),
        vertex([1, 1, -1], [0, 0, -1]),
        vertex([-1, 1, -1], [0, 0, -1]),
        vertex([-1, -1, 1], [0, 0, 1]),
        vertex([1, -1, 1], [0, 0, 1]),
        vertex([1, 1, 1], [0, 0, 1]),
        vertex([-1, 1, 1], [0, 0, 1]),
        vertex([-1, -1, -1], [0, -1, 0]),
        vertex([1, -1, -1], [0, -1, 0]),
        vertex([1, -1, 1], [0, -1, 0]),
        vertex([-1, -1, 1], [0, -1, 0]),
        vertex([-1, 1, -1], [0, 1, 0]),
        vertex([1, 1, -1], [0, 1, 0]),
        vertex([1, 1, 1], [0, 1, 0]),
        vertex([-1, 1, 1], [0, 1, 0]),
        vertex([-1, -1, -1], [-1, 0, 0]),
        vertex([-1, 1, -1], [-1, 0, 0]),
        vertex([-1, 1, 1], [-1, 0, 0]),
        vertex([-1, -1, 1], [-1, 0, 0]),
        vertex([1, -1, -1], [1, 0, 0]),
        vertex([1, 1, -1], [1, 0, 0]),
        vertex([1, 1, 1], [1, 0, 0]),
        vertex([1, -1, 1], [1, 0, 0]),
    ];

    let index_data = [
        0, 1, 2, 2, 3, 0,
        4, 5, 6, 6, 7, 4,
        8, 9, 10, 10, 11, 8,
        12, 13, 14, 14, 15, 12,
        16, 17, 18, 18, 19, 16,
        20, 21, 22, 22, 23, 20,
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub fn create_plane() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        vertex([-1, 0, -1], [0, 1, 0]),
        vertex([1, 0, -1], [0, 1, 0]),
        vertex([1, 0, 1], [0, 1, 0]),
        vertex([-1, 0, 1], [0, 1, 0]),
    ];

    let index_data = [
        0, 1, 2, 2, 3, 0,
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub fn create_texels(size: usize) -> Vec<u8> {
    use std::iter;

    (0..size * size)
        .flat_map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            iter::once(0xFF - (count * 5) as u8)
                .chain(iter::once(0xFF - (count * 15) as u8))
                .chain(iter::once(0xFF - (count * 50) as u8))
                .chain(iter::once(1))
        })
        .collect()
}