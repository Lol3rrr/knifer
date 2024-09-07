// @generated automatically by Diesel CLI.

diesel::table! {
    demos (steam_id, demo_id) {
        steam_id -> Int8,
        demo_id -> Int8,
    }
}

diesel::table! {
    sessions (id) {
        id -> Array<Nullable<Int8>>,
        data -> Nullable<Jsonb>,
        expiry_date -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    demos,
    sessions,
);
