diesel::table! {
    sessions (id) {
        id -> Array<BigInt>,
        data -> Jsonb,
        expiry_date -> Text,
    }
}

diesel::table! {
    demos (steam_id) {
        steam_id -> BigInt,
        demo_id -> BigInt
    }
}
