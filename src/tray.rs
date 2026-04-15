use crate::cli::NordLayerCli;
use crate::parser::parse_gateways;
use ksni::MenuItem;
use ksni::menu::StandardItem;
use notify_rust::Notification;

pub struct NordLayerTray {
    cli: NordLayerCli,
    last_status: String,
}

impl NordLayerTray {
    pub fn new() -> Self {
        Self {
            cli: NordLayerCli::default(),
            last_status: "idle".to_string(),
        }
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
                self.last_status = format!("{}: ok", action);
                Self::notify("NordLayer", &body);
            }
            Err(err) => {
                self.last_status = format!("{}: failed", action);
                Self::notify("NordLayer error", &err.to_string());
            }
        }
    }

    fn list_gateways(&mut self) {
        match self.cli.run(&["gateways"]) {
            Ok(output) => {
                let gateways = parse_gateways(&output);
                if gateways.is_empty() {
                    self.last_status = "gateways: none parsed".to_string();
                    Self::notify("NordLayer gateways", &output);
                } else {
                    self.last_status = format!("gateways: {} found", gateways.len());
                    Self::notify("NordLayer gateways", &gateways.join("\n"));
                }
            }
            Err(err) => {
                self.last_status = "gateways: failed".to_string();
                Self::notify("NordLayer error", &err.to_string());
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
                label: "Quit".to_string(),
                activate: Box::new(|_tray: &mut Self| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

