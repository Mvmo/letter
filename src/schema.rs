// @generated automatically by Diesel CLI.

diesel::table! {
    badges (id) {
        id -> Integer,
        name -> Text,
        color -> Text,
    }
}

diesel::table! {
    tasks (id) {
        id -> Integer,
        text -> Text,
        badge_id -> Nullable<Integer>,
    }
}

diesel::joinable!(tasks -> badges (badge_id));

diesel::allow_tables_to_appear_in_same_query!(
    badges,
    tasks,
);
