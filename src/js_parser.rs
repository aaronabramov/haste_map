use regex::Regex;

lazy_static! {
  // Adapted from https://github.com/facebook/jest/blob/master/packages/jest-haste-map/src/lib/extract_requires.js
  static ref BLOCK_COMMENT: Regex = Regex::new(r#"/\*(.|\s)*?\*/"#).unwrap();
  static ref LINE_COMMENT: Regex = Regex::new(r#"//.*"#).unwrap();

  static ref DYNAMIC_IMPORT: Regex = Regex::new(r#"(?:^|[^.]\s*)(\bimport\s*?\(\s*?)([`'"])(?P<module>[^`'"]+)([`'"]\))"#).unwrap();
  static ref EXPORT: Regex = Regex::new(r#"(\bexport\s+(?P<type>type )?(?:[^'"]+\s+from\s+)??)(['"])(?P<module>[^'"]+)(['"])"#).unwrap();
  static ref IMPORT: Regex = Regex::new(r#"(\bimport\s+(?P<type>type )?(?:[^'"]+\s+from\s+)??)(['"])(?P<module>[^'"]+)(['"])"#).unwrap();
  static ref REQUIRE_JEST: Regex = Regex::new(r#"(?:^|[^.]\s*)(\b(?:require\s*?\.\s*?(?:requireActual|requireMock)|jest\s*?\.\s*?(?:requireActual|requireMock|genMockFromModule))\s*?\(\s*?)([`'"])(?P<module>[^`'"]+)([`'"]\))"#).unwrap();
  static ref REQUIRE: Regex = Regex::new(r#"(?:^|[^.]\s*)(\brequire\s*?\(\s*?)([`'"])(?P<module>[^`'"]+)([`'"]\))"#).unwrap();
}

// Returns a unique sorted list of dependencies.
pub fn parse(content: &String) -> Vec<String> {
  let patterns: Vec<&Regex> = vec![&IMPORT, &EXPORT, &DYNAMIC_IMPORT, &REQUIRE, &REQUIRE_JEST];
  let comment_patterns: Vec<&Regex> = vec![&LINE_COMMENT, &BLOCK_COMMENT];

  let clean_content: String = comment_patterns.iter().fold(content.to_string(), |c, p| {
    p.replace_all(&c, "").to_string()
  });

  let mut captures: Vec<String> = patterns
    .iter()
    .flat_map(|pattern| {
      pattern
        .captures_iter(&clean_content)
        .filter(|c| !c.name("type").is_some()) // Ignore type imports.
        .map(|c| String::from(c.name("module").unwrap().as_str()))
    })
    .collect();

  captures.sort();
  captures.dedup();
  captures
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
    let result = super::parse(&content);
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
    let result = super::parse(&content);
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
    let result = super::parse(&content);
    assert_eq!(result, vec!["a", "b"]);
  }

  #[test]
  fn dynamic_import() {
    let content = String::from(
      r#"
      import('a').then(() => {});
    "#,
    );
    let result = super::parse(&content);
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
    let result = super::parse(&content);
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
    let result = super::parse(&content);
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
    let result = super::parse(&content);
    assert_eq!(result, vec!["a", "b"]);
  }

  #[test]
  fn ignore_line_comments() {
    let content = String::from(
      r#"
      // require.requireActual('a');
      // require.requireMock('b');
    "#,
    );
    let result = super::parse(&content);
    let expected: Vec<&str> = vec![];
    assert_eq!(result, expected);
  }

  #[test]
  fn ignore_block_comments() {
    let content = String::from(
      r#"
      /*
        require.requireActual('a');
        require.requireMock('b');
      */
    "#,
    );
    let result = super::parse(&content);
    let expected: Vec<&str> = vec![];
    assert_eq!(result, expected);
  }

  #[test]
  fn dedupes_duplicate_imports() {
    let content = String::from(
      r#"
        if(foo) {
          require('a');
        }
        if(bar) {
          require('a');
        }
        require('b');
    "#,
    );
    let result = super::parse(&content);
    assert_eq!(result, vec!["a", "b"]);
  }
}
