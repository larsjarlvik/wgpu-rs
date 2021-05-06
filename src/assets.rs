pub struct Asset {
    pub model: &'static str,
    pub meshes: &'static [Mesh],
}

pub struct Mesh {
    pub variants: &'static [&'static str],
    pub density: f32,
    pub size_range: [f32; 2],
    pub slope_range: [f32; 2],
    pub temp_range: [f32; 2],
    pub temp_preferred: f32,
    pub moist_range: [f32; 2],
    pub moist_preferred: f32,
    pub rotation: [(f32, f32); 3],
    pub align: bool,
    pub radius: f32,
}

pub static ASSETS: &'static [Asset] = &[
    Asset {
        model: "trees",
        meshes: &[
            // Snow + Tundra
            Mesh {
                variants: &["pine-1", "pine-2", "pine-3"],
                density: 24.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.4],
                temp_range: [-20.0, 3.0],
                temp_preferred: -5.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.4,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.5,
            },
            Mesh {
                variants: &["spruce-1"],
                density: 24.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.3],
                temp_range: [-20.0, 5.0],
                temp_preferred: -5.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.4,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 2.2,
            },
            // Barren
            Mesh {
                variants: &["tree-dead-1", "tree-dead-2", "tree-dead-3"],
                density: 4.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.3],
                temp_range: [-5.0, 20.0],
                temp_preferred: 10.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.0,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.5,
            },
            Mesh {
                variants: &["log-1", "log-2", "log-3"],
                density: 4.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.3],
                temp_range: [-5.0, 20.0],
                temp_preferred: 10.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.0,
                rotation: [(-0.0, 0.0), (-180.0, 180.0), (-0.0, 0.0)],
                align: true,
                radius: 2.0,
            },
            Mesh {
                variants: &["stump-1"],
                density: 4.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.3],
                temp_range: [-5.0, 20.0],
                temp_preferred: 10.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.0,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: true,
                radius: 0.7,
            },
            // Grassland
            Mesh {
                variants: &["tree-small-1", "tree-1", "tree-2", "tree-3", "tree-4", "tree-5", "tree-6"],
                density: 20.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.7,
            },
            Mesh {
                variants: &["tree-large-1", "tree-large-2"],
                density: 5.0,
                size_range: [1.5, 4.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 1.0,
            },
            Mesh {
                variants: &["bush-1", "bush-2", "bush-3", "bush-4"],
                density: 32.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: true,
                radius: 1.0,
            },
            Mesh {
                variants: &["flower-1", "flower-2", "flower-3"],
                density: 32.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: true,
                radius: 0.0,
            },
            Mesh {
                variants: &["log-1", "log-2", "log-3"],
                density: 10.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.4],
                temp_range: [30.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.7,
                rotation: [(-45.0, 45.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: true,
                radius: 1.5,
            },
            // Tropical
            Mesh {
                variants: &["tree-small-1", "tree-1", "tree-2", "tree-3", "tree-4", "tree-5", "tree-6"],
                density: 62.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [30.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.7,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.7,
            },
            Mesh {
                variants: &["tree-large-1", "tree-large-2"],
                density: 20.0,
                size_range: [1.5, 4.5],
                slope_range: [0.0, 0.2],
                temp_range: [30.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.7,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 1.0,
            },
            Mesh {
                variants: &["fern-1", "fern-2", "fern-3", "fern-4", "fern-5"],
                density: 64.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [30.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.7,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.0,
            },
        ],
    },
    Asset {
        model: "mushrooms",
        meshes: &[
            // Grassland + Tropical
            Mesh {
                variants: &["mushroom-red-1", "mushroom-red-2", "mushroom-red-3"],
                density: 50.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 50.0],
                temp_preferred: 20.0,
                moist_range: [0.5, 1.0],
                moist_preferred: 0.8,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.0,
            },
            Mesh {
                variants: &["mushroom-brown-1", "mushroom-brown-2", "mushroom-brown-3"],
                density: 150.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 50.0],
                temp_preferred: 20.0,
                moist_range: [0.5, 1.0],
                moist_preferred: 0.8,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 0.0,
            },
        ],
    },
    Asset {
        model: "stones",
        meshes: &[
            Mesh {
                variants: &[
                    "stone-small-1",
                    "stone-small-2",
                    "stone-small-3",
                    "stone-small-4",
                    "stone-small-5",
                    "stone-small-6",
                    "stone-small-7",
                ],
                density: 10.0,
                size_range: [0.2, 0.7],
                slope_range: [0.0, 0.5],
                temp_range: [-20.0, 50.0],
                temp_preferred: -5.0,
                moist_range: [0.15, 1.0],
                moist_preferred: 0.65,
                rotation: [(-180.0, 180.0), (-180.0, 180.0), (-180.0, 180.0)],
                align: false,
                radius: 0.7,
            },
            Mesh {
                variants: &["stone-medium-1", "stone-medium-2", "stone-medium-3"],
                density: 0.5,
                size_range: [0.5, 1.5],
                slope_range: [0.0, 0.3],
                temp_range: [-20.0, 50.0],
                temp_preferred: -5.0,
                moist_range: [0.15, 1.0],
                moist_preferred: 0.65,
                rotation: [(-180.0, 180.0), (-180.0, 180.0), (-180.0, 180.0)],
                align: true,
                radius: 1.6,
            },
            Mesh {
                variants: &["stone-big-1", "stone-big-2", "stone-big-3"],
                density: 0.3,
                size_range: [0.5, 1.5],
                slope_range: [0.0, 0.1],
                temp_range: [-20.0, 50.0],
                temp_preferred: -5.0,
                moist_range: [0.15, 1.0],
                moist_preferred: 0.65,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: true,
                radius: 7.5,
            },
        ],
    },
    Asset {
        model: "desert-trees",
        meshes: &[
            Mesh {
                variants: &["tree-dead-1", "tree-dead-2", "bush-dead-1"],
                density: 5.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 20.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.25,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 1.5,
            },
            Mesh {
                variants: &["fern-1", "fern-2", "fern-3"],
                density: 10.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 20.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.25,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.7,
            },
            Mesh {
                variants: &["cactus-1", "cactus-2", "cactus-3", "cactus-4", "cactus-5"],
                density: 10.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.0,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.5,
            },
            Mesh {
                variants: &["tree-joshua-1"],
                density: 3.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.15,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.5,
            },
            // Tropical
            Mesh {
                variants: &["tree-palm-1", "tree-palm-2", "tree-palm-3", "tree-palm-4"],
                density: 20.0,
                size_range: [2.0, 3.5],
                slope_range: [0.0, 0.2],
                temp_range: [30.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.7,
                rotation: [(-5.0, 5.0), (-180.0, 180.0), (-5.0, 5.0)],
                align: false,
                radius: 1.0,
            },
        ],
    },
    Asset {
        model: "desert-rocks",
        meshes: &[
            Mesh {
                variants: &["rock-small-1", "rock-small-2", "rock-small-3"],
                density: 14.0,
                size_range: [1.0, 1.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.0, 0.15],
                moist_preferred: 0.0,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.5,
            },
            Mesh {
                variants: &["rock-medium-1", "rock-medium-2"],
                density: 8.0,
                size_range: [1.0, 1.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.0, 0.15],
                moist_preferred: 0.0,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.5,
            },
            Mesh {
                variants: &["rock-half-1", "rock-half-2", "rock-half-3", "rock-half-4"],
                density: 6.0,
                size_range: [1.0, 1.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.0, 0.15],
                moist_preferred: 0.0,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.5,
            },
            Mesh {
                variants: &["rock-large-1", "rock-large-2", "rock-large-3", "rock-large-4"],
                density: 3.0,
                size_range: [1.0, 1.5],
                slope_range: [0.0, 0.5],
                temp_range: [10.0, 50.0],
                temp_preferred: 30.0,
                moist_range: [0.0, 0.15],
                moist_preferred: 0.0,
                rotation: [(-10.0, 10.0), (-180.0, 180.0), (-10.0, 10.0)],
                align: false,
                radius: 0.5,
            },
        ],
    },
];
