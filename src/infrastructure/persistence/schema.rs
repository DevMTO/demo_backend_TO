// @generated automatically by Diesel CLI.

diesel::table! {
    document_types (id) {
        id -> Int4,
        #[max_length = 50]
        code -> Varchar,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 100]
        format_regex -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    login_attempts (id) {
        id -> Uuid,
        #[max_length = 255]
        identifier -> Varchar,
        #[max_length = 45]
        ip_address -> Varchar,
        user_agent -> Nullable<Text>,
        success -> Bool,
        failure_reason -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_providers (id) {
        id -> Int4,
        user_id -> Uuid,
        #[max_length = 50]
        provider -> Varchar,
        #[max_length = 255]
        provider_user_id -> Varchar,
        access_token -> Nullable<Text>,
        refresh_token -> Nullable<Text>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        session_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        used_at -> Nullable<Timestamptz>,
        is_revoked -> Bool,
    }
}

diesel::table! {
    user_documents (id) {
        id -> Uuid,
        user_id -> Uuid,
        document_type_id -> Int4,
        #[max_length = 50]
        document_number -> Varchar,
        is_primary -> Bool,
        verified -> Bool,
        verified_at -> Nullable<Timestamptz>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        #[max_length = 255]
        refresh_token_hash -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        refresh_expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        #[max_length = 100]
        device_fingerprint -> Nullable<Varchar>,
        is_active -> Bool,
        revoked_at -> Nullable<Timestamptz>,
        #[max_length = 50]
        revoked_reason -> Nullable<Varchar>,
        last_activity_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Text,
        #[max_length = 100]
        display_name -> Nullable<Varchar>,
        #[max_length = 20]
        role -> Varchar,
        email_verified -> Bool,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_login -> Nullable<Timestamptz>,
        #[max_length = 36]
        created_by -> Nullable<Varchar>,
        #[max_length = 36]
        updated_by -> Nullable<Varchar>,
        version -> Int4,
        mfa_enabled -> Bool,
        mfa_secret -> Nullable<Text>,
        mfa_backup_codes -> Nullable<Jsonb>,
    }
}

diesel::joinable!(oauth_providers -> users (user_id));
diesel::joinable!(refresh_tokens -> user_sessions (session_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(user_documents -> document_types (document_type_id));
diesel::joinable!(user_documents -> users (user_id));
diesel::joinable!(user_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    document_types,
    login_attempts,
    oauth_providers,
    refresh_tokens,
    user_documents,
    user_sessions,
    users,
);
