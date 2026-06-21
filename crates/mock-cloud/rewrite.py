import re

with open("src/registry.rs", "r") as f:
    content = f.read()

# Fix the router part to add both paths
router_code = """pub fn router() -> Router<AppState> {
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
}"""

content = re.sub(r'pub fn router\(\) -> Router<AppState> \{.*?\n\}', router_code, content, flags=re.DOTALL)

# Now fix all handlers
# 1. Remove Path(_tenant_id): Path<String> from list/create handlers
content = re.sub(r'Path\(_tenant_id\):\s*Path<String>,\s*', '', content)

# 2. Change Path((_tenant_id, id)): Path<(String, String)> to Path(params): Path<std::collections::HashMap<String, String>>
content = re.sub(r'Path\(\(_tenant_id,\s*id\)\):\s*Path<\(String,\s*String\)>\s*,', 'Path(params): Path<std::collections::HashMap<String, String>>,', content)

# 3. Add let id = params.get("id").cloned().unwrap_or_default(); right inside the handler where needed.
# Let's replace `let reg = state.registry.lock().unwrap();` with `let id = params.get("id").cloned().unwrap_or_default();\n    let reg = state.registry.lock().unwrap();`
# Wait, this only applies to detail handlers. Let's do it generally:
# Find all handlers that use Path(params).
def add_id_extraction(match):
    prefix = match.group(1)
    body = match.group(2)
    return prefix + '{\n    let id = params.get("id").cloned().unwrap_or_default();\n' + body

content = re.sub(r'(Path\(params\): Path<std::collections::HashMap<String, String>>.*?\)\s*->\s*impl IntoResponse\s*)\{\s*\n(.*?)', add_id_extraction, content, flags=re.DOTALL)

with open("src/registry.rs", "w") as f:
    f.write(content)
