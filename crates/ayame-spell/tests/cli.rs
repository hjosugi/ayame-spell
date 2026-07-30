use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

struct Project {
    _temp: TempDir,
    root: PathBuf,
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Project {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("temporary project");
        let root = temp.path().join("project");
        let config_dir = temp.path().join("user-config");
        let cache_dir = temp.path().join("user-cache");
        fs::create_dir_all(&root).expect("project directory");
        fs::create_dir_all(&config_dir).expect("config directory");
        fs::create_dir_all(&cache_dir).expect("cache directory");
        Self {
            _temp: temp,
            root,
            config_dir,
            cache_dir,
        }
    }

    fn write(&self, relative: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent");
        }
        fs::write(&path, contents).expect("fixture");
        path
    }

    fn command(&self) -> Command {
        let mut command = Command::new(assert_cmd::cargo::cargo_bin!("ayame-spell"));
        command
            .current_dir(&self.root)
            .env("AYAME_SPELL_CONFIG_DIR", &self.config_dir)
            .env("AYAME_SPELL_CACHE_DIR", &self.cache_dir)
            .env_remove("AYAME_SPELL_REGISTRY")
            .env("NO_COLOR", "1")
            .env("CLICOLOR_FORCE", "0");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().expect("run ayame-spell")
    }
}

fn assert_code(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn normalized(project: &Project, output: &Output) -> String {
    let root = project.root.to_string_lossy().replace('\\', "/");
    let stdout = String::from_utf8_lossy(&output.stdout)
        .replace('\\', "/")
        .replace(&root, "<PROJECT>")
        .replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&output.stderr)
        .replace('\\', "/")
        .replace(&root, "<PROJECT>")
        .replace("\r\n", "\n");
    format!(
        "status: {}\nstdout:\n{}stderr:\n{}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    )
}

fn json_records(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON Lines record"))
        .collect()
}

#[test]
fn exit_codes_cover_clean_findings_and_errors() {
    let project = Project::new();
    project.write("clean.md", "This is clean.\n");
    project.write("finding.md", "This is teh finding.\n");

    assert_code(&project.run(&["clean.md"]), 0);
    assert_code(&project.run(&["finding.md"]), 1);
    assert_code(&project.run(&["missing.md"]), 2);
}

#[test]
fn human_brief_and_json_formats_are_snapshotted() {
    let project = Project::new();
    project.write("input.md", "This is teh fixture.\n");

    for format in ["human", "brief", "json"] {
        let output = project.run(&["check", "--format", format, "input.md"]);
        assert_code(&output, 1);
        insta::assert_snapshot!(format!("{format}_format"), normalized(&project, &output));
    }
}

#[test]
fn every_rule_has_an_end_to_end_fixture() {
    struct Case {
        fixture: &'static str,
        target: &'static str,
        config: &'static str,
        kind: &'static str,
    }

    let cases = [
        Case {
            fixture: "typo.md",
            target: "typo.md",
            config: "",
            kind: "typo",
        },
        Case {
            fixture: "unknown-word.md",
            target: "unknown-word.md",
            config: "[check]\nmode = \"dictionary\"\n[japanese]\nenabled = false\n",
            kind: "unknown-word",
        },
        Case {
            fixture: "ja-variant.md",
            target: "ja-variant.md",
            config: "[japanese]\nkatakana-style = \"long\"\n",
            kind: "ja-variant",
        },
        Case {
            fixture: "fullwidth-alnum.md",
            target: "fullwidth-alnum.md",
            config: "",
            kind: "fullwidth-alnum",
        },
        Case {
            fixture: "halfwidth-kana.md",
            target: "halfwidth-kana.md",
            config: "",
            kind: "halfwidth-kana",
        },
        Case {
            fixture: "fullwidth-space.rs",
            target: "fullwidth-space.rs",
            config: "",
            kind: "fullwidth-space",
        },
    ];

    for case in cases {
        let project = Project::new();
        project.write(
            "ayame-spell.toml",
            format!("# integration fixture\n{}", case.config),
        );
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/rules")
            .join(case.fixture);
        project.write(case.target, fs::read(fixture).expect("rule fixture"));

        let output = project.run(&["check", "--format", "json", case.target]);
        assert_code(&output, 1);
        let records = json_records(&output);
        assert_eq!(records[0]["kind"], case.kind, "fixture {}", case.fixture);
        assert_eq!(records.last().unwrap()["type"], "summary");
    }
}

#[test]
fn fix_is_idempotent() {
    let project = Project::new();
    let input = project.write("input.rs", "teh １２３ＡＢＣ\n");

    let first = project.run(&["fix", "input.rs"]);
    assert_code(&first, 0);
    let fixed = fs::read_to_string(&input).expect("fixed input");
    assert_eq!(fixed, "the 123ABC\n");

    let second = project.run(&["fix", "input.rs"]);
    assert_code(&second, 0);
    assert_eq!(fs::read_to_string(input).unwrap(), fixed);
    assert!(String::from_utf8_lossy(&second.stderr).contains("0 fixed"));
}

#[test]
fn both_project_config_names_are_discovered() {
    for name in ["ayame-spell.toml", ".ayame-spell.toml"] {
        let project = Project::new();
        project.write(name, "[corrections.words]\nmistkae = \"mistake\"\n");
        project.write("input.md", "A mistkae.\n");

        let output = project.run(&["check", "--format", "json", "input.md"]);
        assert_code(&output, 1);
        assert_eq!(json_records(&output)[0]["word"], "mistkae");
    }
}

#[test]
fn global_and_project_configs_merge() {
    let project = Project::new();
    fs::write(
        project.config_dir.join("config.toml"),
        "[corrections.words]\nglobalbad = \"globalgood\"\n",
    )
    .expect("global config");
    project.write(
        "ayame-spell.toml",
        "[corrections.words]\nprojectbad = \"projectgood\"\n",
    );
    project.write("input.md", "globalbad projectbad\n");

    let output = project.run(&["check", "--format", "json", "input.md"]);
    assert_code(&output, 1);
    let records = json_records(&output);
    let words: Vec<&str> = records
        .iter()
        .filter_map(|record| record["word"].as_str())
        .collect();
    assert_eq!(words, ["globalbad", "projectbad"]);

    let config = project.run(&["config"]);
    assert_code(&config, 0);
    let stdout = String::from_utf8_lossy(&config.stdout);
    assert!(stdout.contains("globalbad"));
    assert!(stdout.contains("projectbad"));
}

#[test]
fn overrides_and_inline_directives_take_precedence() {
    let project = Project::new();
    project.write(
        "ayame-spell.toml",
        "[check]\nmode = \"off\"\n[japanese]\nenabled = false\n\
         [[overrides]]\npaths = [\"docs/**\"]\nmode = \"corrections\"\n",
    );
    project.write("docs/guide.md", "teh\n");
    project.write("src/main.rs", "teh\n");

    let override_output =
        project.run(&["check", "--format", "json", "docs/guide.md", "src/main.rs"]);
    assert_code(&override_output, 1);
    let issues: Vec<Value> = json_records(&override_output)
        .into_iter()
        .filter(|record| record["type"] == "issue")
        .collect();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["path"], "docs/guide.md");

    project.write(
        "ayame-spell.toml",
        "[check]\nmode = \"corrections\"\n[japanese]\nenabled = false\n",
    );
    project.write(
        "directives.md",
        "teh\nteh # ayame-spell:ignore-line\n\
         # ayame-spell:ignore-next-line\nteh\nteh\n",
    );
    let directives = project.run(&["check", "--format", "json", "directives.md"]);
    assert_code(&directives, 1);
    let lines: Vec<u64> = json_records(&directives)
        .iter()
        .filter_map(|record| record["line"].as_u64())
        .collect();
    assert_eq!(lines, [1, 5]);
}

#[test]
fn words_commands_cover_collect_add_and_noninteractive_triage() {
    let project = Project::new();
    project.write("input.md", "teh teh\n");

    let collect = project.run(&["words", "collect", "--plain", "input.md"]);
    assert_code(&collect, 0);
    assert_eq!(String::from_utf8_lossy(&collect.stdout), "teh\n");

    let add = project.run(&["words", "add", "ProjectTerm"]);
    assert_code(&add, 0);
    let words = fs::read_to_string(project.root.join("ayame-words.txt")).unwrap();
    assert!(words.lines().any(|word| word == "ProjectTerm"));

    project.write("clean.md", "This is clean.\n");
    let triage = project.run(&["words", "triage", "clean.md"]);
    assert_code(&triage, 0);
    assert!(String::from_utf8_lossy(&triage.stdout).contains("nothing to triage"));
}

struct RegistryServer {
    url: String,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl RegistryServer {
    fn start() -> Self {
        let dictionary = b"fixtureword\n".to_vec();
        let digest = Sha256::digest(&dictionary)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let index = json!({
            "version": 1,
            "dictionaries": [{
                "name": "fixture",
                "language": "en",
                "kind": "wordlist",
                "description": "Fixture dictionary",
                "file": "fixture.txt",
                "sha256": digest,
                "entries": 1
            }]
        })
        .to_string()
        .into_bytes();

        let listener = TcpListener::bind("127.0.0.1:0").expect("registry listener");
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let mut request = [0_u8; 4096];
                        let size = stream.read(&mut request).unwrap_or(0);
                        let request_text = String::from_utf8_lossy(&request[..size]);
                        let path = request_text
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            .unwrap_or("/");
                        let (status, content_type, body): (&str, &str, &[u8]) = match path {
                            "/index.json" => ("200 OK", "application/json", &index),
                            "/fixture.txt" => ("200 OK", "text/plain", &dictionary),
                            _ => ("404 Not Found", "text/plain", b"not found"),
                        };
                        write!(
                            stream,
                            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\n\
                             Content-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .unwrap();
                        stream.write_all(body).unwrap();
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("registry server: {error}"),
                }
            }
        });

        Self {
            url: format!("http://{address}/index.json"),
            stop,
            thread: Some(handle),
        }
    }
}

impl Drop for RegistryServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            handle.join().expect("registry thread");
        }
    }
}

#[test]
fn dictionary_commands_use_an_offline_fixture_registry() {
    let project = Project::new();
    let registry = RegistryServer::start();
    let run = |args: &[&str]| {
        let mut command = project.command();
        command
            .env("AYAME_SPELL_REGISTRY", &registry.url)
            .env("NO_PROXY", "127.0.0.1,localhost")
            .env("no_proxy", "127.0.0.1,localhost")
            .args(args)
            .output()
            .expect("dictionary command")
    };

    let list = run(&["dict", "list"]);
    assert_code(&list, 0);
    assert!(String::from_utf8_lossy(&list.stdout).contains("fixture"));

    let add = run(&["dict", "add", "fixture"]);
    assert_code(&add, 0);
    let cached = project.cache_dir.join("dicts/fixture.txt");
    assert_eq!(fs::read_to_string(&cached).unwrap(), "fixtureword\n");
    assert!(fs::read_to_string(project.root.join("ayame-spell.toml"))
        .unwrap()
        .contains("registry:fixture"));

    let update = run(&["dict", "update"]);
    assert_code(&update, 0);
    assert!(String::from_utf8_lossy(&update.stdout).contains("updated fixture"));

    let remove = run(&["dict", "remove", "fixture"]);
    assert_code(&remove, 0);
    assert!(!cached.exists());
    assert!(!fs::read_to_string(project.root.join("ayame-spell.toml"))
        .unwrap()
        .contains("registry:fixture"));
}

#[test]
fn init_config_and_completions_subcommands_work() {
    let project = Project::new();

    let init = project.run(&["init"]);
    assert_code(&init, 0);
    assert!(project.root.join("ayame-spell.toml").is_file());

    let config = project.run(&["config"]);
    assert_code(&config, 0);
    assert!(String::from_utf8_lossy(&config.stdout).contains("mode = \"corrections\""));

    let completions = project.run(&["completions", "bash"]);
    assert_code(&completions, 0);
    assert!(String::from_utf8_lossy(&completions.stdout).contains("_ayame-spell"));
}

fn lsp_frame(value: Value) -> Vec<u8> {
    let body = value.to_string();
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

#[test]
fn lsp_completes_initialize_and_shutdown() {
    let project = Project::new();
    let mut input = Vec::new();
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "capabilities": {},
            "rootUri": null
        }
    })));
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    })));
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown",
        "params": null
    })));
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0",
        "method": "exit",
        "params": null
    })));

    let mut child = project
        .command()
        .args(["lsp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start LSP");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&input)
        .expect("LSP input");
    let output = child.wait_with_output().expect("LSP output");

    assert_code(&output, 0);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"serverInfo\""));
    assert!(stdout.contains("\"id\":2"));
}

#[test]
fn crlf_and_paths_with_spaces_are_reported_portably() {
    let project = Project::new();
    project.write(
        "directory with spaces/input file.rs",
        b"teh\r\nlet\xe3\x80\x80value = 1;\r\n",
    );

    let output = project.run(&[
        "check",
        "--format",
        "json",
        "directory with spaces/input file.rs",
    ]);
    assert_code(&output, 1);
    let records = json_records(&output);
    let issues: Vec<&Value> = records
        .iter()
        .filter(|record| record["type"] == "issue")
        .collect();
    assert_eq!(issues.len(), 2);
    assert!(Path::new(issues[0]["path"].as_str().unwrap())
        .ends_with(Path::new("directory with spaces/input file.rs")));
    assert_eq!(issues[0]["line"], 1);
    assert_eq!(issues[0]["column"], 1);
    assert_eq!(issues[1]["line"], 2);
    assert_eq!(issues[1]["column"], 4);
}
