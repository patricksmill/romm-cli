use romm_cli::tui::openapi::EndpointRegistry;

#[test]
fn fixture_path_item_registers_get_with_merged_path_param() {
    let json = include_str!("fixtures/openapi_path_item.json");
    let reg = EndpointRegistry::from_openapi_json(json).expect("parse fixture");
    assert_eq!(reg.endpoints.len(), 1);
    let ep = &reg.endpoints[0];
    assert_eq!(ep.method, "GET");
    assert_eq!(ep.path, "/items/{id}");
    assert_eq!(ep.path_params.len(), 1);
    assert_eq!(ep.path_params[0].name, "id");
}
