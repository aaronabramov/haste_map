use regex::Regex;

lazy_static! {
  // Adapted from https://github.com/facebook/jest/blob/master/packages/jest-haste-map/src/lib/extract_requires.js
  static ref DYNAMIC_IMPORT: Regex = Regex::new(r#"(?:^|[^.]\s*)(\bimport\s*?\(\s*?)([`'"])([^`'"]+)([`'"]\))"#).unwrap();
    // TODO: Filter out `import type ...`
    static ref EXPORT: Regex = Regex::new(r#"(\bexport\s+(?:[^'"]+\s+from\s+)??)(['"])([^'"]+)(['"])"#).unwrap();
    // TODO: Filter out `import type ...`
    static ref IMPORT: Regex = Regex::new(r#"(\bimport\s+(?:[^'"]+\s+from\s+)??)(['"])([^'"]+)(['"])"#).unwrap();
    static ref REQUIRE_JEST: Regex = Regex::new(r#"(?:^|[^.]\s*)(\b(?:require\s*?\.\s*?(?:requireActual|requireMock)|jest\s*?\.\s*?(?:requireActual|requireMock|genMockFromModule))\s*?\(\s*?)([`'"])([^`'"]+)([`'"]\))"#).unwrap();
    static ref REQUIRE: Regex = Regex::new(r#"(?:^|[^.]\s*)(\brequire\s*?\(\s*?)([`'"])([^`'"]+)([`'"]\))"#).unwrap();
}

pub fn parse(content: &String) -> Vec<String> {
  let patterns: Vec<&Regex> = vec![&IMPORT, &EXPORT, &DYNAMIC_IMPORT, &REQUIRE, &REQUIRE_JEST];

  let captures: Vec<String> = patterns
    .iter()
    .flat_map(|pattern| {
      pattern
        .captures_iter(&content)
        .map(|c| String::from(c.get(3).unwrap().as_str()))
    })
    .collect();
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
    assert_eq!(result, vec!["a", "b", "c"]);
  }

  #[test]
  fn export() {
    let content = String::from(
      r#"
      export {a, b} from 'a';
      export * from 'b';
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
}
