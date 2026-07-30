mod auth_routes;
mod file_routes;
mod plugin_routes;
mod settings_routes;
mod site_routes;
mod system_routes;

pub fn routes() -> axum::Router<()> {
    auth_routes::routes()
        .merge(settings_routes::routes())
        .merge(file_routes::routes())
        .merge(plugin_routes::routes())
        .merge(site_routes::routes())
        .merge(system_routes::routes())
}