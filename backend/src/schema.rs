// @generated automatically by Diesel CLI.

diesel::table! {
    analysis_queue (demo_id) {
        demo_id -> Int8,
        steam_id -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    demo_heatmaps (demo_id, steam_id) {
        demo_id -> Int8,
        steam_id -> Text,
        data -> Text,
    }
}

diesel::table! {
    demo_info (demo_id) {
        demo_id -> Int8,
        map -> Text,
    }
}

diesel::table! {
    demo_player_stats (demo_id, steam_id) {
        demo_id -> Int8,
        steam_id -> Text,
        kills -> Int2,
        deaths -> Int2,
        damage -> Int2,
        assists -> Int2,
    }
}

diesel::table! {
    demo_players (demo_id, steam_id) {
        demo_id -> Int8,
        steam_id -> Text,
        name -> Text,
        team -> Int2,
        color -> Int2,
    }
}

diesel::table! {
    demo_round (demo_id, round_number) {
        demo_id -> Int8,
        round_number -> Int2,
        start_tick -> Int8,
        end_tick -> Int8,
        win_reason -> Text,
        events -> Json,
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

diesel::joinable!(analysis_queue -> demos (demo_id));
diesel::joinable!(demo_heatmaps -> demo_info (demo_id));
diesel::joinable!(demo_info -> demos (demo_id));
diesel::joinable!(demo_player_stats -> demo_info (demo_id));
diesel::joinable!(demo_players -> demo_info (demo_id));
diesel::joinable!(demo_round -> demo_info (demo_id));
diesel::joinable!(processing_status -> demos (demo_id));

diesel::allow_tables_to_appear_in_same_query!(
    analysis_queue,
    demo_heatmaps,
    demo_info,
    demo_player_stats,
    demo_players,
    demo_round,
    demos,
    processing_status,
    sessions,
    users,
);
