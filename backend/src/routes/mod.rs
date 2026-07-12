mod auth_routes;
mod file_routes;
mod settings_routes;

pub fn routes() -> axum::Router<()> {
    auth_routes::routes()
        .merge(settings_routes::routes())
        .merge(file_routes::routes())
}
