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
}

pub static ASSETS: &'static [Asset] = &[
    Asset {
        model: "trees",
        meshes: &[
            Mesh {
                variants: &["tree-1", "tree-2", "tree-3", "tree-4", "tree-5", "tree-6"],
                density: 32.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
            },
            Mesh {
                variants: &["tree-large-1", "tree-large-2"],
                density: 8.0,
                size_range: [1.5, 4.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
            },
            Mesh {
                variants: &["tree-small-1"],
                density: 16.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
            },
            Mesh {
                variants: &["pine-1", "pine-2", "pine-3"],
                density: 24.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.4],
                temp_range: [-20.0, 3.0],
                temp_preferred: -5.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.4,
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
            },
            Mesh {
                variants: &["tree-dead-1", "tree-dead-2", "tree-dead-3"],
                density: 4.0,
                size_range: [0.8, 2.5],
                slope_range: [0.0, 0.3],
                temp_range: [-5.0, 20.0],
                temp_preferred: 10.0,
                moist_range: [0.0, 0.3],
                moist_preferred: 0.0,
            },
        ],
    },
    Asset {
        model: "mushrooms",
        meshes: &[
            Mesh {
                variants: &["mushroom-red-1", "mushroom-red-2", "mushroom-red-3"],
                density: 50.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
            },
            Mesh {
                variants: &["mushroom-brown-1", "mushroom-brown-2", "mushroom-brown-3"],
                density: 150.0,
                size_range: [1.5, 2.5],
                slope_range: [0.0, 0.2],
                temp_range: [10.0, 25.0],
                temp_preferred: 17.0,
                moist_range: [0.3, 1.0],
                moist_preferred: 0.5,
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
                density: 1.0,
                size_range: [0.5, 1.5],
                slope_range: [0.0, 0.5],
                temp_range: [-20.0, 50.0],
                temp_preferred: -5.0,
                moist_range: [-10.0, 10.0],
                moist_preferred: 0.5,
            },
            Mesh {
                variants: &["stone-medium-1", "stone-medium-2", "stone-medium-3"],
                density: 0.5,
                size_range: [0.5, 1.5],
                slope_range: [0.0, 0.5],
                temp_range: [-20.0, 50.0],
                temp_preferred: -5.0,
                moist_range: [-10.0, 10.0],
                moist_preferred: 0.5,
            },
            Mesh {
                variants: &["stone-big-1", "stone-big-2", "stone-big-3"],
                density: 0.3,
                size_range: [0.5, 1.5],
                slope_range: [0.0, 0.1],
                temp_range: [-20.0, 50.0],
                temp_preferred: -5.0,
                moist_range: [-10.0, 10.0],
                moist_preferred: 0.5,
            },
        ],
    },
];
