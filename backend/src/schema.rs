// @generated automatically by Diesel CLI.

diesel::table! {
    demo_info (demo_id) {
        demo_id -> Int8,
        map -> Text,
    }
}

diesel::table! {
    demos (demo_id) {
        steam_id -> Text,
        demo_id -> Int8,
    }
}

diesel::table! {
    processing_status (demo_id) {
        demo_id -> Int8,
        info -> Int2,
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

diesel::joinable!(demo_info -> demos (demo_id));
diesel::joinable!(processing_status -> demos (demo_id));

diesel::allow_tables_to_appear_in_same_query!(demo_info, demos, processing_status, sessions, users,);
