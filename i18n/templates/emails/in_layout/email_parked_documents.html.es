{% extends "emails/in_layout/_layout.html" %}
{% import "emails/_macros.html.es" as macros %}

{% block main %}
<br/>
  ¡Hola! Te recordamos que tienes <b><i> {{ parked_count }} documento{{ parked_count | pluralize }} pendiente{{ parked_count | pluralize }}</i></b>.

  {{ macros::terms_acceptance(has_enough_tokens = has_enough_tokens, url_to_tyc = url_to_tyc, is_email_for_parked = true, parked_count = parked_count) }}

  {{ macros::enough_tokens(has_enough_tokens = has_enough_tokens, missing_tokens = missing_tokens, total_price = total_price, buy_tokens_link = buy_tokens_link) }}

  <br/>

{% endblock main %}

{% block footer %}
  {% include "emails/_account_state.html.es" %}
  {% include "emails/_footer.html.es" %}
{% endblock footer %}
