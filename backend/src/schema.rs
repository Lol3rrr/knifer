// @generated automatically by Diesel CLI.

diesel::table! {
    demos (steam_id, demo_id) {
        steam_id -> Text,
        demo_id -> Int8,
    }
}

diesel::table! {
    sessions (id) {
        id -> Text,
        steamid -> Nullable<Text>,
        expiry_date -> Text,
    }
}

diesel::table! {
    users (steamid) {
        steamid -> Text,
        name -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    demos,
    sessions,
    users,
);
