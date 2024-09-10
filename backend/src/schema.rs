diesel::table! {
    sessions (id) {
        id -> Array<BigInt>,
        steamid -> Nullable<Text>,
        expiry_date -> Text,
    }
}

diesel::table! {
    demos (steam_id, demo_id) {
        steam_id -> BigInt,
        demo_id -> BigInt
    }
}

diesel::table! {
    users (steamid) {
        steamid -> Text,
        name -> Text
    }
}
