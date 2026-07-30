use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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
            .env_remove("GITHUB_ACTIONS")
            .env("NO_COLOR", "1")
            .env("CLICOLOR_FORCE", "0");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().expect("run ayame-spell")
    }

    fn run_with_stdin(&self, args: &[&str], input: &[u8]) -> Output {
        let mut child = self
            .command()
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run ayame-spell with stdin");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input)
            .expect("write stdin");
        child.wait_with_output().expect("stdin command output")
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

    for format in ["human", "brief", "json", "github", "sarif"] {
        let output = project.run(&["check", "--format", format, "input.md"]);
        assert_code(&output, 1);
        insta::assert_snapshot!(format!("{format}_format"), normalized(&project, &output));
    }

    let sarif = project.run(&["check", "--format", "sarif", "input.md"]);
    let document: Value = serde_json::from_slice(&sarif.stdout).unwrap();
    assert_eq!(
        document["$schema"],
        "https://json.schemastore.org/sarif-2.1.0.json"
    );
    assert_eq!(document["version"], "2.1.0");
    assert_eq!(document["runs"][0]["results"][0]["ruleId"], "typo");
    assert_eq!(
        document["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        ayame_spell_core::IssueKind::ALL.len()
    );

    let mut automatic = project.command();
    let automatic = automatic
        .env("GITHUB_ACTIONS", "true")
        .args(["check", "input.md"])
        .output()
        .unwrap();
    assert_code(&automatic, 1);
    assert!(String::from_utf8_lossy(&automatic.stdout).starts_with("::warning "));

    let mut explicit = project.command();
    let explicit = explicit
        .env("GITHUB_ACTIONS", "true")
        .args(["check", "--format", "human", "input.md"])
        .output()
        .unwrap();
    assert_code(&explicit, 1);
    assert!(String::from_utf8_lossy(&explicit.stdout).contains("[typo]"));
}

#[test]
fn explain_and_rules_cover_every_code_in_both_languages() {
    let project = Project::new();
    let rules = project.run(&["rules", "--lang", "en"]);
    assert_code(&rules, 0);
    let rules = String::from_utf8_lossy(&rules.stdout);

    for code in [
        "typo",
        "unknown-word",
        "ja-variant",
        "fullwidth-alnum",
        "halfwidth-kana",
        "fullwidth-space",
    ] {
        assert!(rules.contains(code));
        for language in ["en", "ja"] {
            let explanation = project.run(&["explain", code, "--lang", language]);
            assert_code(&explanation, 0);
            assert!(String::from_utf8_lossy(&explanation.stdout).contains(code));
        }
    }

    let alias = project.run(&["--list-rules", "--lang", "ja"]);
    assert_code(&alias, 0);
    assert!(String::from_utf8_lossy(&alias.stdout).contains("カタカナ"));

    let unknown = project.run(&["explain", "not-a-rule"]);
    assert_code(&unknown, 2);
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown issue code"));
}

#[test]
fn baseline_suppresses_legacy_findings_survives_line_shifts_and_prunes() {
    let project = Project::new();
    let legacy = project.write(
        "legacy.md",
        "This is teh existing finding.\nThis recieve remains too.\n",
    );

    let create = project.run(&["baseline", "legacy.md"]);
    assert_code(&create, 0);
    let baseline_path = project.root.join("ayame-spell-baseline.json");
    let baseline: Value =
        serde_json::from_slice(&fs::read(&baseline_path).expect("baseline file")).unwrap();
    assert_eq!(baseline["version"], 1);
    assert_eq!(
        baseline["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["count"].as_u64().unwrap())
            .sum::<u64>(),
        2
    );

    let clean = project.run(&["check", "--format", "json", "legacy.md"]);
    assert_code(&clean, 0);
    assert_eq!(json_records(&clean)[0]["issues"], 0);

    fs::write(
        &legacy,
        "A clean inserted line.\nThis is teh existing finding.\nThis recieve remains too.\nA new teh appears here.\n",
    )
    .unwrap();
    let shifted = project.run(&["check", "--format", "json", "legacy.md"]);
    assert_code(&shifted, 1);
    let shifted_records = json_records(&shifted);
    assert_eq!(
        shifted_records
            .iter()
            .filter(|record| record["type"] == "issue")
            .count(),
        1
    );
    assert_eq!(shifted_records[0]["line"], 4);

    let all = project.run(&["check", "--no-baseline", "--format", "json", "legacy.md"]);
    assert_code(&all, 1);
    assert_eq!(
        json_records(&all)
            .iter()
            .filter(|record| record["type"] == "issue")
            .count(),
        3
    );

    fs::write(
        &legacy,
        "A clean inserted line.\nThis is the existing finding.\nThis recieve remains too.\nA new teh appears here.\n",
    )
    .unwrap();
    let prune = project.run(&["baseline", "--prune", "legacy.md"]);
    assert_code(&prune, 0);
    assert!(String::from_utf8_lossy(&prune.stdout).contains("pruned 1 stale"));
    let pruned: Value = serde_json::from_slice(&fs::read(baseline_path).unwrap()).unwrap();
    assert_eq!(pruned["entries"].as_array().unwrap().len(), 1);
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

    let dry_run = project.run(&["fix", "--dry-run", "input.rs"]);
    assert_code(&dry_run, 1);
    let diff = String::from_utf8_lossy(&dry_run.stdout);
    assert!(diff.contains("--- a/input.rs"));
    assert!(diff.contains("-teh １２３ＡＢＣ"));
    assert!(diff.contains("+the 123ABC"));
    assert_eq!(fs::read_to_string(&input).unwrap(), "teh １２３ＡＢＣ\n");

    let interactive = project.run(&["fix", "--interactive", "input.rs"]);
    assert_code(&interactive, 2);
    assert!(String::from_utf8_lossy(&interactive.stderr).contains("needs an interactive terminal"));

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
fn config_schema_and_validation_cover_editor_workflows() {
    let project = Project::new();
    project.write(
        "ayame-spell.toml",
        "[check]\nmode = \"corrections\"\n[japanese]\nkatakana-style = \"long\"\n",
    );

    let schema = project.run(&["config", "--schema"]);
    assert_code(&schema, 0);
    let document: Value = serde_json::from_slice(&schema.stdout).unwrap();
    assert_eq!(
        document["$id"],
        "https://hjosugi.github.io/ayame-spell/schema/v1/ayame-spell.json"
    );
    assert_eq!(
        document["properties"]["japanese"]["properties"]["katakana-style"]["default"],
        "consistency"
    );

    let valid = project.run(&["config", "--validate"]);
    assert_code(&valid, 0);
    assert!(String::from_utf8_lossy(&valid.stdout).contains("valid:"));

    project.write("invalid.toml", "[japanese]\nkatakana-stle = \"long\"\n");
    let invalid = project.run(&["config", "--validate", "invalid.toml"]);
    assert_code(&invalid, 2);
    let error = String::from_utf8_lossy(&invalid.stderr);
    assert!(error.contains("unknown config key `japanese.katakana-stle`"));
    assert!(error.contains("did you mean `japanese.katakana-style`?"));
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
fn core_flags_override_config_and_file_walking() {
    let project = Project::new();
    project.write("ayame-spell.toml", "[check]\nmode = \"off\"\n");
    project.write("custom.toml", "[check]\nmode = \"corrections\"\n");
    project.write("input.md", "teh\n");

    assert_code(&project.run(&["check", "input.md"]), 0);
    assert_code(
        &project.run(&["check", "--mode", "corrections", "input.md"]),
        1,
    );
    assert_code(
        &project.run(&["check", "--config", "custom.toml", "input.md"]),
        1,
    );
    assert_code(
        &project.run(&["check", "--no-config", "--mode", "off", "input.md"]),
        0,
    );
    assert_code(
        &project.run(&[
            "check",
            "--mode",
            "corrections",
            "--exclude",
            "input.md",
            ".",
        ]),
        0,
    );

    project.write(".gitignore", "ignored.md\n");
    project.write("ignored.md", "teh\n");
    project.write(".hidden.md", "teh\n");
    let walked = project.run(&[
        "check",
        "--mode",
        "corrections",
        "--no-ignore",
        "--hidden",
        "--threads",
        "1",
        ".",
    ]);
    assert_code(&walked, 1);
    let stdout = String::from_utf8_lossy(&walked.stdout);
    assert!(stdout.contains("ignored.md"));
    assert!(stdout.contains(".hidden.md"));

    assert_code(
        &project.run(&[
            "check",
            "--mode",
            "corrections",
            "--max-file-size",
            "2",
            "input.md",
        ]),
        0,
    );
}

#[test]
fn incremental_cache_reuses_identical_results_and_invalidates_on_config() {
    let project = Project::new();
    project.write("ayame-spell.toml", "[check]\nprofile = \"auto\"\n");
    project.write("input.rs", "// recieve this value\n");
    let cache = project.root.join(".scan-cache");
    let cache = cache.to_string_lossy().into_owned();

    let first = project.run(&[
        "check",
        "--format",
        "json",
        "--verbose",
        "--cache-dir",
        &cache,
        "input.rs",
    ]);
    assert_code(&first, 1);
    assert!(String::from_utf8_lossy(&first.stderr).contains("cache hits: 0"));

    let second = project.run(&[
        "check",
        "--format",
        "json",
        "--verbose",
        "--cache-dir",
        &cache,
        "input.rs",
    ]);
    assert_code(&second, 1);
    assert_eq!(first.stdout, second.stdout);
    assert!(String::from_utf8_lossy(&second.stderr).contains("cache hits: 1"));

    project.write(
        "ayame-spell.toml",
        "[check]\nprofile = \"auto\"\n\n[words]\nignore = [\"recieve\"]\n",
    );
    let invalidated = project.run(&[
        "check",
        "--format",
        "json",
        "--verbose",
        "--cache-dir",
        &cache,
        "input.rs",
    ]);
    assert_code(&invalidated, 0);
    assert!(String::from_utf8_lossy(&invalidated.stderr).contains("cache hits: 0"));

    let disabled = project.run(&["check", "--verbose", "--no-cache", "input.rs"]);
    assert_code(&disabled, 0);
    assert!(String::from_utf8_lossy(&disabled.stderr).contains("cache hits: 0"));
}

#[test]
fn quiet_verbose_color_and_stdin_flags_are_end_to_end() {
    let project = Project::new();
    project.write(
        "ayame-spell.toml",
        "[check]\nmode = \"corrections\"\n\
         [[overrides]]\npaths = [\"docs/**\"]\nmode = \"off\"\n",
    );
    project.write("input.md", "teh\n");

    let quiet = project.run(&["check", "--quiet", "input.md"]);
    assert_code(&quiet, 1);
    assert!(quiet.stderr.is_empty());
    assert!(String::from_utf8_lossy(&quiet.stdout).contains("teh"));

    let verbose = project.run(&["check", "--verbose", "input.md"]);
    assert_code(&verbose, 1);
    let stderr = String::from_utf8_lossy(&verbose.stderr);
    assert!(stderr.contains("config root:"));
    assert!(stderr.contains("elapsed:"));

    let always = project.run(&["check", "--color", "always", "input.md"]);
    assert_code(&always, 1);
    assert!(always.stdout.windows(2).any(|bytes| bytes == b"\x1b["));
    let never = project.run(&["check", "--color", "never", "input.md"]);
    assert_code(&never, 1);
    assert!(!never.stdout.windows(2).any(|bytes| bytes == b"\x1b["));

    let mut forced = project.command();
    let forced = forced
        .env_remove("NO_COLOR")
        .env("CLICOLOR_FORCE", "1")
        .args(["check", "input.md"])
        .output()
        .unwrap();
    assert_code(&forced, 1);
    assert!(forced.stdout.windows(2).any(|bytes| bytes == b"\x1b["));

    let stdin = project.run_with_stdin(
        &[
            "check",
            "--format",
            "json",
            "--stdin-filename",
            "docs/input.md",
            "-",
        ],
        b"teh\n",
    );
    assert_code(&stdin, 0);
    assert!(json_records(&stdin)
        .iter()
        .all(|record| record["type"] != "issue"));
}

#[test]
fn words_commands_cover_collect_add_and_noninteractive_triage() {
    let project = Project::new();
    project.write("input.md", "teh teh\n");

    let collect = project.run(&["words", "collect", "--plain", "input.md"]);
    assert_code(&collect, 0);
    assert_eq!(String::from_utf8_lossy(&collect.stdout), "teh\n");
    let completion = project.run(&["__complete", "words-add", "te"]);
    assert_code(&completion, 0);
    assert_eq!(String::from_utf8_lossy(&completion.stdout), "teh\n");

    let add = project.run(&["words", "add", "ProjectTerm"]);
    assert_code(&add, 0);
    let words = fs::read_to_string(project.root.join("ayame-words.txt")).unwrap();
    assert!(words.lines().any(|word| word == "ProjectTerm"));

    project.write("clean.md", "This is clean.\n");
    let triage = project.run(&["words", "triage", "clean.md"]);
    assert_code(&triage, 0);
    assert!(String::from_utf8_lossy(&triage.stdout).contains("nothing to triage"));

    let filtered = project.run(&[
        "words",
        "triage",
        "--kind",
        "ja-variant",
        "--min-count",
        "2",
        "--limit",
        "10",
        "input.md",
    ]);
    assert_code(&filtered, 0);
    assert!(String::from_utf8_lossy(&filtered.stdout).contains("nothing to triage"));

    let non_tty = project.run(&["words", "triage", "input.md"]);
    assert_code(&non_tty, 2);
    assert!(String::from_utf8_lossy(&non_tty.stderr)
        .contains("words triage needs an interactive terminal"));
}

struct RegistryServer {
    url: String,
    index: Arc<Mutex<Vec<u8>>>,
    dictionary: Arc<Mutex<Vec<u8>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl RegistryServer {
    fn start() -> Self {
        let dictionary = b"fixtureword\n".to_vec();
        let index = fixture_registry_index("1.0.0", &dictionary);
        let index = Arc::new(Mutex::new(index));
        let dictionary = Arc::new(Mutex::new(dictionary));
        let thread_index = Arc::clone(&index);
        let thread_dictionary = Arc::clone(&dictionary);

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
                        let (status, content_type, body) = match path {
                            "/index.json" => (
                                "200 OK",
                                "application/json",
                                thread_index.lock().unwrap().clone(),
                            ),
                            "/fixture.txt" => (
                                "200 OK",
                                "text/plain",
                                thread_dictionary.lock().unwrap().clone(),
                            ),
                            _ => ("404 Not Found", "text/plain", b"not found".to_vec()),
                        };
                        write!(
                            stream,
                            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\n\
                             Content-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .unwrap();
                        stream.write_all(&body).unwrap();
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
            index,
            dictionary,
            stop,
            thread: Some(handle),
        }
    }

    fn publish(&self, version: &str, dictionary: &[u8]) {
        *self.dictionary.lock().unwrap() = dictionary.to_vec();
        *self.index.lock().unwrap() = fixture_registry_index(version, dictionary);
    }
}

fn fixture_registry_index(version: &str, dictionary: &[u8]) -> Vec<u8> {
    let digest = Sha256::digest(dictionary)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    json!({
        "version": 2,
        "dictionaries": [{
            "name": "fixture",
            "version": version,
            "language": "en",
            "kind": "wordlist",
            "description": "Fixture dictionary",
            "provenance": "Integration test fixture",
            "file": "fixture.txt",
            "sha256": digest,
            "entries": 1,
            "versions": [{
                "version": version,
                "file": "fixture.txt",
                "sha256": digest,
                "entries": 1
            }],
            "license": "MIT OR Apache-2.0"
        }]
    })
    .to_string()
    .into_bytes()
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

    let list = project
        .command()
        .args(["dict", "--registry", &registry.url, "list"])
        .output()
        .unwrap();
    assert_code(&list, 0);
    assert!(String::from_utf8_lossy(&list.stdout).contains("fixture"));

    let list_json = run(&[
        "dict", "list", "--lang", "en", "--kind", "wordlist", "--json",
    ]);
    assert_code(&list_json, 0);
    let records: Value = serde_json::from_slice(&list_json.stdout).unwrap();
    assert_eq!(records[0]["name"], "fixture");
    assert_eq!(records[0]["installed"], false);
    let add_completion = run(&["__complete", "dict-add", "fix"]);
    assert_code(&add_completion, 0);
    assert_eq!(String::from_utf8_lossy(&add_completion.stdout), "fixture\n");

    let search = run(&["dict", "search", "dictionary"]);
    assert_code(&search, 0);
    assert!(String::from_utf8_lossy(&search.stdout).contains("fixture"));

    let noninteractive = run(&["dict", "add"]);
    assert_code(&noninteractive, 2);
    assert!(String::from_utf8_lossy(&noninteractive.stderr).contains("use `ayame-spell dict list`"));

    let add = run(&["dict", "add", "fixture"]);
    assert_code(&add, 0);
    let cached = project.cache_dir.join("dicts/fixture@1.0.0.txt");
    assert_eq!(fs::read_to_string(&cached).unwrap(), "fixtureword\n");
    let lock = fs::read_to_string(project.root.join("ayame-spell.lock")).unwrap();
    let expected_digest = Sha256::digest(b"fixtureword\n")
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert!(lock.contains("name = \"fixture\""));
    assert!(lock.contains("version = \"1.0.0\""));
    assert!(lock.contains(&expected_digest));
    assert!(fs::read_to_string(project.root.join("ayame-spell.toml"))
        .unwrap()
        .contains("registry:fixture"));
    let remove_completion = run(&["__complete", "dict-remove", "fix"]);
    assert_code(&remove_completion, 0);
    assert_eq!(
        String::from_utf8_lossy(&remove_completion.stdout),
        "fixture\n"
    );

    let info = run(&["dict", "info", "fixture", "--json"]);
    assert_code(&info, 0);
    let info: Value = serde_json::from_slice(&info.stdout).unwrap();
    assert_eq!(info["installed"], true);
    assert_eq!(info["enabled"], true);
    assert_eq!(info["version"], "1.0.0");
    assert_eq!(info["provenance"], "Integration test fixture");
    assert_eq!(info["license"], "MIT OR Apache-2.0");
    assert!(info["source_url"]
        .as_str()
        .unwrap()
        .ends_with("/fixture.txt"));

    let update_check = run(&["dict", "update", "--check"]);
    assert_code(&update_check, 0);
    assert!(String::from_utf8_lossy(&update_check.stdout).contains("up to date fixture@1.0.0"));

    fs::remove_file(&cached).unwrap();
    let restore = run(&["dict", "add", "--cache-only", "fixture"]);
    assert_code(&restore, 0);
    assert_eq!(fs::read_to_string(&cached).unwrap(), "fixtureword\n");

    registry.publish("2.0.0", b"fixtureword-v2\n");
    fs::remove_file(project.cache_dir.join("index.json")).unwrap();
    let update_available = run(&["dict", "update", "--check"]);
    assert_code(&update_available, 1);
    assert!(String::from_utf8_lossy(&update_available.stdout)
        .contains("update available fixture: 1.0.0 -> 2.0.0"));

    let update = run(&["dict", "update"]);
    assert_code(&update, 0);
    assert!(String::from_utf8_lossy(&update.stdout).contains("updated fixture: 1.0.0 -> 2.0.0"));
    let updated_cached = project.cache_dir.join("dicts/fixture@2.0.0.txt");
    assert_eq!(
        fs::read_to_string(&updated_cached).unwrap(),
        "fixtureword-v2\n"
    );

    let vendor = run(&["dict", "vendor", "fixture"]);
    assert_code(&vendor, 0);
    assert_eq!(
        fs::read_to_string(project.root.join("dict/fixture.txt")).unwrap(),
        "fixtureword-v2\n"
    );
    assert!(fs::read_to_string(project.root.join("ayame-spell.toml"))
        .unwrap()
        .contains("dict/fixture.txt"));
    assert!(!fs::read_to_string(project.root.join("ayame-spell.toml"))
        .unwrap()
        .contains("registry:fixture"));

    let remove = run(&["dict", "remove", "fixture"]);
    assert_code(&remove, 0);
    assert!(!cached.exists());
    assert!(!updated_cached.exists());
    assert!(project.root.join("dict/fixture.txt").is_file());
}

#[test]
fn init_config_and_completions_subcommands_work() {
    let project = Project::new();

    let init = project.run(&["init", "--yes"]);
    assert_code(&init, 0);
    assert!(project.root.join("ayame-spell.toml").is_file());

    let config = project.run(&["config"]);
    assert_code(&config, 0);
    assert!(String::from_utf8_lossy(&config.stdout).contains("mode = \"corrections\""));

    let completions = project.run(&["completions", "bash"]);
    assert_code(&completions, 0);
    let completions = String::from_utf8_lossy(&completions.stdout);
    assert!(completions.contains("_ayame-spell"));
    assert!(completions.contains("kind=\"dict-add\""));
    assert!(completions.contains("ayame-spell __complete \"$kind\""));

    project.write("team.words", "ProjectTerm\n");
    let word_file = project.run(&["__complete", "word-file", "team"]);
    assert_code(&word_file, 0);
    assert_eq!(String::from_utf8_lossy(&word_file.stdout), "team.words\n");

    let empty_project = Project::new();
    let empty = empty_project.run(&["__complete", "dict-add", "anything"]);
    assert_code(&empty, 0);
    assert!(empty.stdout.is_empty());
}

#[test]
fn import_commands_migrate_cspell_typos_and_prh_without_silent_loss() {
    let project = Project::new();
    project.write(
        "cspell.json",
        r#"{
          "words": ["AyameProduct"],
          "ignoreWords": ["brandtoken"],
          "ignorePaths": ["generated/**"],
          "dictionaries": ["typescript", "private-team"],
          "language": "en"
        }"#,
    );

    let preview = project.run(&["import", "cspell", "cspell.json", "--dry-run"]);
    assert_code(&preview, 0);
    let preview_stdout = String::from_utf8_lossy(&preview.stdout);
    assert!(preview_stdout.contains("registry:typescript-node"));
    assert!(preview_stdout.contains("AyameProduct"));
    let preview_stderr = String::from_utf8_lossy(&preview.stderr);
    assert!(preview_stderr.contains("private-team"));
    assert!(preview_stderr.contains("language"));
    assert!(!project.root.join("ayame-spell.toml").exists());

    let cspell = project.run(&["import", "cspell", "cspell.json"]);
    assert_code(&cspell, 0);
    let config = fs::read_to_string(project.root.join("ayame-spell.toml")).unwrap();
    assert!(config.contains("generated/**"));
    assert!(config.contains("brandtoken"));
    assert!(config.contains("registry:typescript-node"));
    assert_eq!(
        fs::read_to_string(project.root.join("ayame-words.txt")).unwrap(),
        "AyameProduct\n"
    );

    project.write(
        "_typos.toml",
        r#"
        [default.extend-words]
        teh = "the"
        crateword = "crateword"

        [files]
        extend-exclude = ["vendor/**"]

        [type.rust]
        extend-glob = ["*.rs"]
        "#,
    );
    let typos = project.run(&["import", "typos"]);
    assert_code(&typos, 0);
    assert!(String::from_utf8_lossy(&typos.stderr).contains("type"));
    let config = fs::read_to_string(project.root.join("ayame-spell.toml")).unwrap();
    assert!(config.contains("teh = \"the\""));
    assert!(config.contains("vendor/**"));

    project.write(
        "rules.yml",
        r#"
        version: 1
        meta: team rules
        rules:
          - expected: ウェブサイト
            note: review copy
            patterns:
              - /Web ?サイト/i
          - expected: unsupported
        "#,
    );
    let prh = project.run(&["import", "prh", "rules.yml"]);
    assert_code(&prh, 0);
    let prh_stderr = String::from_utf8_lossy(&prh.stderr);
    assert!(prh_stderr.contains("meta"));
    assert!(prh_stderr.contains("note"));
    assert!(prh_stderr.contains("rule 2"));
    let rules = fs::read_to_string(project.root.join("dict/imported-prh.toml")).unwrap();
    assert!(rules.contains("(?i)Web ?サイト"));
    let config = fs::read_to_string(project.root.join("ayame-spell.toml")).unwrap();
    assert!(config.contains("dict/imported-prh.toml"));

    project.write("input.md", "Web サイトを開く。\n");
    let check = project.run(&["check", "--format", "json", "input.md"]);
    assert_code(&check, 1);
    let records = json_records(&check);
    assert!(records.iter().any(|record| {
        record["kind"] == "ja-variant" && record["suggestions"][0] == "ウェブサイト"
    }));
}

fn lsp_frame(value: Value) -> Vec<u8> {
    let body = value.to_string();
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

fn lsp_messages(bytes: &[u8]) -> Vec<Value> {
    let mut messages = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let header_end = bytes[offset..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|position| offset + position)
            .expect("LSP header");
        let headers = String::from_utf8_lossy(&bytes[offset..header_end]);
        let length: usize = headers
            .lines()
            .find_map(|line| {
                line.strip_prefix("Content-Length:")
                    .map(str::trim)
                    .and_then(|value| value.parse().ok())
            })
            .expect("Content-Length");
        let body_start = header_end + 4;
        let body_end = body_start + length;
        messages.push(serde_json::from_slice(&bytes[body_start..body_end]).expect("LSP JSON"));
        offset = body_end;
    }
    messages
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
fn lsp_full_lifecycle_pull_hover_actions_commands_and_incremental_sync() {
    let project = Project::new();
    let document = project.write("input.md", "This is teh.\n");
    let uri = lsp_types::Url::from_file_path(&document)
        .unwrap()
        .to_string();
    let root_uri = lsp_types::Url::from_directory_path(&project.root)
        .unwrap()
        .to_string();
    let mut input = Vec::new();
    for message in [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "capabilities": {},
                "rootUri": root_uri,
                "initializationOptions": { "locale": "ja-JP", "debounceMs": 0 }
            }
        }),
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri, "languageId": "markdown", "version": 1,
                    "text": "This is teh.\n"
                }
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 10, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 9}}
        }),
        json!({
            "jsonrpc": "2.0", "id": 11, "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 0, "character": 8},
                    "end": {"line": 0, "character": 11}
                },
                "context": {"diagnostics": []}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 12, "method": "textDocument/diagnostic",
            "params": {
                "textDocument": {"uri": uri},
                "identifier": null,
                "previousResultId": null
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 13, "method": "workspace/diagnostic",
            "params": {"identifier": null, "previousResultIds": []}
        }),
        json!({
            "jsonrpc": "2.0", "id": 14, "method": "workspace/executeCommand",
            "params": {
                "command": "ayame-spell.addWords",
                "arguments": [{"words": ["ProjectTerm"], "scope": "project"}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 15, "method": "workspace/executeCommand",
            "params": {
                "command": "ayame-spell.addWords",
                "arguments": [{"words": ["GlobalTerm"], "scope": "global"}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 16, "method": "workspace/executeCommand",
            "params": {
                "command": "ayame-spell.addCorrection",
                "arguments": [{"word": "mistkae", "replacement": "mistake"}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 17, "method": "workspace/executeCommand",
            "params": {"command": "ayame-spell.unknown", "arguments": []}
        }),
        json!({
            "jsonrpc": "2.0", "id": 18, "method": "workspace/executeCommand",
            "params": {
                "command": "ayame-spell.server.fixAll",
                "arguments": [{"uri": uri}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{
                    "range": {
                        "start": {"line": 0, "character": 8},
                        "end": {"line": 0, "character": 11}
                    },
                    "rangeLength": 3,
                    "text": "the"
                }]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 19, "method": "textDocument/diagnostic",
            "params": {
                "textDocument": {"uri": uri},
                "identifier": null,
                "previousResultId": null
            }
        }),
        json!({
            "jsonrpc": "2.0", "method": "$/cancelRequest",
            "params": {"id": 20}
        }),
        json!({
            "jsonrpc": "2.0", "id": 20, "method": "textDocument/diagnostic",
            "params": {
                "textDocument": {"uri": uri},
                "identifier": null,
                "previousResultId": null
            }
        }),
    ] {
        input.extend(lsp_frame(message));
    }

    let large = "a".repeat(4 * 1024 * 1024 + 1);
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0", "method": "textDocument/didChange",
        "params": {
            "textDocument": {"uri": uri, "version": 3},
            "contentChanges": [{"text": large}]
        }
    })));
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0", "id": 21, "method": "textDocument/diagnostic",
        "params": {
            "textDocument": {"uri": uri},
            "identifier": null,
            "previousResultId": null
        }
    })));
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0", "id": 99, "method": "shutdown", "params": null
    })));
    input.extend(lsp_frame(json!({
        "jsonrpc": "2.0", "method": "exit", "params": null
    })));

    let output = project.run_with_stdin(&["lsp", "--stdio"], &input);
    assert_code(&output, 0);
    let messages = lsp_messages(&output.stdout);
    let response = |id: i64| {
        messages
            .iter()
            .find(|message| message["id"] == id)
            .unwrap_or_else(|| panic!("missing LSP response {id}"))
    };

    assert_eq!(
        response(1)["result"]["capabilities"]["textDocumentSync"]["change"],
        2
    );
    assert_eq!(
        response(1)["result"]["capabilities"]["diagnosticProvider"]["workspaceDiagnostics"],
        true
    );
    assert_eq!(response(1)["result"]["capabilities"]["hoverProvider"], true);
    assert!(response(10)["result"]["contents"]["value"]
        .as_str()
        .unwrap()
        .contains("既知のスペルミス"));
    let actions = response(11)["result"].as_array().unwrap();
    for title in [
        "Add \"teh\" to global words",
        "Ignore findings in this file",
        "Ignore this line",
        "Add correction \"teh\"",
        "fix all safe issues",
    ] {
        assert!(
            actions
                .iter()
                .any(|action| action["title"].as_str().unwrap_or("").contains(title)),
            "missing code action {title}"
        );
    }
    assert_eq!(response(12)["result"]["kind"], "full");
    assert_eq!(response(12)["result"]["items"][0]["code"], "typo");
    assert_eq!(response(13)["result"]["items"][0]["kind"], "full");
    assert_eq!(response(17)["error"]["code"], -32602);
    assert_eq!(response(19)["result"]["items"].as_array().unwrap().len(), 0);
    assert_eq!(response(20)["error"]["code"], -32800);
    assert_eq!(response(21)["result"]["items"].as_array().unwrap().len(), 0);
    assert!(messages.iter().any(|message| {
        message["method"] == "window/showMessage"
            && message["params"]["message"]
                .as_str()
                .is_some_and(|value| value.contains("skipped"))
    }));
    assert!(messages.iter().any(|message| {
        message["method"] == "workspace/applyEdit"
            && message["params"]["edit"]["changes"][&uri][0]["newText"] == "the"
    }));
    assert!(fs::read_to_string(project.root.join("ayame-words.txt"))
        .unwrap()
        .contains("ProjectTerm"));
    assert!(fs::read_to_string(project.config_dir.join("words.txt"))
        .unwrap()
        .contains("GlobalTerm"));
    assert!(fs::read_to_string(project.root.join("ayame-spell.toml"))
        .unwrap()
        .contains("mistkae = \"mistake\""));
}

#[test]
fn lsp_normalises_every_japanese_variant_occurrence() {
    let project = Project::new();
    project.write(
        "ayame-spell.toml",
        "[japanese]\nkatakana-style = \"long\"\n",
    );
    let document = project.write("input.md", "サーバ と サーバ\n");
    let uri = lsp_types::Url::from_file_path(document)
        .unwrap()
        .to_string();
    let root_uri = lsp_types::Url::from_directory_path(&project.root)
        .unwrap()
        .to_string();
    let mut input = Vec::new();
    for message in [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}, "rootUri": root_uri}
        }),
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri, "languageId": "markdown", "version": 1,
                    "text": "サーバ と サーバ\n"
                }
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 3}
                },
                "context": {"diagnostics": []}
            }
        }),
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": null}),
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    ] {
        input.extend(lsp_frame(message));
    }

    let output = project.run_with_stdin(&["lsp"], &input);
    assert_code(&output, 0);
    let messages = lsp_messages(&output.stdout);
    let actions = messages.iter().find(|message| message["id"] == 2).unwrap()["result"]
        .as_array()
        .unwrap();
    let normalise = actions
        .iter()
        .find(|action| {
            action["title"]
                .as_str()
                .is_some_and(|title| title.starts_with("Normalise every"))
        })
        .expect("document-wide Japanese normalisation action");
    let edits = normalise["edit"]["changes"][&uri].as_array().unwrap();
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().all(|edit| edit["newText"] == "サーバー"));
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
