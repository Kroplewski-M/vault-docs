use chrono::Utc;
use domain::models::File;
use leptos::prelude::*;

use crate::components::{file_card::FileCard, svg::empty_files::EmptyFilesIcon};

#[component]
pub fn Vault() -> impl IntoView {
    let files = populate_files();
    let has_files = !files.is_empty();
    //let files = Vec::<File>::new();
    view! {
        <h1>"My Vault"</h1>
        <Show when= move|| {has_files} fallback=EmptyFiles>
            {
                files.iter().cloned()
                .map(|f| view!{<FileCard file=f/>}).collect_view()
            }
        </Show>
    }
}
#[component]
fn EmptyFiles() -> impl IntoView {
    view! {
        <div class="empty-state">
            <EmptyFilesIcon />
            <p>"No files yet"</p>
        </div>
    }
}
fn populate_files() -> Vec<File> {
    let entries: [(&str, usize, &str, &str); 20] = [
        ("file", 54_532, "pdf", "Mat"),
        ("invoice", 128_450, "docx", "Mat"),
        ("notes", 1_024, "txt", "Jane"),
        ("report", 87_300, "odt", "Jane"),
        ("contract", 210_880, "pdf", "Alex"),
        ("resume", 45_120, "docx", "Mat"),
        ("minutes", 3_072, "txt", "Jane"),
        ("proposal", 152_640, "odt", "Alex"),
        ("budget", 34_210, "pdf", "Mat"),
        ("summary", 8_192, "txt", "Jane"),
        ("agreement", 98_765, "docx", "Alex"),
        ("draft", 12_500, "odt", "Mat"),
        ("memo", 2_048, "txt", "Jane"),
        ("letter", 15_360, "pdf", "Alex"),
        ("statement", 76_800, "docx", "Mat"),
        ("timesheet", 4_096, "txt", "Jane"),
        ("checklist", 6_144, "odt", "Alex"),
        ("presentation", 305_152, "pdf", "Mat"),
        ("plan", 51_200, "docx", "Jane"),
        ("review", 9_500, "txt", "Alex"),
    ];

    entries
        .into_iter()
        .map(|(name, size_byte, ext, created_by)| File {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            size_byte,
            ext: ext.to_string(),
            added: Utc::now(),
            created_by: created_by.to_string(),
        })
        .collect()
}
