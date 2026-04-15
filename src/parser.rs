// ── Shared types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gateway {
    pub id: String,
    pub name: String,
    pub is_private: bool,
}

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
/// Go template for `nordlayer gateways -f GATEWAYS_TEMPLATE`.
/// Outputs one gateway per line as: PRIVATE|id|name  or  SHARED|id|name
pub const GATEWAYS_TEMPLATE: &str = "{{range .Private}}PRIVATE|{{.Id}}|{{.Name}}{{\"\\n\"}}{{end}}{{range .Shared}}SHARED|{{.Id}}|{{.Name}}{{\"\\n\"}}{{end}}";
/// Parse `nordlayer gateways -f GATEWAYS_TEMPLATE` output.
/// Each line is: `PRIVATE|id|name` or `SHARED|id|name`
pub fn parse_gateways_output(output: &str) -> Vec<Gateway> {
    fn first_marker(s: &str) -> Option<(usize, bool, usize)> {
        let private = s.find("PRIVATE|").map(|idx| (idx, true, "PRIVATE|".len()));
        let shared = s.find("SHARED|").map(|idx| (idx, false, "SHARED|".len()));
        match (private, shared) {
            (Some(p), Some(sh)) => Some(if p.0 <= sh.0 { p } else { sh }),
            (Some(p), None) => Some(p),
            (None, Some(sh)) => Some(sh),
            (None, None) => None,
        }
    }

    let normalized = output.replace("\\n", "\n");
    let mut rest = normalized.as_str();
    let mut gateways = Vec::new();

    while let Some((marker_pos, is_private, marker_len)) = first_marker(rest) {
        rest = &rest[marker_pos + marker_len..];
        let next_pos = first_marker(rest)
            .map(|(idx, _, _)| idx)
            .unwrap_or(rest.len());
        let payload = rest[..next_pos].trim();

        let mut parts = payload.splitn(2, '|');
        let Some(id) = parts.next().map(str::trim).filter(|s| !s.is_empty()) else {
            rest = &rest[next_pos..];
            continue;
        };
        let Some(name) = parts.next().map(str::trim).filter(|s| !s.is_empty()) else {
            rest = &rest[next_pos..];
            continue;
        };

        gateways.push(Gateway {
            id: id.to_string(),
            name: name.to_string(),
            is_private,
        });

        rest = &rest[next_pos..];
    }

    gateways
}
// ── Heuristic / plain-text parsers (fallback) ─────────────────────────────────
/// Classify a status string from any output format.
/// Handles both the template output ("Connected") and full human-readable text.
pub fn parse_connection_status(output: &str) -> ConnectionStatus {
    let lower = output.to_ascii_lowercase();
    if lower.contains("not logged in") || lower.contains("logged out") {
        ConnectionStatus::NotLoggedIn
    } else if lower.contains("not connected") {
        ConnectionStatus::Disconnected
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

/// Parse login state from plain-text `nordlayer status` output.
/// Returns:
/// - `Some(true)` when output indicates logged in
/// - `Some(false)` when output indicates logged out/not logged in
/// - `None` when login state cannot be determined
pub fn parse_login_state(output: &str) -> Option<bool> {
    let lower = output.to_ascii_lowercase();
    if lower.contains("not logged in") || lower.contains("logged out") {
        return Some(false);
    }

    for line in output.lines() {
        let lower_line = line.to_ascii_lowercase();
        if lower_line.starts_with("login:") {
            if lower_line.contains("logged in") {
                return Some(true);
            }
            if lower_line.contains("not logged in") || lower_line.contains("logged out") {
                return Some(false);
            }
        }
    }

    None
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
// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    // Template-based parsers
    #[test]
    fn template_gateways_one_per_line() {
        let output = "PRIVATE|id1|Private Gateway\nSHARED|id2|Shared Gateway\n";
        let gateways = parse_gateways_output(output);
        assert_eq!(
            gateways,
            vec![
                Gateway {
                    id: "id1".into(),
                    name: "Private Gateway".into(),
                    is_private: true
                },
                Gateway {
                    id: "id2".into(),
                    name: "Shared Gateway".into(),
                    is_private: false
                },
            ]
        );
    }
    #[test]
    fn template_gateways_ignores_blank_lines() {
        let output = "PRIVATE|id1|Private\n\nSHARED|id2|Shared\n";
        let gateways = parse_gateways_output(output);
        assert_eq!(gateways.len(), 2);
        assert!(gateways[0].is_private);
        assert!(!gateways[1].is_private);
    }

    #[test]
    fn template_gateways_supports_escaped_newline_stream() {
        let output = "PRIVATE|id1|Private\\nSHARED|id2|Shared\\n";
        let gateways = parse_gateways_output(output);
        assert_eq!(gateways.len(), 2);
        assert_eq!(gateways[0].id, "id1");
        assert_eq!(gateways[1].id, "id2");
    }

    #[test]
    fn template_gateways_supports_glued_stream_without_newlines() {
        let output = "PRIVATE|id1|Approved GatewaySHARED|id2|United States";
        let gateways = parse_gateways_output(output);
        assert_eq!(gateways.len(), 2);
        assert_eq!(gateways[0].name, "Approved Gateway");
        assert_eq!(gateways[1].name, "United States");
    }

    #[test]
    fn template_gateways_keeps_spaces_in_names() {
        let output = "PRIVATE|approved-gw|My Private Network\n";
        let gateways = parse_gateways_output(output);
        assert_eq!(gateways.len(), 1);
        assert_eq!(gateways[0].name, "My Private Network");
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
        assert_eq!(
            parse_connection_status(output),
            ConnectionStatus::NotLoggedIn
        );
    }

    #[test]
    fn parses_vpn_not_connected_status() {
        let output = "Login: Logged in [user org]\nVPN: Not Connected\n";
        assert_eq!(
            parse_connection_status(output),
            ConnectionStatus::Disconnected
        );
    }

    #[test]
    fn parses_login_state_logged_in() {
        let output = "Login: Logged in [user org]\nVPN: Not Connected\n";
        assert_eq!(parse_login_state(output), Some(true));
    }

    #[test]
    fn parses_login_state_logged_out() {
        let output = "Login: Not logged in\n";
        assert_eq!(parse_login_state(output), Some(false));
    }
    #[test]
    fn parses_unknown_status() {
        assert_eq!(
            parse_connection_status("mystery state"),
            ConnectionStatus::Unknown
        );
    }
}
