use bytes::Buf;
use home::home_dir;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Result as HyperResult, Server,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, fs, process::Command};

static NOTFOUND: &[u8] = b"Not Found";
lazy_static::lazy_static! {
    static ref CONFIG:OnceCell<UpdateHookConfig> = OnceCell::new();
    static ref CONFIG_DIR:OnceCell<String> = OnceCell::new();
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Project {
    pub repo: String,
    pub command: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]

struct UpdateHookConfig {
    pub port: i32,
    pub path: Option<String>,
    pub project: Vec<Project>,
}

fn get_config() -> &'static UpdateHookConfig {
    CONFIG.get().unwrap()
}
fn get_config_dir() -> &'static str {
    CONFIG_DIR.get().unwrap()
}

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "example_global_404_handler=debug")
    }

    tracing_subscriber::fmt::init();
    CONFIG_DIR
        .set(format!("{}/.hook", home_dir().unwrap().display()))
        .unwrap();
    let config = env::var("HOOK_CONFIG").unwrap_or_else(|_| "config.toml".to_owned());

    let data = fs::read_to_string(format!("{}/{}", get_config_dir(), config)).unwrap();

    let config_data = toml::from_str(data.as_str()).unwrap();
    CONFIG.set(config_data).unwrap();

    fs::create_dir_all(format!("{}/logs", get_config_dir())).unwrap();
    let addr = format!("127.0.0.1:{}", get_config().port).parse().unwrap();

    let make_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_req)) });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        println!("{}", e)
    }
}

async fn handle_req(req: Request<Body>) -> HyperResult<Response<Body>> {
    let paths = get_config().path.as_deref().unwrap_or(&"/");

    println!("REQUEST {}", req.uri().path());
    match (req.method(), req.uri().path()) {
        (&Method::POST, path) if path == paths => handle_github(req).await,
        _ => Ok(not_found()),
    }
}
fn not_found() -> Response<Body> {
    println!("NOT FOUND!");
    Response::builder()
        // .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

async fn handle_github(req: Request<Body>) -> HyperResult<Response<Body>> {
    println!("HANDING GITHUB REQUEST");
    // Serve a file by asynchronously reading it by chunks using tokio-util crate.
    let whole_body = hyper::body::aggregate(req).await?;
    // Decode as JSON...
    let data: Result<Value, serde_urlencoded::de::Error> =
        serde_urlencoded::from_reader(whole_body.reader());
    // let data: Result<Value, serde_json::Error> = serde_json::from_str(body.as_str());
    if let Ok(json) = data {
        let fuck_github: Value = serde_json::from_str(json["payload"].as_str().unwrap()).unwrap();
        let name = fuck_github["repository"]["full_name"].as_str().unwrap();

        for project in &get_config().project {
            if project.repo.to_lowercase() == name.to_lowercase() {
                let command = project.command.split(' ').collect::<Vec<&str>>();
                let mut iter = command.iter();
                let command_name = iter.next().unwrap();
                let args: Vec<&&str> = iter.collect();
                let result = Command::new(command_name).args(args).output();
                if let Ok(result) = result {
                    let file_name = format!(
                        "{}-{}",
                        project.repo.replace("/", "_"),
                        chrono::offset::Local::now()
                    );
                    fs::write(
                        format!("{}/logs/{}-stderr", get_config_dir(), file_name),
                        result.stderr,
                    )
                    .unwrap();
                    fs::write(
                        format!("{}/logs/{}-stdout", get_config_dir(), file_name),
                        result.stdout,
                    )
                    .unwrap();
                }
                println!("Same repo")
            }
        }
    }

    Ok(not_found())
}
