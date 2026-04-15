use crate::cli::NordLayerCli;
use crate::parser::{
    ConnectionStatus, GATEWAYS_TEMPLATE,
    parse_connection_status, parse_gateway_from_status, parse_gateways_output,
};
use ksni::MenuItem;
use ksni::menu::{StandardItem, SubMenu};
use notify_rust::Notification;

/// Returns 1.0 if the point (px, py) lies inside the shield polygon, 0.0 otherwise.
/// The shield is a pentagon: rectangular top [left_x..right_x, top_y..split_y]
/// tapering to a single point at (tip_x, bot_y).
fn shield_coverage(
    px: f32, py: f32,
    top_y: f32, split_y: f32, bot_y: f32,
    left_x: f32, right_x: f32, tip_x: f32,
) -> f32 {
    if py < top_y || py > bot_y {
        return 0.0;
    }
    let inside = if py <= split_y {
        px >= left_x && px <= right_x
    } else {
        let t = (py - split_y) / (bot_y - split_y);
        let l = left_x + t * (tip_x - left_x);
        let r = right_x - t * (right_x - tip_x);
        px >= l && px <= r
    };
    if inside { 1.0 } else { 0.0 }
}

/// Generates a 22×22 shield-shaped icon as an ARGB32 big-endian pixmap.
/// Uses 4× supersampling per pixel for smooth edges.
fn make_shield_icon(r: u8, g: u8, b: u8) -> ksni::Icon {
    const W: i32 = 22;
    const H: i32 = 22;

    // Shield geometry (in pixel-space, origin top-left)
    let top_y = 1.0f32;    // top of the shield
    let split_y = 13.5f32; // where rect ends and triangle begins
    let bot_y = 20.5f32;   // bottom tip y
    let left_x = 2.5f32;   // left edge
    let right_x = 19.5f32; // right edge
    let tip_x = 11.0f32;   // horizontal centre (tip of the point)

    // 4-tap rotated grid supersampling offsets
    const TAPS: [(f32, f32); 4] = [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)];

    let mut data = Vec::with_capacity((W * H * 4) as usize);
    for y in 0..H {
        for x in 0..W {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let coverage: f32 = TAPS
                .iter()
                .map(|&(dx, dy)| {
                    shield_coverage(px + dx, py + dy, top_y, split_y, bot_y, left_x, right_x, tip_x)
                })
                .sum::<f32>()
                / TAPS.len() as f32;

            let alpha = (coverage * 255.0).round() as u8;
            // ARGB big-endian
            data.push(alpha);
            data.push(r);
            data.push(g);
            data.push(b);
        }
    }
    ksni::Icon { width: W, height: H, data }
}

pub struct NordLayerTray {
    cli: NordLayerCli,
    last_status: String,
    connection_status: ConnectionStatus,
    gateways: Vec<String>,
}

impl NordLayerTray {
    pub fn new() -> Self {
        let mut tray = Self {
            cli: NordLayerCli::default(),
            last_status: "idle".to_string(),
            connection_status: ConnectionStatus::Unknown,
            gateways: Vec::new(),
        };
        tray.refresh_status();
        tray.refresh_gateways();
        tray
    }

    fn notify(summary: &str, body: &str) {
        let _ = Notification::new().summary(summary).body(body).show();
    }

    fn run_action(&mut self, action: &str, args: &[&str]) {
        match self.cli.run(args) {
            Ok(output) => {
                let body = if output.is_empty() {
                    format!("{} completed", action)
                } else {
                    output.lines().take(8).collect::<Vec<_>>().join("\n")
                };
                Self::notify("NordLayer", &body);
            }
            Err(err) => {
                Self::notify("NordLayer error", &err.to_string());
            }
        }

        self.refresh_status();
    }

    fn list_gateways(&mut self) {
        self.refresh_gateways();
        self.refresh_status();
    }

    fn refresh_gateways(&mut self) {
        match self.cli.run_formatted(&["gateways"], GATEWAYS_TEMPLATE) {
            Ok(output) => {
                self.gateways = parse_gateways_output(&output);
            }
            Err(err) => {
                Self::notify("NordLayer error", &err.to_string());
                self.gateways = Vec::new();
            }
        }
    }

    fn refresh_status(&mut self) {
        match self.cli.run(&["status"]) {
            Ok(output) => {
                let status = parse_connection_status(&output);
                self.connection_status = status;
                self.last_status = match status {
                    ConnectionStatus::Connected => match parse_gateway_from_status(&output) {
                        Some(gw) => format!("connected: {}", gw),
                        None => "connected".to_string(),
                    },
                    _ => status.label().to_string(),
                };
            }
            Err(_) => {
                self.connection_status = ConnectionStatus::Unknown;
                self.last_status = "unknown".to_string();
            }
        }
    }
}

impl ksni::Tray for NordLayerTray {
    fn id(&self) -> String {
        "com.raymens.nordlayer-kde".to_string()
    }

    fn title(&self) -> String {
        "NordLayer".to_string()
    }

    /// Return empty so KDE prefers our coloured pixmap below.
    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let (r, g, b): (u8, u8, u8) = match self.connection_status {
            ConnectionStatus::Connected => (76, 175, 80),          // green
            ConnectionStatus::Connecting
            | ConnectionStatus::Reconnecting => (255, 152, 0),     // orange
            ConnectionStatus::Disconnected => (229, 57, 53),       // red
            ConnectionStatus::NotLoggedIn => (255, 193, 7),        // amber
            ConnectionStatus::Unknown => (158, 158, 158),          // grey
        };
        vec![make_shield_icon(r, g, b)]
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        // Build the per-gateway "Connect to…" submenu items.
        let gateway_items: Vec<MenuItem<Self>> = if self.gateways.is_empty() {
            vec![StandardItem {
                label: "No gateways — click Refresh Gateways".to_string(),
                enabled: false,
                ..Default::default()
            }
            .into()]
        } else {
            self.gateways
                .iter()
                .map(|gw| {
                    let gw = gw.clone();
                    StandardItem {
                        label: gw.clone(),
                        activate: Box::new(move |tray: &mut Self| {
                            tray.run_action("connect", &["connect", gw.as_str()]);
                        }),
                        ..Default::default()
                    }
                    .into()
                })
                .collect()
        };

        vec![
            StandardItem {
                label: format!("Status: {}", self.last_status),
                enabled: false,
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Connect to...".to_string(),
                submenu: gateway_items,
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Disconnect".to_string(),
                activate: Box::new(|tray: &mut Self| {
                    tray.run_action("disconnect", &["disconnect"])
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Login".to_string(),
                activate: Box::new(|tray: &mut Self| tray.run_action("login", &["login"])),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Refresh Status".to_string(),
                activate: Box::new(|tray: &mut Self| tray.refresh_status()),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Refresh Gateways".to_string(),
                activate: Box::new(|tray: &mut Self| tray.list_gateways()),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".to_string(),
                activate: Box::new(|_tray: &mut Self| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}
