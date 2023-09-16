// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_group_role"))]
    pub struct UserGroupRole;
}

diesel::table! {
    artifacts (id) {
        id -> Int4,
        filename -> Varchar,
        sha256 -> Nullable<Bytea>,
        sha512 -> Nullable<Bytea>,
        size -> Int4,
        download_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    core_release_artifacts (core_release_id, artifact_id) {
        core_release_id -> Int4,
        artifact_id -> Int4,
    }
}

diesel::table! {
    core_releases (id) {
        id -> Int4,
        version -> Varchar,
        note -> Nullable<Text>,
        date_released -> Nullable<Timestamp>,
        date_uploaded -> Timestamp,
        prerelease -> Nullable<Bool>,
        yanked -> Nullable<Bool>,
        links -> Nullable<Jsonb>,
        uploader_id -> Int4,
        core_id -> Int4,
        platform_id -> Int4,
        system_id -> Int4,
        owner_id -> Int4,
    }
}

diesel::table! {
    core_tags (tag_id, core_id) {
        core_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::table! {
    cores (id) {
        id -> Int4,
        #[max_length = 255]
        slug -> Varchar,
        name -> Varchar,
        description -> Text,
        metadata -> Nullable<Jsonb>,
        links -> Nullable<Jsonb>,
        owner_id -> Int4,
    }
}

diesel::table! {
    groups (id) {
        id -> Int4,
        #[max_length = 255]
        slug -> Varchar,
        name -> Varchar,
        description -> Text,
        links -> Nullable<Jsonb>,
    }
}

diesel::table! {
    platform_tags (tag_id, platform_id) {
        platform_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::table! {
    platforms (id) {
        id -> Int4,
        #[max_length = 255]
        slug -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        description -> Text,
        links -> Nullable<Jsonb>,
        metadata -> Nullable<Jsonb>,
        owner_id -> Int4,
    }
}

diesel::table! {
    system_release_artifacts (artifact_id, system_file_release_id) {
        system_file_release_id -> Int4,
        artifact_id -> Int4,
    }
}

diesel::table! {
    system_releases (id) {
        id -> Int4,
        version -> Varchar,
        note -> Nullable<Text>,
        date_released -> Nullable<Timestamp>,
        date_uploaded -> Timestamp,
        prerelease -> Nullable<Int4>,
        yanked -> Nullable<Bool>,
        links -> Nullable<Jsonb>,
        user_id -> Int4,
        system_id -> Int4,
    }
}

diesel::table! {
    system_tags (tag_id, system_id) {
        system_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::table! {
    systems (id) {
        id -> Int4,
        #[max_length = 255]
        slug -> Varchar,
        name -> Varchar,
        description -> Text,
        manufacturer -> Varchar,
        links -> Nullable<Jsonb>,
        metadata -> Nullable<Jsonb>,
        owner_id -> Int4,
    }
}

diesel::table! {
    tags (id) {
        id -> Int4,
        #[max_length = 255]
        slug -> Varchar,
        description -> Nullable<Text>,
        color -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserGroupRole;

    user_groups (group_id, user_id) {
        group_id -> Int4,
        user_id -> Int4,
        role -> UserGroupRole,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Nullable<Varchar>,
        #[max_length = 255]
        display_name -> Nullable<Varchar>,
        #[max_length = 255]
        avatar_url -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        auth_provider -> Nullable<Varchar>,
        need_reset -> Bool,
        deleted -> Bool,
        description -> Text,
        links -> Nullable<Jsonb>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::joinable!(core_release_artifacts -> artifacts (artifact_id));
diesel::joinable!(core_release_artifacts -> core_releases (core_release_id));
diesel::joinable!(core_releases -> cores (core_id));
diesel::joinable!(core_releases -> groups (owner_id));
diesel::joinable!(core_releases -> platforms (platform_id));
diesel::joinable!(core_releases -> systems (system_id));
diesel::joinable!(core_releases -> users (uploader_id));
diesel::joinable!(core_tags -> cores (core_id));
diesel::joinable!(core_tags -> tags (tag_id));
diesel::joinable!(cores -> groups (owner_id));
diesel::joinable!(platform_tags -> platforms (platform_id));
diesel::joinable!(platform_tags -> tags (tag_id));
diesel::joinable!(platforms -> groups (owner_id));
diesel::joinable!(system_release_artifacts -> artifacts (artifact_id));
diesel::joinable!(system_release_artifacts -> system_releases (system_file_release_id));
diesel::joinable!(system_releases -> systems (system_id));
diesel::joinable!(system_releases -> users (user_id));
diesel::joinable!(system_tags -> systems (system_id));
diesel::joinable!(system_tags -> tags (tag_id));
diesel::joinable!(systems -> groups (owner_id));
diesel::joinable!(user_groups -> groups (group_id));
diesel::joinable!(user_groups -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    artifacts,
    core_release_artifacts,
    core_releases,
    core_tags,
    cores,
    groups,
    platform_tags,
    platforms,
    system_release_artifacts,
    system_releases,
    system_tags,
    systems,
    tags,
    user_groups,
    users,
);
