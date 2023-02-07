use galvanic_assert::*;

#[allow(dead_code)]
pub fn rematch<'a>(expr: &'a str) -> Box<dyn Matcher<'a, String> + 'a> {
  Box::new(move |actual: &String| {
    let re = regex::Regex::new(expr).unwrap();
    let builder = MatchResultBuilder::for_("rematch");
    if re.is_match(actual) {
      builder.matched()
    } else {
      builder.failed_because(&format!("{:?} does not match {:?}", expr, actual))
    }
  })
}
