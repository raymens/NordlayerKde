use NordlayerKde::parser::parse_gateways;

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

