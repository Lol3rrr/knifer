// @generated automatically by Diesel CLI.

diesel::table! {
    analysis_queue (demo_id) {
        demo_id -> Text,
        steam_id -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    demo_head_to_head (demo_id, player, enemy) {
        demo_id -> Text,
        player -> Text,
        enemy -> Text,
        kills -> Int2,
    }
}

diesel::table! {
    demo_heatmaps (demo_id, steam_id, team) {
        demo_id -> Text,
        steam_id -> Text,
        team -> Text,
        data -> Text,
    }
}

diesel::table! {
    demo_info (demo_id) {
        demo_id -> Text,
        map -> Text,
    }
}

diesel::table! {
    demo_player_stats (demo_id, steam_id) {
        demo_id -> Text,
        steam_id -> Text,
        kills -> Int2,
        deaths -> Int2,
        damage -> Int2,
        assists -> Int2,
    }
}

diesel::table! {
    demo_players (demo_id, steam_id) {
        demo_id -> Text,
        steam_id -> Text,
        name -> Text,
        team -> Int2,
        color -> Int2,
    }
}

diesel::table! {
    demo_round (demo_id, round_number) {
        demo_id -> Text,
        round_number -> Int2,
        start_tick -> Int8,
        end_tick -> Int8,
        win_reason -> Text,
        events -> Json,
    }
}

diesel::table! {
    demo_teams (demo_id, team) {
        demo_id -> Text,
        team -> Int2,
        end_score -> Int2,
        start_name -> Text,
    }
}

diesel::table! {
    demos (steam_id, demo_id) {
        steam_id -> Text,
        demo_id -> Text,
        uploaded_at -> Timestamptz,
    }
}

diesel::table! {
    processing_status (demo_id) {
        demo_id -> Text,
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

diesel::allow_tables_to_appear_in_same_query!(
    analysis_queue,
    demo_head_to_head,
    demo_heatmaps,
    demo_info,
    demo_player_stats,
    demo_players,
    demo_round,
    demo_teams,
    demos,
    processing_status,
    sessions,
    users,
);
