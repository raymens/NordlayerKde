use nordlayer_kde::parser::{
    ConnectionStatus, Gateway, parse_connection_status, parse_gateway_from_status,
    parse_gateways_output,
};

// ── Template-based parser tests ───────────────────────────────────────────────
#[test]
fn template_gateways_one_per_line() {
    let out = "PRIVATE|nl|Netherlands\nPRIVATE|be|Belgium\nSHARED|us|United States\n";
    let gws = parse_gateways_output(out);
    assert_eq!(
        gws,
        vec![
            Gateway {
                id: "nl".into(),
                name: "Netherlands".into(),
                is_private: true
            },
            Gateway {
                id: "be".into(),
                name: "Belgium".into(),
                is_private: true
            },
            Gateway {
                id: "us".into(),
                name: "United States".into(),
                is_private: false
            },
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
fn parses_disconnected_status() {
    let output = "Status: Disconnected";
    assert_eq!(
        parse_connection_status(output),
        ConnectionStatus::Disconnected
    );
}

#[test]
fn parses_connecting_status() {
    let output = "Status: Connecting";
    assert_eq!(
        parse_connection_status(output),
        ConnectionStatus::Connecting
    );
}

#[test]
fn extracts_gateway_when_connected() {
    let output = "Status: Connected\nGateway: uk-lon-1";
    assert_eq!(
        parse_gateway_from_status(output),
        Some("uk-lon-1".to_string())
    );
}
