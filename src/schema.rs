table! {
    authors (arxiv_id, auth) {
        arxiv_id -> Text,
        auth -> Text,
    }
}

table! {
    papers (arxiv_id) {
        arxiv_id -> Text,
        title -> Text,
        #[sql_name = "abstract"]
        abstract_ -> Text,
        prim_sub -> Text,
    }
}

table! {
    pins (id) {
        id -> Text,
        ref_id -> Nullable<Text>,
        arxiv_id -> Text,
        pub_date -> Timestamp,
    }
}

table! {
    subjects (arxiv_id, sub) {
        arxiv_id -> Text,
        sub -> Text,
    }
}

table! {
    update_time (subject) {
        subject -> Text,
        rss_time -> Timestamp,
    }
}

joinable!(authors -> papers (arxiv_id));
joinable!(pins -> papers (arxiv_id));
joinable!(subjects -> papers (arxiv_id));

allow_tables_to_appear_in_same_query!(
    authors,
    papers,
    pins,
    subjects,
    update_time,
);
