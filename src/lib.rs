// Coords HUD: XYZ + seta do norte + direção, acima da barra de vida.
// O texto usa o text-shape do jogo, então já sai na fonte do Minecraft.

use horizon_mod_sdk::{
    game, render, Registration,
    render::{Point, QuadShape, Shape, TextShape},
};

const LARANJA: u32 = 0xFFFF_AA00;
const BRANCO: u32 = 0xFFFF_FFFF;

/// Quanto cada caractere avança na fonte padrão do Minecraft (em px da GUI).
fn largura_char(c: char) -> f32 {
    match c {
        ' ' => 4.0,
        ':' => 2.0,
        _ => 6.0, // dígitos, '-', X, Y, Z, N, E, S, W
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
fn seta(cx: f32, cy: f32, yaw: f32) -> Shape {
    // Ângulo a partir de "para cima", no sentido horário.
    let a = (180.0 - yaw).to_radians();
    let (s, c) = a.sin_cos();
    let girar = |px: f32, py: f32| Point {
        x: cx + px * c - py * s,
        y: cy + px * s + py * c,
    };
    Shape::Quad(QuadShape {
        corners: (girar(0.0, -4.0), girar(3.0, 3.0), girar(0.0, 1.0), girar(-3.0, 3.0)),
        color: BRANCO,
    })
}

struct Mod;

impl horizon_mod_sdk::Mod for Mod {
    fn init() -> Result<(Self, Registration), String> {
        Ok((Self, Registration::new().on_frame()))
    }

    fn frame(&mut self, _segundos: f32) {
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

        // rótulo, espaço, coords, vão, seta (8 px), vão, direção
        let total = largura(rotulo) + 4.0 + largura(&coords) + 7.0 + 8.0 + 7.0 + largura(dir);
        let mut x = ((w - total) / 2.0).round();
        let y = h - 95.0; // centralizado, mas acima da dica de bloco

        let mut formas = Vec::with_capacity(4);
        formas.push(texto(rotulo, x, y, LARANJA));
        x += largura(rotulo) + 4.0;
        formas.push(texto(&coords, x, y, BRANCO));
        x += largura(&coords) + 7.0;
        formas.push(seta(x + 4.0, y + 4.0, p.yaw));
        x += 8.0 + 7.0;
        formas.push(texto(dir, x, y, BRANCO));

        render::set_hud(&formas);
    }
}

horizon_mod_sdk::export_mod!(Mod);
