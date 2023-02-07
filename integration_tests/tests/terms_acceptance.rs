constata_lib::describe_one! {
  use integration_tests::*;

  integration_test!{ can_accept_terms_and_conditions_in_website (c, d)
    let person = c.make_person().await;
    let tyc = person.get_or_create_terms_acceptance().await?;
    d.goto(&format!("http://localhost:8000/terms_acceptance/{}", tyc.token())).await;

    d.click(".footer-dialog .btn-reject").await;
    d.wait_for_text(".modal.reject p", r"You wont be able to use.*").await;
    d.click(".modal.reject .btn-back").await;
    d.click(".footer-dialog .btn-accept").await;
    d.wait_for_text(".modal.confirm", r"I confirm to have read.*").await;
    d.click(".modal.confirm .btn-confirm").await;
    d.wait_for_text(".footer-dialog", r"You have accepted our Terms and Conditions.*").await;

    d.goto("http://localhost:8000/terms_acceptance/").await;
    d.driver.query(By::Css(".footer-dialog")).not_exists().await?;
  }
}
