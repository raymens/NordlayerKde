use crate::cli::NordLayerCli;
use crate::parser::{
    ConnectionStatus, GATEWAYS_TEMPLATE, Gateway, parse_connection_status,
    parse_gateway_from_status, parse_gateways_output, parse_login_state,
};
use ksni::MenuItem;
use ksni::menu::{StandardItem, SubMenu};
use notify_rust::Notification;

fn format_notification_summary(action: &str, status: &str, is_error: bool) -> String {
    if is_error {
        format!("NordLayer: {} failed ({})", action, status)
    } else {
        format!("NordLayer: {} ({})", action, status)
    }
}

fn format_action_details(action: &str, cli_output: &str, status: &str) -> String {
    let details = if cli_output.trim().is_empty() {
        format!("{} completed", action)
    } else {
        cli_output.lines().take(8).collect::<Vec<_>>().join("\n")
    };
    format!("{}\nStatus: {}", details, status)
}

/// Returns 1.0 if the point (px, py) lies inside the shield polygon, 0.0 otherwise.
/// The shield is a pentagon: rectangular top [left_x..right_x, top_y..split_y]
/// tapering to a single point at (tip_x, bot_y).
fn shield_coverage(
    px: f32,
    py: f32,
    top_y: f32,
    split_y: f32,
    bot_y: f32,
    left_x: f32,
    right_x: f32,
    tip_x: f32,
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
    let top_y = 1.0f32; // top of the shield
    let split_y = 13.5f32; // where rect ends and triangle begins
    let bot_y = 20.5f32; // bottom tip y
    let left_x = 2.5f32; // left edge
    let right_x = 19.5f32; // right edge
    let tip_x = 11.0f32; // horizontal centre (tip of the point)

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
                    shield_coverage(
                        px + dx,
                        py + dy,
                        top_y,
                        split_y,
                        bot_y,
                        left_x,
                        right_x,
                        tip_x,
                    )
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
    ksni::Icon {
        width: W,
        height: H,
        data,
    }
}

pub struct NordLayerTray {
    cli: NordLayerCli,
    last_status: String,
    connection_status: ConnectionStatus,
    gateways: Vec<Gateway>,
    current_gateway: Option<String>,
    is_logged_in: Option<bool>,
}

#[derive(Debug, Clone)]
struct StatusSnapshot {
    connection_status: ConnectionStatus,
    last_status: String,
    current_gateway: Option<String>,
    is_logged_in: Option<bool>,
}

impl StatusSnapshot {
    fn from_status_output(output: &str) -> Self {
        let is_logged_in = parse_login_state(output);
        let connection_status = parse_connection_status(output);
        let (last_status, current_gateway) = match connection_status {
            ConnectionStatus::Connected => match parse_gateway_from_status(output) {
                Some(gateway) => (format!("connected: {}", gateway), Some(gateway)),
                None => ("connected".to_string(), None),
            },
            _ => (connection_status.label().to_string(), None),
        };

        Self {
            connection_status,
            last_status,
            current_gateway,
            is_logged_in,
        }
    }

    fn unknown() -> Self {
        Self {
            connection_status: ConnectionStatus::Unknown,
            last_status: "unknown".to_string(),
            current_gateway: None,
            is_logged_in: None,
        }
    }
}

impl NordLayerTray {
    pub fn new() -> Self {
        let mut tray = Self {
            cli: NordLayerCli::default(),
            last_status: "idle".to_string(),
            connection_status: ConnectionStatus::Unknown,
            gateways: Vec::new(),
            current_gateway: None,
            is_logged_in: None,
        };
        tray.refresh_status();
        tray.refresh_gateways();
        tray
    }

    fn notify(summary: &str, body: &str) {
        let safe_body = if body.trim().is_empty() {
            "No details available"
        } else {
            body
        };
        let _ = Notification::new()
            .appname("NordLayer KDE Tray")
            .summary(summary)
            .body(safe_body)
            .show();
    }

    fn notify_action_result(&self, action: &str, details: &str, is_error: bool) {
        // Some KDE notification layouts de-emphasize/hide the body, so keep
        // the most important info in the summary line too.
        let summary = format_notification_summary(action, &self.last_status, is_error);
        Self::notify(&summary, details);
    }

    fn apply_snapshot(&mut self, snapshot: StatusSnapshot) {
        self.connection_status = snapshot.connection_status;
        self.last_status = snapshot.last_status;
        self.current_gateway = snapshot.current_gateway;
        self.is_logged_in = snapshot.is_logged_in;
    }

    fn gateway_label(&self, gateway: &Gateway) -> String {
        if self.current_gateway.as_deref() == Some(gateway.id.as_str()) {
            format!("✓ {}", gateway.name)
        } else {
            gateway.name.clone()
        }
    }

    fn build_gateway_items(
        &self,
        gateways: Vec<&Gateway>,
        empty_label: &str,
    ) -> Vec<MenuItem<Self>> {
        if gateways.is_empty() {
            return vec![
                StandardItem {
                    label: empty_label.to_string(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            ];
        }

        gateways
            .into_iter()
            .map(|gateway| {
                let gateway_id = gateway.id.clone();
                let gateway_label = self.gateway_label(gateway);
                StandardItem {
                    label: gateway_label,
                    activate: Box::new(move |tray: &mut Self| {
                        tray.run_action("connect", &["connect", &gateway_id]);
                    }),
                    ..Default::default()
                }
                .into()
            })
            .collect()
    }

    fn run_action(&mut self, action: &str, args: &[&str]) {
        match self.cli.run(args) {
            Ok(output) => {
                self.refresh_status();
                let body = format_action_details(action, &output, &self.last_status);
                self.notify_action_result(action, &body, false);
            }
            Err(err) => {
                self.refresh_status();
                let body = format!("{} failed: {}\nStatus: {}", action, err, self.last_status);
                self.notify_action_result(action, &body, true);
            }
        }
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
                self.notify_action_result("refresh gateways", &err.to_string(), true);
                self.gateways = Vec::new();
            }
        }
    }

    fn refresh_status(&mut self) {
        match self.cli.run(&["status"]) {
            Ok(output) => {
                self.apply_snapshot(StatusSnapshot::from_status_output(&output));
            }
            Err(_) => {
                self.apply_snapshot(StatusSnapshot::unknown());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_action_details, format_notification_summary};

    #[test]
    fn notification_summary_success_contains_action_and_status() {
        let summary = format_notification_summary("connect", "connected: nl", false);
        assert_eq!(summary, "NordLayer: connect (connected: nl)");
    }

    #[test]
    fn notification_summary_error_contains_failed_marker() {
        let summary = format_notification_summary("connect", "unknown", true);
        assert_eq!(summary, "NordLayer: connect failed (unknown)");
    }

    #[test]
    fn action_details_uses_fallback_when_cli_output_empty() {
        let details = format_action_details("disconnect", "", "disconnected");
        assert_eq!(details, "disconnect completed\nStatus: disconnected");
    }

    #[test]
    fn action_details_truncates_to_eight_lines_and_appends_status() {
        let output = ["l1", "l2", "l3", "l4", "l5", "l6", "l7", "l8", "l9", "l10"].join("\n");
        let details = format_action_details("connect", &output, "connected: nl");
        assert!(details.contains("l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8"));
        assert!(!details.contains("l9"));
        assert!(details.ends_with("Status: connected: nl"));
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
            ConnectionStatus::Connected => (76, 175, 80), // green
            ConnectionStatus::Connecting | ConnectionStatus::Reconnecting => (255, 152, 0), // orange
            ConnectionStatus::Disconnected => (229, 57, 53),                                // red
            ConnectionStatus::NotLoggedIn => (255, 193, 7),                                 // amber
            ConnectionStatus::Unknown => (158, 158, 158),                                   // grey
        };
        vec![make_shield_icon(r, g, b)]
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let private_gateways: Vec<&Gateway> =
            self.gateways.iter().filter(|g| g.is_private).collect();
        let shared_gateways: Vec<&Gateway> =
            self.gateways.iter().filter(|g| !g.is_private).collect();

        let private_items = self.build_gateway_items(private_gateways, "No private gateways");
        let shared_items = self.build_gateway_items(shared_gateways, "No shared gateways");

        vec![
            StandardItem {
                label: format!("Status: {}", self.last_status),
                enabled: false,
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Private Gateways".to_string(),
                submenu: private_items,
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Shared Gateways".to_string(),
                submenu: shared_items,
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
            {
                if self.is_logged_in == Some(true) {
                    StandardItem {
                        label: "Logged in".to_string(),
                        enabled: false,
                        ..Default::default()
                    }
                    .into()
                } else {
                    StandardItem {
                        label: "Login".to_string(),
                        activate: Box::new(|tray: &mut Self| tray.run_action("login", &["login"])),
                        ..Default::default()
                    }
                    .into()
                }
            },
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
