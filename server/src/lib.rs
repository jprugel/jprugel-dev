use std::io;

use rocket::{get, response, tokio::fs};
use rocket_db_pools::diesel::{prelude::*, PgPool};
use rocket_db_pools::{Connection, Database};
use sycamore::prelude::*;

use client::components::article::Details;

#[derive(Database)]
#[database("posts")]
pub struct Db(PgPool);

#[derive(Queryable, Selectable)]
#[diesel(table_name = posts)]
struct Post {
    id: i32,
    title: String,
    date: String,
    body: String,
}

table! {
    posts (id) {
        id -> Int4,
        title -> Text,
        date -> Text,
        body -> Text,
        published -> Bool,
    }
}

#[get("/")]
pub async fn index(mut db: Connection<Db>) -> io::Result<response::content::RawHtml<String>> {
    let index_html = String::from_utf8(fs::read("./client/dist/index.html").await?)
        .expect("index.html should be valid utf-8");

    let rendered = sycamore::render_to_string(|| {
        view! {
            client::App {}
        }
    });

    let db_info: Vec<Post> = posts::table
        .select(Post::as_select())
        .filter(posts::published.eq(true))
        .load(&mut db)
        .await
        .unwrap();

    let mut example_article = String::new();

    let articles: Vec<String> = db_info.iter().map(generate_article).collect();

    articles
        .into_iter()
        .for_each(|x| example_article.push_str(x.as_str()));

    let index_html = index_html.replace("%sycamore.body", &rendered);
    let index_html = index_html.replace("%db_info.body", example_article.as_str());

    Ok(response::content::RawHtml(index_html))
}

fn generate_article(db_info: &Post) -> String {
    let summary = db_info
        .body
        .split_inclusive("  ")
        .collect::<Vec<&str>>();

    let answer = summary[1];

    sycamore::render_to_string(|| {
        view! {
            client::components::article::Article(
                id=db_info.id,
                title=db_info.title.clone(),
                date=db_info.date.clone(),
                body=db_info.body.clone(),
                summary=answer.to_string(),
            )
        }
    })
}