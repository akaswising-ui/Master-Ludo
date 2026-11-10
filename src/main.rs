use macroquad::prelude::*;
use master_ludo::{Game, Phase, PlayerColor, Pos, FINISHED, SAFE_SQUARES, TRACK, YARD};

const BACKGROUND: Color = Color::new(0.055, 0.075, 0.12, 1.0);
const PANEL: Color = Color::new(0.10, 0.13, 0.19, 1.0);
const BOARD: Color = Color::new(0.98, 0.97, 0.92, 1.0);
const INK: Color = Color::new(0.12, 0.15, 0.20, 1.0);
const MUTED: Color = Color::new(0.66, 0.71, 0.78, 1.0);

#[derive(Clone, Copy)]
struct BoardLayout {
    left: f32,
    top: f32,
    cell: f32,
    side: f32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Master Ludo".to_owned(),
        window_width: 720,
        window_height: 900,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player_count = 4usize;
    let mut game: Option<Game> = None;
    let mut in_menu = true;

    loop {
        clear_background(BACKGROUND);
        let (mouse_x, mouse_y) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);

        if in_menu {
            draw_menu(player_count);
            if clicked {
                let width = screen_width();
                let button_width = (width - 52.0) / 3.0;
                let option_y = screen_height() * 0.49;
                for (index, count) in [2usize, 3, 4].into_iter().enumerate() {
                    let x = 18.0 + index as f32 * (button_width + 8.0);
                    if point_in_rect(mouse_x, mouse_y, x, option_y, button_width, 58.0) {
                        player_count = count;
                    }
                }
                let start_y = screen_height() * 0.65;
                if point_in_rect(mouse_x, mouse_y, width * 0.18, start_y, width * 0.64, 66.0) {
                    game = Some(Game::new(player_count));
                    in_menu = false;
                }
            }
        } else {
            let mut return_to_menu = false;
            let mut restart_game = false;
            if let Some(active_game) = game.as_mut() {
                let layout = board_layout();
                draw_game(active_game, layout);

                let panel_y = layout.top + layout.side + 16.0;
                let roll_rect = (screen_width() * 0.5 - 104.0, panel_y + 63.0, 208.0, 52.0);
                let menu_rect = (screen_width() - 104.0, 24.0, 88.0, 38.0);

                if clicked
                    && point_in_rect(
                        mouse_x,
                        mouse_y,
                        menu_rect.0,
                        menu_rect.1,
                        menu_rect.2,
                        menu_rect.3,
                    )
                {
                    return_to_menu = true;
                } else if clicked
                    && active_game.phase == Phase::AwaitRoll
                    && point_in_rect(
                        mouse_x,
                        mouse_y,
                        roll_rect.0,
                        roll_rect.1,
                        roll_rect.2,
                        roll_rect.3,
                    )
                {
                    let roll = macroquad::rand::gen_range(1u8, 7u8);
                    let _ = active_game.roll_dice(roll);
                    let legal = active_game.legal_moves();
                    if legal.len() == 1 {
                        let _ = active_game.move_token(legal[0]);
                    }
                } else if clicked && active_game.phase == Phase::AwaitToken {
                    if let Some(token_index) = tapped_token(active_game, layout, mouse_x, mouse_y) {
                        let _ = active_game.move_token(token_index);
                    }
                } else if clicked
                    && active_game.phase == Phase::GameOver
                    && point_in_rect(
                        mouse_x,
                        mouse_y,
                        roll_rect.0,
                        roll_rect.1,
                        roll_rect.2,
                        roll_rect.3,
                    )
                {
                    restart_game = true;
                }

                if !clicked
                    && is_key_pressed(KeyCode::Space)
                    && active_game.phase == Phase::AwaitRoll
                {
                    let roll = macroquad::rand::gen_range(1u8, 7u8);
                    let _ = active_game.roll_dice(roll);
                    let legal = active_game.legal_moves();
                    if legal.len() == 1 {
                        let _ = active_game.move_token(legal[0]);
                    }
                }
            }
            if return_to_menu {
                in_menu = true;
                game = None;
            } else if restart_game {
                game = Some(Game::new(player_count));
            }
        }

        next_frame().await;
    }
}

fn draw_menu(player_count: usize) {
    let width = screen_width();
    let height = screen_height();
    let title = "MASTER LUDO";
    draw_text(
        title,
        28.0,
        height * 0.25,
        48.0_f32.min(width * 0.105),
        WHITE,
    );
    draw_text(
        "A classic board game, built entirely in Rust",
        30.0,
        height * 0.25 + 36.0,
        17.0,
        MUTED,
    );

    draw_text("CHOOSE PLAYERS", 22.0, height * 0.42, 18.0, WHITE);
    let button_width = (width - 52.0) / 3.0;
    let button_y = height * 0.49;
    for (index, count) in [2usize, 3, 4].into_iter().enumerate() {
        let x = 18.0 + index as f32 * (button_width + 8.0);
        let selected = count == player_count;
        draw_button(
            x,
            button_y,
            button_width,
            58.0,
            &format!("{count} PLAYERS"),
            if selected {
                color_for(PlayerColor::Green)
            } else {
                PANEL
            },
            if selected { WHITE } else { MUTED },
        );
    }

    draw_button(
        width * 0.18,
        height * 0.65,
        width * 0.64,
        66.0,
        "START GAME",
        color_for(PlayerColor::Red),
        WHITE,
    );
    draw_text(
        "OFFLINE  •  PASS AND PLAY  •  2–4 PLAYERS",
        20.0,
        height * 0.84,
        14.0,
        MUTED,
    );
    draw_text(
        "Roll a six to leave the yard. Get all four tokens home.",
        20.0,
        height * 0.84 + 26.0,
        13.0,
        MUTED,
    );
}

fn board_layout() -> BoardLayout {
    let side = (screen_width() - 28.0)
        .min(screen_height() * 0.54)
        .min(760.0)
        .max(260.0);
    let side = side.min(screen_width() - 16.0);
    BoardLayout {
        left: (screen_width() - side) * 0.5,
        top: 112.0,
        cell: side / 15.0,
        side,
    }
}

fn draw_game(game: &Game, layout: BoardLayout) {
    draw_text("MASTER LUDO", 18.0, 40.0, 25.0, WHITE);
    draw_button(
        screen_width() - 104.0,
        18.0,
        88.0,
        38.0,
        "MENU",
        PANEL,
        WHITE,
    );

    draw_board_base(layout);
    draw_tokens(game, layout);
    draw_status(game, layout);
}

fn draw_board_base(layout: BoardLayout) {
    let l = layout.left;
    let t = layout.top;
    let c = layout.cell;

    draw_rectangle(
        l - 4.0,
        t - 4.0,
        layout.side + 8.0,
        layout.side + 8.0,
        BOARD,
    );
    draw_rectangle(
        l,
        t,
        c * 6.0,
        c * 6.0,
        tint(color_for(PlayerColor::Red), 0.20),
    );
    draw_rectangle(
        l + c * 9.0,
        t,
        c * 6.0,
        c * 6.0,
        tint(color_for(PlayerColor::Green), 0.20),
    );
    draw_rectangle(
        l + c * 9.0,
        t + c * 9.0,
        c * 6.0,
        c * 6.0,
        tint(color_for(PlayerColor::Yellow), 0.20),
    );
    draw_rectangle(
        l,
        t + c * 9.0,
        c * 6.0,
        c * 6.0,
        tint(color_for(PlayerColor::Blue), 0.20),
    );

    // The four white yards.
    for (color, gx, gy) in [
        (PlayerColor::Red, 1.0, 1.0),
        (PlayerColor::Green, 10.0, 1.0),
        (PlayerColor::Yellow, 10.0, 10.0),
        (PlayerColor::Blue, 1.0, 10.0),
    ] {
        draw_rectangle(l + gx * c, t + gy * c, c * 4.0, c * 4.0, BOARD);
        draw_rectangle_lines(
            l + gx * c,
            t + gy * c,
            c * 4.0,
            c * 4.0,
            1.5,
            tint(color_for(color), 0.65),
        );
    }

    // Shared route squares.
    for (index, pos) in TRACK.iter().enumerate() {
        let x = l + pos.x as f32 * c;
        let y = t + pos.y as f32 * c;
        draw_rectangle(x, y, c, c, BOARD);
        draw_rectangle_lines(x, y, c, c, 1.0, Color::new(0.78, 0.79, 0.77, 1.0));
        if SAFE_SQUARES.contains(&index) {
            draw_text(
                "★",
                x + c * 0.22,
                y + c * 0.72,
                c * 0.66,
                Color::new(0.80, 0.61, 0.10, 1.0),
            );
        }
    }

    // Coloured home lanes.
    for color in PlayerColor::ALL {
        let lane = color.home_lane();
        for (step, pos) in lane.iter().enumerate() {
            let x = l + pos.x as f32 * c;
            let y = t + pos.y as f32 * c;
            draw_rectangle(
                x,
                y,
                c,
                c,
                tint(color_for(color), if step == 5 { 0.95 } else { 0.55 }),
            );
            draw_rectangle_lines(x, y, c, c, 1.0, BOARD);
        }
    }

    // Four-colour finish in the middle.
    let x0 = l + c * 6.0;
    let y0 = t + c * 6.0;
    let left_top = vec2(x0, y0);
    let right_top = vec2(x0 + c * 3.0, y0);
    let right_bottom = vec2(x0 + c * 3.0, y0 + c * 3.0);
    let left_bottom = vec2(x0, y0 + c * 3.0);
    let center = vec2(x0 + c * 1.5, y0 + c * 1.5);
    draw_triangle(left_top, right_top, center, color_for(PlayerColor::Green));
    draw_triangle(
        right_top,
        right_bottom,
        center,
        color_for(PlayerColor::Yellow),
    );
    draw_triangle(
        right_bottom,
        left_bottom,
        center,
        color_for(PlayerColor::Blue),
    );
    draw_triangle(left_bottom, left_top, center, color_for(PlayerColor::Red));
    draw_rectangle_lines(x0, y0, c * 3.0, c * 3.0, 1.5, INK);

    // Start squares are tinted to show each player's entry point.
    for color in PlayerColor::ALL {
        let start = TRACK[color.start_index()];
        let x = l + start.x as f32 * c;
        let y = t + start.y as f32 * c;
        draw_rectangle(
            x + c * 0.08,
            y + c * 0.08,
            c * 0.84,
            c * 0.84,
            tint(color_for(color), 0.85),
        );
    }
}

fn draw_tokens(game: &Game, layout: BoardLayout) {
    let c = layout.cell;
    for (player_index, player) in game.players.iter().enumerate() {
        for token_index in 0..4 {
            let (x, y) = token_screen_position(game, player_index, token_index, layout);
            let is_selectable =
                player_index == game.current_player && game.legal_moves().contains(&token_index);
            if is_selectable {
                draw_circle_lines(x, y, c * 0.39, 3.0, GOLD);
            }
            let token_color = color_for(player.color);
            draw_circle(
                x,
                y + c * 0.045,
                c * 0.28,
                Color::new(0.05, 0.06, 0.08, 0.22),
            );
            draw_circle(x, y, c * 0.27, token_color);
            draw_circle_lines(x, y, c * 0.27, 2.0, BOARD);
            draw_circle(x, y, c * 0.09, Color::new(1.0, 1.0, 1.0, 0.88));
        }
    }
}

fn draw_status(game: &Game, layout: BoardLayout) {
    let panel_y = layout.top + layout.side + 16.0;
    let player_color = game.current_color();
    draw_rectangle(14.0, panel_y, screen_width() - 28.0, 126.0, PANEL);
    draw_circle(34.0, panel_y + 27.0, 8.0, color_for(player_color));
    draw_text(
        &format!("{} PLAYER", player_color.label()),
        52.0,
        panel_y + 33.0,
        18.0,
        WHITE,
    );
    draw_text(
        &format!(
            "HOME: {}/4",
            game.players[game.current_player].finished_count()
        ),
        screen_width() - 112.0,
        panel_y + 33.0,
        13.0,
        MUTED,
    );
    draw_text(&game.message, 22.0, panel_y + 54.0, 13.0, MUTED);

    let button_label = match game.phase {
        Phase::AwaitRoll => "ROLL DICE",
        Phase::AwaitToken => "PICK A TOKEN",
        Phase::GameOver => "PLAY AGAIN",
    };
    let button_color = if game.phase == Phase::AwaitToken {
        Color::new(0.28, 0.31, 0.37, 1.0)
    } else {
        color_for(player_color)
    };
    draw_button(
        screen_width() * 0.5 - 104.0,
        panel_y + 63.0,
        208.0,
        52.0,
        button_label,
        button_color,
        WHITE,
    );
    if let Some(dice) = game.dice {
        draw_text(&format!("DICE: {dice}"), 22.0, panel_y + 94.0, 14.0, WHITE);
    }
    if game.phase == Phase::GameOver {
        draw_text("Winner declared", 22.0, panel_y + 116.0, 12.0, GOLD);
    }
}

fn token_screen_position(
    game: &Game,
    player_index: usize,
    token_index: usize,
    layout: BoardLayout,
) -> (f32, f32) {
    let player = &game.players[player_index];
    let progress = player.tokens[token_index];
    let pos = if progress == YARD {
        yard_position(player.color, token_index)
    } else if progress == FINISHED {
        let offsets = [(-0.32, -0.32), (0.32, -0.32), (-0.32, 0.32), (0.32, 0.32)];
        let (dx, dy) = offsets[token_index];
        return (
            layout.left + layout.cell * (7.5 + dx),
            layout.top + layout.cell * (7.5 + dy),
        );
    } else if progress <= 51 {
        TRACK[(player.color.start_index() + progress as usize) % TRACK.len()]
    } else {
        player.color.home_lane()[(progress - 52) as usize]
    };

    let offsets = [(-0.16, -0.16), (0.16, -0.16), (-0.16, 0.16), (0.16, 0.16)];
    let (dx, dy) = offsets[token_index];
    (
        layout.left + (pos.x as f32 + 0.5 + dx) * layout.cell,
        layout.top + (pos.y as f32 + 0.5 + dy) * layout.cell,
    )
}

fn tapped_token(game: &Game, layout: BoardLayout, x: f32, y: f32) -> Option<usize> {
    let radius = layout.cell * 0.48;
    game.legal_moves().into_iter().find(|&token_index| {
        let (tx, ty) = token_screen_position(game, game.current_player, token_index, layout);
        let dx = x - tx;
        let dy = y - ty;
        dx * dx + dy * dy <= radius * radius
    })
}

fn yard_position(color: PlayerColor, token_index: usize) -> Pos {
    let positions = match color {
        PlayerColor::Red => [
            Pos::new(2, 2),
            Pos::new(4, 2),
            Pos::new(2, 4),
            Pos::new(4, 4),
        ],
        PlayerColor::Green => [
            Pos::new(10, 2),
            Pos::new(12, 2),
            Pos::new(10, 4),
            Pos::new(12, 4),
        ],
        PlayerColor::Yellow => [
            Pos::new(10, 10),
            Pos::new(12, 10),
            Pos::new(10, 12),
            Pos::new(12, 12),
        ],
        PlayerColor::Blue => [
            Pos::new(2, 10),
            Pos::new(4, 10),
            Pos::new(2, 12),
            Pos::new(4, 12),
        ],
    };
    positions[token_index]
}

fn color_for(color: PlayerColor) -> Color {
    match color {
        PlayerColor::Red => Color::new(0.88, 0.20, 0.22, 1.0),
        PlayerColor::Green => Color::new(0.12, 0.62, 0.38, 1.0),
        PlayerColor::Yellow => Color::new(0.96, 0.70, 0.12, 1.0),
        PlayerColor::Blue => Color::new(0.16, 0.40, 0.85, 1.0),
    }
}

fn tint(color: Color, strength: f32) -> Color {
    Color::new(
        1.0 - (1.0 - color.r) * strength,
        1.0 - (1.0 - color.g) * strength,
        1.0 - (1.0 - color.b) * strength,
        1.0,
    )
}

fn draw_button(x: f32, y: f32, w: f32, h: f32, label: &str, fill: Color, text: Color) {
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.5, Color::new(1.0, 1.0, 1.0, 0.16));
    let font_size = (h * 0.31).clamp(13.0, 22.0);
    let estimated_width = label.len() as f32 * font_size * 0.53;
    draw_text(
        label,
        x + (w - estimated_width) * 0.5,
        y + h * 0.62,
        font_size,
        text,
    );
}

fn point_in_rect(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}
