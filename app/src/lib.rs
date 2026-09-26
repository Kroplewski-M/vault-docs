use domain::models::CurrentUser;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Outlet, ProtectedParentRoute, Route, Router, Routes},
    path,
};

use crate::{
    components::navbar::Navbar,
    page::{home::Home, vault::Vault},
};
pub mod components;
pub mod page;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
                <link
                    href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@500;600;700&family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap"
                    rel="stylesheet"
                />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[server]
pub async fn get_current_user() -> Result<Option<CurrentUser>, ServerFnError> {
    use axum::Extension;
    let user = leptos_axum::extract::<Option<Extension<CurrentUser>>>().await?;
    Ok(user.map(|Extension(u)| u))
}
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let user = Resource::new(|| (), |_| get_current_user());
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/vault-docs.css" />

        // sets the document title
        <Title text="Vault Docs" />
        // content for this welcome page
        <Navbar />
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=Home />
                    <ProtectedParentRoute
                    path=path!("") view=Outlet
                    condition=move || user.get().map(|r| matches!(r, Ok(Some(_))))
                    redirect_path=|| "/">
                        <Route path=path!("/vault") view=Vault />
                    </ProtectedParentRoute>
                </Routes>
            </main>
        </Router>
    }
}
