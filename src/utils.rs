#[macro_export]
macro_rules! debug_query {
    ($query:ident) => {{
        let q = diesel::debug_query::<diesel::pg::Pg, _>(&$query);
        rocket::trace::debug!(query = %q);
    }};
}
