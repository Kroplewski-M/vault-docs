use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::svg::{
    folder::FolderIcon, key::KeyIcon, lock::LockIcon, search::SearchIcon,
};

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Title formatter=|text| format!("{text} - Home") />
        <section class="hero">
            <img src="/logo.svg" alt="vault docs logo" style="height:250px" />
            <h1>"Your documents. Encrypted, organized, searchable."</h1>
            <p>
                "Vault Docs is a privacy-focused document platform. Every file is protected "
                "with its own encryption key, stored in encrypted object storage, and found "
                "again in seconds with AI-powered semantic search."
            </p>
            <a href="/" class="button-default">
                "Create An Account Now"
            </a>
        </section>

        <section class="features">
            <div class="feature">
                <LockIcon class="feature-icon" />
                <h3>"Encrypted storage"</h3>
                <p>"Files live in encrypted object storage, never in the clear."</p>
            </div>

            <div class="feature">
                <KeyIcon class="feature-icon" />
                <h3>"Per-file keys"</h3>
                <p>"Every document is sealed with its own unique encryption key."</p>
            </div>

            <div class="feature">
                <SearchIcon class="feature-icon" />
                <h3>"AI semantic search"</h3>
                <p>"Find documents by meaning, not just filename."</p>
            </div>

            <div class="feature">
                <FolderIcon class="feature-icon" />
                <h3>"Stay organized"</h3>
                <p>"Fold documents into folders and tags that make sense to you."</p>
            </div>
        </section>
    }
}
