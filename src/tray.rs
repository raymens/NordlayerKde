use crate::cli::NordLayerCli;
use crate::parser::{parse_connection_status, parse_gateways};
use ksni::MenuItem;
use ksni::menu::StandardItem;
use notify_rust::Notification;

pub struct NordLayerTray {
    cli: NordLayerCli,
    last_status: String,
}

impl NordLayerTray {
    pub fn new() -> Self {
        let mut tray = Self {
            cli: NordLayerCli::default(),
            last_status: "idle".to_string(),
        };
        tray.refresh_status();
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
        match self.cli.run(&["gateways"]) {
            Ok(output) => {
                let gateways = parse_gateways(&output);
                if gateways.is_empty() {
                    Self::notify("NordLayer gateways", &output);
                } else {
                    Self::notify("NordLayer gateways", &gateways.join("\n"));
                }
            }
            Err(err) => {
                Self::notify("NordLayer error", &err.to_string());
            }
        }

        self.refresh_status();
    }

    fn refresh_status(&mut self) {
        self.last_status = match self.cli.run(&["status"]) {
            Ok(output) => parse_connection_status(&output).label().to_string(),
            Err(_) => "unknown".to_string(),
        };
    }
}

impl ksni::Tray for NordLayerTray {
    fn id(&self) -> String {
        "com.raymens.nordlayer-kde".to_string()
    }

    fn title(&self) -> String {
        "NordLayer".to_string()
    }

    fn icon_name(&self) -> String {
        "network-vpn".to_string()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: format!("Status: {}", self.last_status),
                enabled: false,
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
                label: "Connect".to_string(),
                activate: Box::new(|tray: &mut Self| tray.run_action("connect", &["connect"])),
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
                label: "List Gateways".to_string(),
                activate: Box::new(|tray: &mut Self| tray.list_gateways()),
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
                label: "Quit".to_string(),
                activate: Box::new(|_tray: &mut Self| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

