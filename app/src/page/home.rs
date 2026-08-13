use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Title formatter=|text| format!("{text} - Home") />
        <p>"home"</p>
    }
}
