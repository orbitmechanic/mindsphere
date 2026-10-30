use std::collections::HashMap;

use bevy::prelude::*;

use crate::util::color_from_hex;

pub struct SkinDef {
    pub id: &'static str,
    pub label: &'static str,
    pub swatch: &'static str,
    pub path: &'static str,
}

pub const SKINS: &[SkinDef] = &[
    SkinDef {
        id: "earth",
        label: "Earth",
        swatch: "#6f927c",
        path: "textures/earth_muted.jpg",
    },
    SkinDef {
        id: "mercury",
        label: "Mercury",
        swatch: "#8d827a",
        path: "textures/mercury.jpg",
    },
    SkinDef {
        id: "venus",
        label: "Venus",
        swatch: "#c8ad7a",
        path: "textures/venus_surface.jpg",
    },
    SkinDef {
        id: "mars",
        label: "Mars",
        swatch: "#a85f42",
        path: "textures/mars.jpg",
    },
    SkinDef {
        id: "jupiter",
        label: "Jupiter",
        swatch: "#b79872",
        path: "textures/jupiter.jpg",
    },
    SkinDef {
        id: "saturn",
        label: "Saturn",
        swatch: "#c3af7e",
        path: "textures/saturn.jpg",
    },
    SkinDef {
        id: "uranus",
        label: "Uranus",
        swatch: "#8fc8d0",
        path: "textures/uranus.jpg",
    },
    SkinDef {
        id: "neptune",
        label: "Neptune",
        swatch: "#4a6ca8",
        path: "textures/neptune.jpg",
    },
    SkinDef {
        id: "pluto",
        label: "Pluto",
        swatch: "#b09a7e",
        path: "textures/pluto.png",
    },
    SkinDef {
        id: "moon",
        label: "Moon",
        swatch: "#a7a7a2",
        path: "textures/moon.jpg",
    },
    SkinDef {
        id: "exo_lava",
        label: "Exo I · Lava",
        swatch: "#d04a20",
        path: "textures/exo_lava.png",
    },
    SkinDef {
        id: "exo_ocean",
        label: "Exo II · Ocean",
        swatch: "#2f8b78",
        path: "textures/exo_ocean.png",
    },
    SkinDef {
        id: "exo_ice",
        label: "Exo III · Ice",
        swatch: "#b7d6e8",
        path: "textures/exo_ice.png",
    },
    SkinDef {
        id: "geodesic",
        label: "Geodesic",
        swatch: "#2d9aad",
        path: "textures/geodesic.png",
    },
    SkinDef {
        id: "solid_slate",
        label: "Slate",
        swatch: "#283447",
        path: "textures/solid_slate.png",
    },
    SkinDef {
        id: "solid_teal",
        label: "Teal",
        swatch: "#165c61",
        path: "textures/solid_teal.png",
    },
    SkinDef {
        id: "solid_violet",
        label: "Violet",
        swatch: "#40305f",
        path: "textures/solid_violet.png",
    },
    SkinDef {
        id: "solid_sand",
        label: "Sand",
        swatch: "#7d6847",
        path: "textures/solid_sand.png",
    },
    SkinDef {
        id: "solid_obsidian",
        label: "Obsidian",
        swatch: "#0a0d14",
        path: "textures/solid_obsidian.png",
    },
    SkinDef {
        id: "gradient_aurora",
        label: "Aurora",
        swatch: "#2ea88e",
        path: "textures/gradient_aurora.png",
    },
    SkinDef {
        id: "gradient_ember",
        label: "Ember",
        swatch: "#c94b38",
        path: "textures/gradient_ember.png",
    },
    SkinDef {
        id: "gradient_ocean",
        label: "Ocean",
        swatch: "#2f7eaf",
        path: "textures/gradient_ocean.png",
    },
    SkinDef {
        id: "gradient_dusk",
        label: "Dusk",
        swatch: "#8e557f",
        path: "textures/gradient_dusk.png",
    },
];

pub const BACKGROUNDS: &[(&str, &str)] = &[("stars", "Generated stars"), ("skymap", "Star skymap")];

#[derive(Resource)]
pub struct SceneTextures {
    pub skins: HashMap<String, Handle<Image>>,
    pub stars: Handle<Image>,
    pub skymap: Handle<Image>,
}

impl SceneTextures {
    pub fn load(asset_server: &AssetServer) -> Self {
        let mut skins = HashMap::new();
        for skin in SKINS {
            skins.insert(skin.id.to_owned(), asset_server.load(skin.path));
        }
        Self {
            skins,
            stars: asset_server.load("textures/deep_space.png"),
            skymap: asset_server.load("textures/nebula.png"),
        }
    }

    pub fn skin(&self, id: &str) -> Handle<Image> {
        self.skins
            .get(id)
            .or_else(|| self.skins.get("earth"))
            .cloned()
            .unwrap_or_default()
    }

    pub fn background(&self, id: &str) -> Handle<Image> {
        if id == "skymap" {
            self.skymap.clone()
        } else {
            self.stars.clone()
        }
    }
}

pub fn skin_label(id: &str) -> &'static str {
    SKINS
        .iter()
        .find(|skin| skin.id == id)
        .map(|skin| skin.label)
        .unwrap_or("Earth")
}

pub fn skin_swatch(id: &str) -> Color {
    SKINS
        .iter()
        .find(|skin| skin.id == id)
        .map(|skin| color_from_hex(skin.swatch))
        .unwrap_or(Color::srgb(0.3, 0.5, 0.5))
}

pub fn background_label(id: &str) -> &'static str {
    BACKGROUNDS
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, label)| *label)
        .unwrap_or("Generated stars")
}
