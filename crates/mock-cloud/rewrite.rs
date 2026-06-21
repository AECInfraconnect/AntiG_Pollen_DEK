use std::fs;

fn main() {
    let mut content = fs::read_to_string("src/registry.rs").unwrap();
    
    // 1. Replace the router
    let new_router = r#"pub fn router() -> Router<AppState> {
    let internal_routes = Router::new()
        // Tenants
        .route("/tenants", get(list_tenants).post(create_tenant))
        .route("/tenants/:id", get(get_tenant).patch(patch_tenant))
        // Principals
        .route("/principals", get(list_principals).post(create_principal))
        .route("/principals/:id", get(get_principal).patch(patch_principal))
        // Devices
        .route("/devices", get(list_devices).post(create_device))
        .route("/devices/:id", get(get_device).patch(patch_device))
        .route("/devices/:id/capabilities", post(register_capabilities))
        // Agents
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).patch(patch_agent))
        // MCP Servers
        .route("/mcp_servers", get(list_mcp_servers).post(create_mcp_server))
        .route("/mcp_servers/:id", get(get_mcp_server).patch(patch_mcp_server))
        // Tools
        .route("/tools", get(list_tools).post(create_tool))
        .route("/tools/:id", get(get_tool).patch(patch_tool))
        // Resources
        .route("/resources", get(list_resources).post(create_resource))
        .route("/resources/:id", get(get_resource).patch(patch_resource))
        // Relationships
        .route("/relationships", get(list_relationships).post(create_relationship))
        // Policies
        .route("/policies", get(list_policies).post(create_policy))
        .route("/policies/:id", get(get_policy).patch(patch_policy))
        // PEP Deployments
        .route("/pep_deployments", get(list_pep_deployments).post(create_pep_deployment))
        .route("/pep_deployments/:id", get(get_pep_deployment).patch(patch_pep_deployment));

    Router::new()
        .nest("/v1/registry", internal_routes.clone())
        .nest("/v1/tenants/:tenant_id/registry", internal_routes)
}"#;

    // Replace the router function (find `pub fn router() -> Router<AppState> {` up to `}`)
    let start_idx = content.find("pub fn router() -> Router<AppState> {").unwrap();
    let end_idx = content[start_idx..].find("\n}").unwrap() + start_idx + 2;
    content.replace_range(start_idx..end_idx, new_router);

    // 2. For every handler that takes Path(id): Path<String>, change to Path(params): Path<std::collections::HashMap<String, String>>
    // and insert `let id = params.get("id").cloned().unwrap_or_default();`
    content = content.replace("Path(id): Path<String>", "Path(params): Path<std::collections::HashMap<String, String>>");
    
    // We also need to insert the id extraction. We can look for `Path(params): Path<std::collections::HashMap<String, String>>`
    // and then the next `{` to insert the line.
    
    let mut final_content = String::new();
    let mut in_handler_with_params = false;
    for line in content.lines() {
        if line.contains("Path(params): Path<std::collections::HashMap<String, String>>") {
            in_handler_with_params = true;
            final_content.push_str(line);
            final_content.push('\n');
            continue;
        }
        
        if in_handler_with_params && line.contains("{") {
            final_content.push_str(line);
            final_content.push('\n');
            final_content.push_str("    let id = params.get(\"id\").cloned().unwrap_or_default();\n");
            in_handler_with_params = false;
            continue;
        }
        
        final_content.push_str(line);
        final_content.push('\n');
    }
    
    fs::write("src/registry.rs", final_content).unwrap();
}
