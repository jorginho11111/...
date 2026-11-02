// Coords HUD: XYZ + seta do norte + direção, centralizado acima da dica de bloco.
// O texto usa o text-shape do jogo, então já sai na fonte do Minecraft.

use horizon_mod_sdk::{
    game, render, storage, Mod, Registration,
    render::{Point, QuadShape, Shape, TextShape},
    ui::{self, ButtonWidget, TextWidget, Widget},
};

const CORES: [(u32, u32, &str); 5] = [
    (0xFFFF_AA00, 0xFFFF_FFFF, "Laranja e branco"),
    (0xFF55_FFFF, 0xFFFF_FFFF, "Ciano e branco"),
    (0xFF55_FF55, 0xFFFF_FFFF, "Verde e branco"),
    (0xFFFF_55FF, 0xFFFF_FFFF, "Rosa e branco"),
    (0xFFFF_FFFF, 0xFFFF_FFFF, "Branco"),
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
    editando: bool,
}

impl CoordsHud {
    fn settings(&self) {
        let button = |id, label: String| Widget::Button(ButtonWidget { id, label });
        if !self.editando {
            ui::set_settings(&[button(EDITAR, "Editar HUD".to_owned())]);
            return;
        }
        ui::set_settings(&[
            Widget::Text(TextWidget {
                id: 100,
                text: "Mover o HUD em passos de 8 px".to_owned(),
                color: 0,
            }),
            Widget::Text(TextWidget {
                id: 101,
                text: "O tamanho ajusta o layout e a seta; a fonte do jogo tem tamanho fixo.".to_owned(),
                color: 0,
            }),
            button(ESQUERDA, "Mover para a esquerda".to_owned()),
            button(DIREITA, "Mover para a direita".to_owned()),
            button(CIMA, "Mover para cima".to_owned()),
            button(BAIXO, "Mover para baixo".to_owned()),
            button(
                TAMANHO,
                format!("Tamanho/layout: {}%", (ESCALAS[self.escala] * 100.0) as u32),
            ),
            button(COR, format!("Cor: {}", CORES[self.cor].2)),
            button(CONCLUIR, "Concluir edição".to_owned()),
        ]);
    }

    fn save(&self) {
        let _ = storage::set("offset_x", self.offset_x.round().to_string().as_bytes());
        let _ = storage::set("offset_y", self.offset_y.round().to_string().as_bytes());
        let _ = storage::set("size", self.escala.to_string().as_bytes());
        let _ = storage::set("color", self.cor.to_string().as_bytes());
    }

    fn draw(&self) {
        let Some(p) = game::get_player() else {
            render::set_hud(&[]);
            return;
        };
        let (w, h) = render::gui_size();
        let rotulo = "XYZ:";
        let coords = format!(
            "{} {} {}",
            p.x.floor() as i64,
            p.y.floor() as i64,
            p.z.floor() as i64
        );
        let dir = direcao(p.yaw);
        let escala = ESCALAS[self.escala];
        let (cor_rotulo, cor_texto, cor_seta) = CORES[self.cor];

        let total = (largura(rotulo) + 4.0 + largura(&coords) + 7.0 + 8.0 + 7.0 + largura(dir)) * escala;
        let mut x = ((w - total) / 2.0 + self.offset_x).clamp(0.0, (w - total).max(0.0));
        let y = (h - 95.0 + self.offset_y).clamp(8.0, (h - 12.0).max(8.0));

        let mut formas = Vec::with_capacity(4);
        formas.push(texto(rotulo, x, y, cor_rotulo));
        x += (largura(rotulo) + 4.0) * escala;
        formas.push(texto(&coords, x, y, cor_texto));
        x += (largura(&coords) + 7.0) * escala;
        formas.push(seta(x + 4.0 * escala, y + 4.0 * escala, p.yaw, escala, cor_seta));
        x += 15.0 * escala;
        formas.push(texto(dir, x, y, cor_texto));
        render::set_hud(&formas);
    }
}

fn stored_i32(key: &str, default: i32) -> i32 {
    storage::get(key)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

impl Mod for CoordsHud {
    fn init() -> Result<(Self, Registration), String> {
        let escala = stored_i32("size", 0).clamp(0, ESCALAS.len() as i32 - 1) as usize;
        let cor = stored_i32("color", 0).clamp(0, CORES.len() as i32 - 1) as usize;
        let mod_state = Self {
            offset_x: stored_i32("offset_x", 0) as f32,
            offset_y: stored_i32("offset_y", 0) as f32,
            escala,
            cor,
            editando: false,
        };
        mod_state.settings();
        Ok((mod_state, Registration::new().on_frame()))
    }

    fn frame(&mut self, _segundos: f32) {
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
