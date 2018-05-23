use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;
use types::SourceFile;
use utils;

lazy_static!{}

pub fn parse_chunk(chunk: &[PathBuf]) -> Vec<SourceFile> {
  let now = Instant::now();
  let patterns = make_patterns();
  let parsed_chunk: Vec<SourceFile> = chunk
    .iter()
    .map(|path| (path.clone(), File::open(&path)))
    .filter(|tuple| tuple.1.is_ok())
    .map(|(path, open)| (path, open.unwrap()))
    .map(|(path, mut file)| {
      let mut content = String::new();
      let result = file.read_to_string(&mut content);
      (path, content, result)
    })
    .filter(|tuple| tuple.2.is_ok())
    .map(|(path, content, _)| {
      let result = SourceFile {
        path,
        dependencies: extract_deps(&content, &patterns),
      };
      result
    })
    .collect();
  utils::skip_log_time(now, &"PARSED CHUNK");
  parsed_chunk
}

fn extract_deps(content: &String, patterns: &Vec<Regex>) -> Vec<String> {
  let captures: Vec<String> = patterns
    .iter()
    .flat_map(|pattern| {
      pattern
        .captures_iter(&content)
        .filter(|c| !c.name("type").is_some()) // Ignore type imports.
        .map(|c| String::from(c.name("module").unwrap().as_str()))
    })
    .collect();
  captures
}

fn make_patterns() -> Vec<Regex> {
  // Adapted from https://github.com/facebook/jest/blob/master/packages/jest-haste-map/src/lib/extract_requires.js
  vec![
   Regex::new(r#"(?:^|[^.]\s*)(\bimport\s*?\(\s*?)([`'"])(?P<module>[^`'"]+)([`'"]\))"#).unwrap(),
   Regex::new(r#"(\bexport\s+(?P<type>type )?(?:[^'"]+\s+from\s+)??)(['"])(?P<module>[^'"]+)(['"])"#).unwrap(),
   Regex::new(r#"(\bimport\s+(?P<type>type )?(?:[^'"]+\s+from\s+)??)(['"])(?P<module>[^'"]+)(['"])"#).unwrap(),
   Regex::new(r#"(?:^|[^.]\s*)(\b(?:require\s*?\.\s*?(?:requireActual|requireMock)|jest\s*?\.\s*?(?:requireActual|requireMock|genMockFromModule))\s*?\(\s*?)([`'"])(?P<module>[^`'"]+)([`'"]\))"#).unwrap(),
   Regex::new(r#"(?:^|[^.]\s*)(\brequire\s*?\(\s*?)([`'"])(?P<module>[^`'"]+)([`'"]\))"#).unwrap(),
  ]
}

#[cfg(test)]
mod test {
  #[test]
  fn require() {
    let content = String::from(
      r#"
      require('a');
      const a = () => { require('b') };
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    assert_eq!(result, vec!["a", "b"]);
  }

  #[test]
  fn import() {
    let content = String::from(
      r#"
      import a from 'a';
      import type B from 'b';
      import {a, b, c} from 'c';
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    assert_eq!(result, vec!["a", "c"]);
  }

  #[test]
  fn export() {
    let content = String::from(
      r#"
      export {a, b} from 'a';
      export * from 'b';
      export type {c} from 'c';
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    assert_eq!(result, vec!["a", "b"]);
  }

  #[test]
  fn dynamic_import() {
    let content = String::from(
      r#"
      import('a').then(() => {});
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    assert_eq!(result, vec!["a"]);
  }

  #[test]
  fn multiline_import() {
    let content = String::from(
      r#"
      import {
        foo,
        bar,
      } from "a";
      import foo as bar, {
        baz,
        biz,
      } from "b";
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    assert_eq!(result, vec!["a", "b"]);
  }

  #[test]
  fn type_import() {
    let content = String::from(
      r#"
      import type MyType from "a";
      import type { MyOtherType } from "b";
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    let expected: Vec<&str> = vec![];
    assert_eq!(result, expected);
  }

  #[test]
  fn jest_require() {
    let content = String::from(
      r#"
      require.requireActual('a');
      require.requireMock('b');
    "#,
    );
    let result = super::extract_deps(&content, &super::make_patterns());
    assert_eq!(result, vec!["a", "b"]);
  }
}
