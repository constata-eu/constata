{% extends "emails/in_layout/_layout.html" %}
{% import "emails/_macros.html.es" as macros %}

{% block main %}
<br/>
  {{ macros::hello_and_document(document_friendly_name = document_friendly_name) }}

  {{ macros::terms_acceptance(has_enough_tokens = has_enough_tokens, url_to_tyc = url_to_tyc, is_email_for_parked = false, parked_count = parked_count) }}

  {{ macros::welcome_and_start(give_welcome = give_welcome, document_friendly_name = document_friendly_name, accepted = accepted, eta = eta, url_to_tyc = url_to_tyc) }}

  {{ macros::cost_and_give_gift(has_enough_tokens = has_enough_tokens, gift = gift, cost = cost, missing_tokens_for_other = missing_tokens_for_other)}}

  {{ macros::enough_tokens(has_enough_tokens = has_enough_tokens, missing_tokens = missing_tokens, total_price = total_price, buy_tokens_link = buy_tokens_link) }}
<br/>

{% endblock main %}

{% block footer %}
  {% include "emails/_account_state.html.es" %}
  {% include "emails/_footer.html.es" %}
{% endblock footer %}
