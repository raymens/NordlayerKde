use NordlayerKde::parser::{
    ConnectionStatus, Gateway, parse_connection_status, parse_gateway_from_status,
    parse_gateways, parse_gateways_output, parse_status_output,
};

// ── Template-based parser tests ───────────────────────────────────────────────

#[test]
fn template_status_connected_with_gateway() {
    let (status, gw) = parse_status_output("Connected\tus-east-1");
    assert_eq!(status, ConnectionStatus::Connected);
    assert_eq!(gw, Some("us-east-1".to_string()));
}

#[test]
fn template_status_disconnected_no_gateway() {
    let (status, gw) = parse_status_output("Disconnected\t");
    assert_eq!(status, ConnectionStatus::Disconnected);
    assert_eq!(gw, None);
}

#[test]
fn template_gateways_one_per_line() {
    let out = "PRIVATE|nl|Netherlands\nPRIVATE|be|Belgium\nSHARED|us|United States\n";
    let gws = parse_gateways_output(out);
    assert_eq!(
        gws,
        vec![
            Gateway { id: "nl".into(), name: "Netherlands".into(), is_private: true },
            Gateway { id: "be".into(), name: "Belgium".into(), is_private: true },
            Gateway { id: "us".into(), name: "United States".into(), is_private: false },
        ]
    );
}

#[test]
fn template_gateways_ignores_blank_lines() {
    let out = "PRIVATE|id1|Private\n\nSHARED|id2|Shared\n";
    let gws = parse_gateways_output(out);
    assert_eq!(gws.len(), 2);
    assert!(gws[0].is_private);
    assert!(!gws[1].is_private);
}

// ── Heuristic / plain-text parser tests (fallback) ────────────────────────────

#[test]
fn parses_pipe_table_output() {
    let output = "Gateway | City\n--------|-----\n| us-east-1 | New York\n| uk-lon-1 | London\n";
    let gateways = parse_gateways(output);

    assert_eq!(gateways, vec!["us-east-1", "uk-lon-1"]);
}

#[test]
fn parses_plain_line_output() {
    let output = "de-ber-1 online\nfr-par-1 online\n";
    let gateways = parse_gateways(output);

    assert_eq!(gateways, vec!["de-ber-1", "fr-par-1"]);
}

#[test]
fn parses_disconnected_status() {
    let output = "Status: Disconnected";
    assert_eq!(parse_connection_status(output), ConnectionStatus::Disconnected);
}

#[test]
fn parses_connecting_status() {
    let output = "Status: Connecting";
    assert_eq!(parse_connection_status(output), ConnectionStatus::Connecting);
}

#[test]
fn extracts_gateway_when_connected() {
    let output = "Status: Connected\nGateway: uk-lon-1";
    assert_eq!(parse_gateway_from_status(output), Some("uk-lon-1".to_string()));
}

