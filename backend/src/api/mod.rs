mod auth;
mod settings;

pub fn routes() -> axum::Router<()> {
    auth::routes().merge(settings::routes())
}
