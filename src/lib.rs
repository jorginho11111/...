// Coords HUD: XYZ + seta do norte + direção, centralizado acima da dica de bloco.
// O texto usa o text-shape do jogo, então já sai na fonte do Minecraft.

use horizon_mod_sdk::{
    game, render, storage, Mod, Registration,
    render::{Point, QuadShape, Shape, TextShape},
    ui::{self, ButtonWidget, TextWidget, Widget},
};

const CORES: [(u32, u32, &str); 5] = [
    (0xFFFF_AA00, 0xFFFF_FFFF, "Orange and white"),
    (0xFF55_FFFF, 0xFFFF_FFFF, "Cyan and white"),
    (0xFF55_FF55, 0xFFFF_FFFF, "Green and white"),
    (0xFFFF_55FF, 0xFFFF_FFFF, "Pink and white"),
    (0xFFFF_FFFF, 0xFFFF_FFFF, "White"),
];
const ESCALAS: [f32; 4] = [1.0, 1.25, 1.5, 1.75];
const PASSO: f32 = 8.0;

const EDITAR: u32 = 1;
const ESQUERDA: u32 = 2;
const DIREITA: u32 = 3;
const CIMA: u32 = 4;
const BAIXO: u32 = 5;
const TAMANHO: u32 = 6;
const COR: u32 = 7;
const CONCLUIR: u32 = 8;
const TOGGLE_COORDS: u32 = 9;
const TOGGLE_DIRECTION: u32 = 10;
const TOGGLE_DAYS: u32 = 11;

/// Quanto cada caractere avança na fonte padrão do Minecraft (em px da GUI).
fn largura_char(c: char) -> f32 {
    match c {
        ' ' => 4.0,
        ':' => 2.0,
        _ => 6.0,
    }
}

fn largura(s: &str) -> f32 {
    s.chars().map(largura_char).sum()
}

/// Yaw do Minecraft: 0 = sul, 90 = oeste, 180 = norte, 270 = leste.
fn direcao(yaw: f32) -> &'static str {
    const NOMES: [&str; 8] = ["S", "SW", "W", "NW", "N", "NE", "E", "SE"];
    let y = yaw.rem_euclid(360.0);
    NOMES[(((y + 22.5) / 45.0) as usize) % 8]
}

fn texto(s: &str, x: f32, y: f32, cor: u32) -> Shape {
    Shape::Text(TextShape {
        text: s.to_string(),
        at: Point { x, y },
        color: cor,
        centered: false,
        shadow: true,
    })
}

/// Seta que aponta para o norte (como uma agulha de bússola).
fn seta(cx: f32, cy: f32, yaw: f32, escala: f32, cor: u32) -> Shape {
    let a = (180.0 - yaw).to_radians();
    let (s, c) = a.sin_cos();
    let girar = |px: f32, py: f32| Point {
        x: cx + (px * c - py * s) * escala,
        y: cy + (px * s + py * c) * escala,
    };
    Shape::Quad(QuadShape {
        corners: (girar(0.0, -4.0), girar(3.0, 3.0), girar(0.0, 1.0), girar(-3.0, 3.0)),
        color: cor,
    })
}

struct CoordsHud {
    offset_x: f32,
    offset_y: f32,
    escala: usize,
    cor: usize,
    show_coordinates: bool,
    show_direction: bool,
    show_days: bool,
    editando: bool,
}

impl CoordsHud {
    fn settings(&self) {
        let button = |id, label: String| Widget::Button(ButtonWidget { id, label });
        if !self.editando {
            ui::set_settings(&[
                button(EDITAR, "Edit HUD".to_owned()),
                button(
                    TOGGLE_COORDS,
                    format!("Coordinates: {}", if self.show_coordinates { "ON" } else { "OFF" }),
                ),
                button(
                    TOGGLE_DIRECTION,
                    format!("Direction guide: {}", if self.show_direction { "ON" } else { "OFF" }),
                ),
                button(
                    TOGGLE_DAYS,
                    format!("Day counter: {}", if self.show_days { "ON" } else { "OFF" }),
                ),
            ]);
            return;
        }
        ui::set_settings(&[
            Widget::Text(TextWidget {
                id: 100,
                text: "Move the HUD in 8 px steps".to_owned(),
                color: 0,
            }),
            Widget::Text(TextWidget {
                id: 101,
                text: "Size adjusts the layout and arrow; the game font has a fixed size.".to_owned(),
                color: 0,
            }),
            button(ESQUERDA, "Move left".to_owned()),
            button(DIREITA, "Move right".to_owned()),
            button(CIMA, "Move up".to_owned()),
            button(BAIXO, "Move down".to_owned()),
            button(
                TAMANHO,
                format!("Size/layout: {}%", (ESCALAS[self.escala] * 100.0) as u32),
            ),
            button(COR, format!("Color: {}", CORES[self.cor].2)),
            button(CONCLUIR, "Done editing".to_owned()),
        ]);
    }

    fn save(&self) {
        let _ = storage::set("offset_x", self.offset_x.round().to_string().as_bytes());
        let _ = storage::set("offset_y", self.offset_y.round().to_string().as_bytes());
        let _ = storage::set("size", self.escala.to_string().as_bytes());
        let _ = storage::set("color", self.cor.to_string().as_bytes());
        let _ = storage::set(
            "show_coordinates",
            if self.show_coordinates { b"true" } else { b"false" },
        );
        let _ = storage::set(
            "show_direction",
            if self.show_direction { b"true" } else { b"false" },
        );
        let _ = storage::set("show_days", if self.show_days { b"true" } else { b"false" });
    }

    fn draw(&self) {
        let Some(player) = game::get_player() else {
            render::set_hud(&[]);
            return;
        };
        let (w, h) = render::gui_size();
        let label = "XYZ:";
        let coordinates = format!(
            "{} {} {}",
            player.x.floor() as i64,
            player.y.floor() as i64,
            player.z.floor() as i64
        );
        let direction = direcao(player.yaw);
        let day = if self.show_days {
            game::get_clock().map(|clock| format!("Day {}", clock.day_time / 24_000 + 1))
        } else {
            None
        };
        let scale = ESCALAS[self.escala];
        let (label_color, text_color, _) = CORES[self.cor];

        let coordinate_width = if self.show_coordinates {
            largura(label) + 4.0 + largura(&coordinates)
        } else {
            0.0
        };
        let direction_width = if self.show_direction {
            15.0 + largura(direction)
        } else {
            0.0
        };
        let day_width = day.as_deref().map_or(0.0, largura);
        let visible_groups: usize = (if self.show_coordinates { 1 } else { 0 })
            + (if self.show_direction { 1 } else { 0 })
            + (if day.is_some() { 1 } else { 0 });
        let total = (coordinate_width
            + direction_width
            + day_width
            + 7.0 * visible_groups.saturating_sub(1) as f32)
            * scale;
        let mut x = ((w - total) / 2.0 + self.offset_x).clamp(0.0, (w - total).max(0.0));
        let y = (h - 95.0 + self.offset_y).clamp(8.0, (h - 12.0).max(8.0));
        let mut forms = Vec::with_capacity(5);
        let mut has_group = false;

        if self.show_coordinates {
            forms.push(texto(label, x, y, label_color));
            x += (largura(label) + 4.0) * scale;
            forms.push(texto(&coordinates, x, y, text_color));
            x += largura(&coordinates) * scale;
            has_group = true;
        }
        if self.show_direction {
            if has_group {
                x += 7.0 * scale;
            }
            forms.push(seta(x + 4.0 * scale, y + 4.0 * scale, player.yaw, scale, text_color));
            x += 15.0 * scale;
            forms.push(texto(direction, x, y, text_color));
            x += largura(direction) * scale;
            has_group = true;
        }
        if let Some(day) = day {
            if has_group {
                x += 7.0 * scale;
            }
            forms.push(texto(&day, x, y, text_color));
        }
        render::set_hud(&forms);
    }
}

fn stored_i32(key: &str, default: i32) -> i32 {
    storage::get(key)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn stored_bool(key: &str, default: bool) -> bool {
    storage::get(key)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

impl Mod for CoordsHud {
    fn init() -> Result<(Self, Registration), String> {
        let scale = stored_i32("size", 0).clamp(0, ESCALAS.len() as i32 - 1) as usize;
        let color = stored_i32("color", 0).clamp(0, CORES.len() as i32 - 1) as usize;
        let mod_state = Self {
            offset_x: stored_i32("offset_x", 0) as f32,
            offset_y: stored_i32("offset_y", 0) as f32,
            escala: scale,
            cor: color,
            show_coordinates: stored_bool("show_coordinates", true),
            show_direction: stored_bool("show_direction", true),
            show_days: stored_bool("show_days", true),
            editando: false,
        };
        mod_state.settings();
        Ok((mod_state, Registration::new().on_frame()))
    }

    fn frame(&mut self, _seconds: f32) {
        self.draw();
    }

    fn ui_select(&mut self, id: u32, _row: u32) {
        match id {
            EDITAR if !self.editando => self.editando = true,
            CONCLUIR if self.editando => self.editando = false,
            ESQUERDA if self.editando => self.offset_x -= PASSO,
            DIREITA if self.editando => self.offset_x += PASSO,
            CIMA if self.editando => self.offset_y -= PASSO,
            BAIXO if self.editando => self.offset_y += PASSO,
            TAMANHO if self.editando => self.escala = (self.escala + 1) % ESCALAS.len(),
            COR if self.editando => self.cor = (self.cor + 1) % CORES.len(),
            TOGGLE_COORDS if !self.editando => self.show_coordinates = !self.show_coordinates,
            TOGGLE_DIRECTION if !self.editando => self.show_direction = !self.show_direction,
            TOGGLE_DAYS if !self.editando => self.show_days = !self.show_days,
            _ => return,
        }
        self.save();
        self.settings();
        self.draw();
    }

    fn ui_settings_opened(&mut self) {
        self.editando = false;
        self.settings();
    }
}

horizon_mod_sdk::export_mod!(CoordsHud);
