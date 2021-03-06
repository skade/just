extern crate tempdir;
extern crate brev;
extern crate regex;

use tempdir::TempDir;
use super::std::{fs, path, process};

fn integration_test(
  args:            &[&str],
  justfile:        &str,
  expected_status: i32,
  expected_stdout: &str,
  expected_stderr: &str,
) {
  let tmp = TempDir::new("just-integration")
    .unwrap_or_else(|err| panic!("integration test: failed to create temporary directory: {}", err));
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(path, justfile);
  let mut binary = super::std::env::current_dir().unwrap();
  binary.push("./target/debug/just");
  let output = process::Command::new(binary)
    .current_dir(tmp.path())
    .args(args)
    .output()
    .expect("just invocation failed");

  let mut failure = false;

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {} != {}", status, expected_status);
    failure = true;
  }

  let stdout = super::std::str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("bad stdout:\ngot:\n{}\n\nexpected:\n{}", stdout, expected_stdout);
    failure = true;
  }

  let stderr = super::std::str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("bad stderr:\ngot:\n{}\n\nexpected:\n{}", stderr, expected_stderr);
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}

fn search_test<P: AsRef<path::Path>>(path: P) {
  let mut binary = super::std::env::current_dir().unwrap();
  binary.push("./target/debug/just");
  let output = process::Command::new(binary)
    .current_dir(path)
    .output()
    .expect("just invocation failed");

  assert_eq!(output.status.code().unwrap(), 0);

  let stdout = super::std::str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "ok\n");

  let stderr = super::std::str::from_utf8(&output.stderr).unwrap();
  assert_eq!(stderr, "echo ok\n");
}

#[test]
fn test_justfile_search() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path);
}

#[test]
fn test_capitalized_justfile_search() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("Justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path);
}

#[test]
fn test_capitalization_priority() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();
  path.push("Justfile");
  brev::dump(&path, "default:\n\techo fail");
  path.pop();

  // if we see "default\n\techo fail" in `justfile` then we're running
  // in a case insensitive filesystem, so just bail
  path.push("justfile");
  if brev::slurp(&path) == "default:\n\techo fail" {
    return;
  }
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path);
}

#[test]
fn default() {
  integration_test(
    &[],
    "default:\n echo hello\nother: \n echo bar",
    0,
    "hello\n",
    "echo hello\n",
  )
}

#[test]
fn quiet() {
  integration_test(
    &[],
    "default:\n @echo hello",
    0,
    "hello\n",
    "",
  )
}

#[test]
fn order() {
  let text = "
b: a
  echo b
  @mv a b

a:
  echo a
  @touch F
  @touch a

d: c
  echo d
  @rm c

c: b
  echo c
  @mv b c";
  integration_test(
    &["a", "d"],
    text,
    0,
    "a\nb\nc\nd\n",
    "echo a\necho b\necho c\necho d\n",
  );
}

#[test]
fn list() {
  let text = 
"b: a
a:
d: c
c: b";
  integration_test(
    &["--list"],
    text,
    0,
    "a b c d\n",
    "",
  );
}

#[test]
fn select() {
  let text = 
"b:
  @echo b
a:
  @echo a
d:
  @echo d
c:
  @echo c";
  integration_test(
    &["d", "c"],
    text,
    0,
    "d\nc\n",
    "",
  );
}

#[test]
fn print() {
  let text = 
"b:
  echo b
a:
  echo a
d:
  echo d
c:
  echo c";
  integration_test(
    &["d", "c"],
    text,
    0,
    "d\nc\n",
    "echo d\necho c\n",
  );
}


#[test]
fn show() {
  let text = 
r#"hello = "foo"
bar = hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#;
  integration_test(
    &["--show", "recipe"],
    text,
    0,
    r#"recipe:
    echo {{hello + "bar" + bar}}
"#,
    "",
  );
}
/*
#[test]
fn debug() {
  let text = 
r#"hello = "foo"
bar = hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#;
  integration_test(
    &["--debug"],
    text,
    0,
    r#"bar = hello + hello # "foofoo"

hello = "foo" # "foo"

recipe:
    echo {{hello + "bar" + bar # "foobarfoofoo"}}
"#,
    "",
  );
}
*/

#[test]
fn status_passthrough() {
  let text = 
"
recipe:
 @exit 100";
  integration_test(
    &[],
    text,
    100,
    "",
    "Recipe \"recipe\" failed with exit code 100\n",
  );
}

#[test]
fn error() {
  integration_test(
    &[],
    "bar:\nhello:\nfoo: bar baaaaaaaz hello",
    255,
    "",
    "error: recipe `foo` has unknown dependency `baaaaaaaz`
  |
3 | foo: bar baaaaaaaz hello
  |          ^^^^^^^^^
",
  );
}

#[test]
fn backtick_success() {
  integration_test(
    &[],
    "a = `printf Hello,`\nbar:\n printf '{{a + `printf ' world!'`}}'",
    0,
    "Hello, world!",
    "printf 'Hello, world!'\n",
  );
}

#[test]
fn backtick_trimming() {
  integration_test(
    &[],
    "a = `echo Hello,`\nbar:\n echo '{{a + `echo ' world!'`}}'",
    0,
    "Hello, world!\n",
    "echo 'Hello, world!'\n",
  );
}

#[test]
fn backtick_code_assignment() {
  integration_test(
    &[],
    "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
    100,
    "",
    "backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation() {
  integration_test(
    &[],
    "b = a\na = `echo hello`\nbar:\n echo '{{`exit 200`}}'",
    200,
    "",
    "backtick failed with exit code 200
  |
4 |  echo '{{`exit 200`}}'
  |          ^^^^^^^^^^
",
  );
}

#[test]
fn shebang_backtick_failure() {
  integration_test(
    &[],
    "foo:
 #!/bin/sh
 echo hello
 echo {{`exit 123`}}",
    123,
    "",
    "backtick failed with exit code 123
  |
4 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  );
}

#[test]
fn command_backtick_failure() {
  integration_test(
    &[],
    "foo:
 echo hello
 echo {{`exit 123`}}",
    123,
    "hello\n",
    "echo hello\nbacktick failed with exit code 123
  |
3 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  );
}

#[test]
fn assignment_backtick_failure() {
  integration_test(
    &[],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    222,
    "",
    "backtick failed with exit code 222
  |
4 | a = `exit 222`
  |     ^^^^^^^^^^
",
  );
}

#[test]
fn unknown_override_options() {
  integration_test(
    &["--set", "foo", "bar", "a", "b", "--set", "baz", "bob", "--set", "a", "b"],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    255,
    "",
    "baz and foo set on the command line but not present in justfile\n",
  );
}

#[test]
fn unknown_override_args() {
  integration_test(
    &["foo=bar", "baz=bob", "a=b", "a", "b"],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    255,
    "",
    "baz and foo set on the command line but not present in justfile\n",
  );
}

#[test]
fn overrides_first() {
  integration_test(
    &["foo=bar", "a=b", "recipe", "baz=bar"],
    r#"
foo = "foo"
a = "a"
baz = "baz"
    
recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    0,
    "arg=baz=bar\nbarbbaz\n",
    "echo arg=baz=bar\necho barbbaz\n",
  );
}

// shebangs are printed

#[test]
fn dry_run() {
  integration_test(
    &["--dry-run", "shebang", "command"],
    r#"
var = `echo stderr 1>&2; echo backtick`

command:
  @touch /this/is/not/a/file
  {{var}}
  echo {{`echo command interpolation`}}

shebang:
  #!/bin/sh
  touch /this/is/not/a/file
  {{var}}
  echo {{`echo shebang interpolation`}}"#,
    0,
    "",
    "stderr
#!/bin/sh
touch /this/is/not/a/file
backtick
echo shebang interpolation
touch /this/is/not/a/file
backtick
echo command interpolation
",
  );
}

#[test]
fn evaluate() {
  integration_test(
    &["--evaluate"],
    r#"
foo = "a\t"
baz = "c"
bar = "b\t"
abc = foo + bar + baz

wut:
  touch /this/is/not/a/file
"#,
    0,
    r#"abc = "a	b	c"
bar = "b	"
baz = "c"
foo = "a	"
"#,
    "",
  );
}

#[test]
fn export_success() {
  integration_test(
    &[],
    r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  echo $foo $bar $abc
"#,
    0,
    "a b abc\n",
    "echo $foo $bar $abc\n",
  );
}

#[test]
fn export_shebang() {
  integration_test(
    &[],
    r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  #!/bin/sh
  echo $foo $bar $abc
"#,
    0,
    "a b abc\n",
    "",
  );
}

#[test]
fn export_recipe_backtick() {
  integration_test(
    &[],
    r#"
export exported_variable = "A-IS-A"

recipe:
  echo {{`echo recipe $exported_variable`}}
"#,
    0,
    "recipe A-IS-A\n",
    "echo recipe A-IS-A\n",
  );
}

#[test]
fn raw_string() {
  integration_test(
    &[],
    r#"
export exported_variable = '\\\\\\"'

recipe:
  echo {{`echo recipe $exported_variable`}}
"#,
    0,
    "recipe \\\"\n",
    "echo recipe \\\\\\\"\n",
  );
}

#[test]
fn line_error_spacing() {
  integration_test(
    &[],
    r#"








???
"#,
    255,
    "",
    "error: unknown start of token:
   |
10 | ???
   | ^
",
  );
}

#[test]
fn quiet_flag_no_stdout() {
  integration_test(
    &["--quiet"],
    r#"
default:
  @echo hello
"#,
    0,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_stderr() {
  integration_test(
    &["--quiet"],
    r#"
default:
  @echo hello 1>&2
"#,
    0,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_command_echoing() {
  integration_test(
    &["--quiet"],
    r#"
default:
  exit
"#,
    0,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_error_messages() {
  integration_test(
    &["--quiet"],
    r#"
default:
  exit 100
"#,
    100,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_assignment_backtick_stderr() {
  integration_test(
    &["--quiet"],
    r#"
a = `echo hello 1>&2`
default:
  exit 100
"#,
    100,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_interpolation_backtick_stderr() {
  integration_test(
    &["--quiet"],
    r#"
default:
  echo `echo hello 1>&2`
  exit 100
"#,
    100,
    "",
    "",
  );
}

#[test]
fn quiet_flag_or_dry_run_flag() {
  integration_test(
    &["--quiet", "--dry-run"],
    r#""#,
    255,
    "",
    "--dry-run and --quiet may not be used together\n",
  );
}
