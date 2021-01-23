pub struct Asset {
    pub name: &'static str,
    pub density: f32,
    pub min_size: f32,
    pub max_size: f32,
}

pub static ASSETS: &'static [Asset] = &[
    Asset {
        name: "pine-1",
        density: 0.001875,
        min_size: 1.5,
        max_size: 2.5,
    },
    Asset {
        name: "pine-2",
        density: 0.001875,
        min_size: 1.5,
        max_size: 2.5,
    },
    Asset {
        name: "pine-3",
        density: 0.001875,
        min_size: 1.5,
        max_size: 2.5,
    },
    Asset {
        name: "rock-1",
        density: 0.001875,
        min_size: 1.5,
        max_size: 4.5,
    },
];
