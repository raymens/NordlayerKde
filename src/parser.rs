// ── Shared types ──────────────────────────────────────────────────────────────

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
// ── Template-based parsing (primary) ─────────────────────────────────────────
//
// Use these constants with `NordLayerCli::run_formatted`.
// If field names don't match your nordlayer version, adjust them here and recompile.
//
//   nordlayer status   -f "{{.Status}}\t{{.Server}}"
//   nordlayer gateways -f "{{.Name}}"
//
// Verify available fields: nordlayer status --help / nordlayer gateways --help
/// Go template for `nordlayer status -f STATUS_TEMPLATE`.
/// Emits a single tab-separated line:  `<status>\t<server>`
/// e.g.  `Connected\tus-east-1`   or   `Disconnected\t`
pub const STATUS_TEMPLATE: &str = "{{.Status}}\t{{.Server}}";
/// Go template for `nordlayer gateways -f GATEWAYS_TEMPLATE`.
/// Outputs one gateway ID per line (both shared and private gateways).
pub const GATEWAYS_TEMPLATE: &str = "{{range .Shared}}{{.Id}}\n{{end}}{{range .Private}}{{.Id}}\n{{end}}";
/// Parse `nordlayer status -f STATUS_TEMPLATE` output.
/// Returns `(ConnectionStatus, Option<gateway_name>)`.
pub fn parse_status_output(output: &str) -> (ConnectionStatus, Option<String>) {
    let line = output.trim();
    let mut parts = line.splitn(2, '\t');
    let status_str = parts.next().unwrap_or("").trim();
    let gateway = parts
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    (parse_connection_status(status_str), gateway)
}
/// Parse `nordlayer gateways -f GATEWAYS_TEMPLATE` output.
/// Each non-empty line is a gateway name.
pub fn parse_gateways_output(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}
// ── Heuristic / plain-text parsers (fallback) ─────────────────────────────────
/// Classify a status string from any output format.
/// Handles both the template output ("Connected") and full human-readable text.
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
/// Extract a gateway name from plain-text `nordlayer status` output.
/// Looks for lines whose key contains "gateway" or "server".
pub fn parse_gateway_from_status(output: &str) -> Option<String> {
    for line in output.lines() {
        let lower = line.to_ascii_lowercase();
        if (lower.contains("gateway") || lower.contains("server")) && lower.contains(':') {
            if let Some(value) = line.splitn(2, ':').nth(1) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}
/// Parse plain-text `nordlayer gateways` table output (fallback).
pub fn parse_gateways(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
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
// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    // Template-based parsers
    #[test]
    fn template_status_connected_with_gateway() {
        let output = "Connected\tus-east-1";
        let (status, gw) = parse_status_output(output);
        assert_eq!(status, ConnectionStatus::Connected);
        assert_eq!(gw, Some("us-east-1".to_string()));
    }
    #[test]
    fn template_status_disconnected_no_gateway() {
        let output = "Disconnected\t";
        let (status, gw) = parse_status_output(output);
        assert_eq!(status, ConnectionStatus::Disconnected);
        assert_eq!(gw, None);
    }
    #[test]
    fn template_gateways_one_per_line() {
        let output = "us-east-1\nuk-lon-1\nde-ber-1\n";
        assert_eq!(
            parse_gateways_output(output),
            vec!["us-east-1", "uk-lon-1", "de-ber-1"]
        );
    }
    #[test]
    fn template_gateways_ignores_blank_lines() {
        let output = "us-east-1\n\nuk-lon-1\n";
        assert_eq!(parse_gateways_output(output), vec!["us-east-1", "uk-lon-1"]);
    }
    // Heuristic parsers (plain-text fallback)
    #[test]
    fn parses_connected_status() {
        let output = "Status: Connected\nGateway: us-east-1";
        assert_eq!(parse_connection_status(output), ConnectionStatus::Connected);
    }
    #[test]
    fn extracts_gateway_from_status_output() {
        let output = "Status: Connected\nGateway: us-east-1";
        assert_eq!(
            parse_gateway_from_status(output),
            Some("us-east-1".to_string())
        );
    }
    #[test]
    fn extracts_gateway_with_current_prefix() {
        let output = "Status: Connected\nCurrent gateway: de-ber-1";
        assert_eq!(
            parse_gateway_from_status(output),
            Some("de-ber-1".to_string())
        );
    }
    #[test]
    fn returns_none_when_disconnected_no_gateway() {
        let output = "Status: Disconnected";
        assert_eq!(parse_gateway_from_status(output), None);
    }
    #[test]
    fn parses_not_logged_in_status() {
        let output = "Error: not logged in";
        assert_eq!(parse_connection_status(output), ConnectionStatus::NotLoggedIn);
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
