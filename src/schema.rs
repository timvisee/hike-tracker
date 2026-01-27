// @generated automatically by Diesel CLI.

diesel::table! {
    groups (id) {
        id -> Text,
        name -> Text,
        scout_group -> Text,
        members -> Text,
        phone_number -> Text,
        start_time -> Nullable<Timestamp>,
        finish_time -> Nullable<Timestamp>,
        created_at -> Timestamp,
        group_number -> Integer,
        route -> Text,
    }
}

diesel::table! {
    posts (id) {
        id -> Text,
        name -> Text,
        post_order -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    scans (id) {
        id -> Text,
        group_id -> Text,
        post_id -> Text,
        arrival_time -> Timestamp,
        departure_time -> Nullable<Timestamp>,
    }
}

diesel::joinable!(scans -> groups (group_id));
diesel::joinable!(scans -> posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(groups, posts, scans,);
