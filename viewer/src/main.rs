use std::{str::FromStr, sync::OnceLock};

use chrono::{DateTime, FixedOffset, Local, Offset, TimeZone};
use gix::{bstr::ByteSlice, date::time::Format, objs::FindExt};
use graphql_client::GraphQLQuery;
use graphql_client::reqwest::post_graphql_blocking;
use prettytable::row;
use reqwest::blocking::Client;
use snafu::{ResultExt, Whatever, whatever};

static GH_TOKEN: OnceLock<String> = OnceLock::new();

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gh_graphql/schema.docs.graphql",
    query_path = "gh_graphql/search_pr.graphql",
    response_derives = "Debug,Serialize,Deserialize"
)]
struct RepoView;
fn gh_token() -> &'static str {
    GH_TOKEN.get_or_init(|| std::env::var("GH_TOKEN").unwrap())
}

fn main() -> Result<(), Whatever> {
    let client = Client::builder()
        .user_agent("graphql-rust/0.1.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", gh_token())).unwrap(),
            ))
            .collect(),
        )
        .build()
        .whatever_context("client build")?;
    let variables = repo_view::Variables {
        owner: "facebook".to_string(),
        name: "rocksdb".to_string(),
    };

    let response_body =
        post_graphql_blocking::<RepoView, _>(&client, "https://api.github.com/graphql", variables)
            .unwrap();

    dbg!(&response_body);

    let response_data: repo_view::ResponseData = response_body.data.expect("missing response data");

    let stars: Option<i64> = response_data
        .repository
        .as_ref()
        .map(|repo| repo.stargazers.total_count);

    let mut table = prettytable::Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row!(b => "issue", "comments"));

    for issue in response_data
        .repository
        .expect("missing repository")
        .issues
        .nodes
        .expect("issue nodes is null")
        .iter()
        .flatten()
    {
        table.add_row(row!(issue.title, issue.comments.total_count));
    }

    table.printstd();
    Ok(())
}

struct Cli {
    git_dir: Option<String>,
    trunk_name: String,
}

#[test]
fn test_git() {
    use gix;
    let repo = gix::discover(".").unwrap();
    let head = repo.head().unwrap();
    let oid = head.id().unwrap();
    let commit: gix::Commit<'_> = repo.find_commit(oid).unwrap();

    let raw_time = commit.time().unwrap();

    let time: DateTime<Local> = chrono::DateTime::from_naive_utc_and_offset(
        chrono::NaiveDateTime::from_timestamp(raw_time.seconds, 0),
        TimeZone::from_offset(&FixedOffset::east_opt(raw_time.offset).unwrap()),
    );

    println!(
        "{} {} {}",
        oid.shorten_or_id(),
        commit
            .message()
            .unwrap()
            .title
            .strip_suffix(b"\n")
            .unwrap()
            .to_str_lossy(),
        time.format("%y-%m-%d %H:%M")
    );
    let trunk = "main";

    let r = repo.find_reference(trunk).unwrap();
    let tc = repo.find_commit(r.target().id()).unwrap();

    let mut walk = tc.ancestors().all().unwrap();
    while let Some(Ok(info)) = walk.next() {
        if info.parent_ids().any(|o| &oid == &o) {
            println!("{}", info.object().unwrap().message_raw().unwrap())
        }
    }
}
