#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Reconnecting,
    NotLoggedIn,
    Unknown,
}

impl ConnectionStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Disconnected => "disconnected",
            Self::Connecting => "connecting",
            Self::Reconnecting => "reconnecting",
            Self::NotLoggedIn => "not logged in",
            Self::Unknown => "unknown",
        }
    }
}

pub fn parse_connection_status(output: &str) -> ConnectionStatus {
    let lower = output.to_ascii_lowercase();

    if lower.contains("not logged in") || lower.contains("logged out") {
        ConnectionStatus::NotLoggedIn
    } else if lower.contains("reconnecting") {
        ConnectionStatus::Reconnecting
    } else if lower.contains("connecting") {
        ConnectionStatus::Connecting
    } else if lower.contains("disconnected") {
        ConnectionStatus::Disconnected
    } else if lower.contains("connected") {
        ConnectionStatus::Connected
    } else {
        ConnectionStatus::Unknown
    }
}

pub fn parse_gateways(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            // Skip common table header/separator lines.
            let lower = trimmed.to_ascii_lowercase();
            if lower.contains("gateway") && lower.contains("city") {
                return None;
            }
            if trimmed.chars().all(|c| c == '-' || c == '+' || c == '|') {
                return None;
            }

            let first = trimmed
                .trim_start_matches(['|', '-', '*', ' '])
                .split(['|', ' '])
                .find(|part| !part.is_empty())?;

            Some(first.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ConnectionStatus, parse_connection_status, parse_gateways};

    #[test]
    fn parses_connected_status() {
        let output = "Status: Connected\nGateway: us-east-1";
        assert_eq!(parse_connection_status(output), ConnectionStatus::Connected);
    }

    #[test]
    fn parses_not_logged_in_status() {
        let output = "Error: not logged in";
        assert_eq!(
            parse_connection_status(output),
            ConnectionStatus::NotLoggedIn
        );
    }

    #[test]
    fn parses_unknown_status() {
        assert_eq!(parse_connection_status("mystery state"), ConnectionStatus::Unknown);
    }

    #[test]
    fn parses_simple_rows() {
        let output = "us-east-1 online\nuk-lon-1 online\n";
        assert_eq!(parse_gateways(output), vec!["us-east-1", "uk-lon-1"]);
    }

    #[test]
    fn skips_header_and_separators() {
        let output = "Gateway | City\n------- | ----\n| de-ber-1 | Berlin\n";
        assert_eq!(parse_gateways(output), vec!["de-ber-1"]);
    }
}



